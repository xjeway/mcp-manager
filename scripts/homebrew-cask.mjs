import crypto from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'

import { collectCliFormulaAssets, renderFormula } from './homebrew-formula.mjs'

const REQUIRED_ARCHES = ['arm', 'intel']

function assetArch(assetName) {
  if (!assetName.endsWith('.app.tar.gz')) {
    return null
  }
  if (/(^|[._-])(aarch64|arm64)([._-]|$)/i.test(assetName)) {
    return 'arm'
  }
  if (/(^|[._-])(x64|x86_64|amd64)([._-]|$)/i.test(assetName)) {
    return 'intel'
  }
  return null
}

export function collectMacosCaskAssets(assets) {
  const selected = {}

  for (const asset of assets) {
    const arch = assetArch(asset.name ?? '')
    if (arch && !selected[arch]) {
      selected[arch] = asset
    }
  }

  const missing = REQUIRED_ARCHES.filter((arch) => !selected[arch])
  if (missing.length > 0) {
    throw new Error(`Missing required macOS Homebrew assets: ${missing.join(', ')}`)
  }

  return {
    arm: selected.arm,
    intel: selected.intel,
  }
}

export function parseVersionFromTag(tagName) {
  const version = tagName.replace(/^v/, '')
  if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version)) {
    throw new Error(`Homebrew cask publishing requires a stable semver tag: ${tagName}`)
  }
  return version
}

export function renderCask({ assets, owner, repo, version }) {
  return `cask "mcp-manager" do
  version "${version}"
  sha256 arm: "${assets.arm.sha256}",
        intel: "${assets.intel.sha256}"

  asset_name = Hardware::CPU.arm? ? "${assets.arm.name}" : "${assets.intel.name}"
  url "https://github.com/${owner}/${repo}/releases/download/v#{version}/#{asset_name}"
  name "MCP Manager"
  desc "Desktop app for managing MCP server configs"
  homepage "https://github.com/${owner}/${repo}"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "MCP Manager.app"
end
`
}

export function sha256FromAssetDigest(asset) {
  const match = /^sha256:([a-f0-9]{64})$/i.exec(asset.digest ?? '')
  return match ? match[1].toLowerCase() : null
}

export function parseArgs(argv) {
  const args = {
    command: argv[2],
    owner: null,
    repo: null,
    event: process.env.GITHUB_EVENT_PATH,
    tag: null,
    tapDir: 'tap',
  }

  for (let index = 3; index < argv.length; index += 1) {
    const arg = argv[index]
    const value = argv[index + 1]
    if (arg === '--owner') {
      args.owner = value
      index += 1
    } else if (arg === '--repo') {
      args.repo = value
      index += 1
    } else if (arg === '--event') {
      args.event = value
      index += 1
    } else if (arg === '--tap-dir') {
      args.tapDir = value
      index += 1
    } else if (arg === '--tag') {
      args.tag = value
      index += 1
    } else {
      throw new Error(`Unknown argument: ${arg}`)
    }
  }

  if (!args.owner || !args.repo) {
    const [owner, repo] = (process.env.GITHUB_REPOSITORY ?? '').split('/')
    args.owner ||= owner
    args.repo ||= repo
  }

  return args
}

async function fetchJson(url, token) {
  const response = await fetch(url, {
    headers: {
      Accept: 'application/vnd.github+json',
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      'X-GitHub-Api-Version': '2022-11-28',
    },
  })
  if (!response.ok) {
    throw new Error(`GitHub API request failed ${response.status}: ${url}`)
  }
  return response.json()
}

async function loadRelease({ eventPath, owner, repo }) {
  const event = eventPath && fs.existsSync(eventPath) ? JSON.parse(fs.readFileSync(eventPath, 'utf8')) : {}
  const release = event.release
  if (!release?.tag_name || !release?.id) {
    throw new Error('GitHub release event payload is missing release.tag_name')
  }

  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN
  const assets = await fetchJson(
    `https://api.github.com/repos/${owner}/${repo}/releases/${release.id}/assets?per_page=100`,
    token,
  )

  return {
    ...release,
    assets,
  }
}

async function loadReleaseByTag({ owner, repo, tag }) {
  if (!tag) {
    throw new Error('Missing release tag. Pass --tag for manual dispatch.')
  }

  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN
  const release = await fetchJson(
    `https://api.github.com/repos/${owner}/${repo}/releases/tags/${encodeURIComponent(tag)}`,
    token,
  )
  const assets = await fetchJson(
    `https://api.github.com/repos/${owner}/${repo}/releases/${release.id}/assets?per_page=100`,
    token,
  )

  return {
    ...release,
    assets,
  }
}

async function sha256ForUrl(url) {
  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN
  const response = await fetch(url, {
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  })
  if (!response.ok) {
    throw new Error(`Asset download failed ${response.status}: ${url}`)
  }

  const hash = crypto.createHash('sha256')
  const reader = response.body.getReader()
  while (true) {
    const { done, value } = await reader.read()
    if (done) {
      break
    }
    hash.update(value)
  }
  return hash.digest('hex')
}

async function sha256ForAsset(asset) {
  return sha256FromAssetDigest(asset) ?? (await sha256ForUrl(asset.browser_download_url))
}

async function publish(args) {
  const release = args.tag
    ? await loadReleaseByTag({
        owner: args.owner,
        repo: args.repo,
        tag: args.tag,
      })
    : await loadRelease({
        eventPath: args.event,
        owner: args.owner,
        repo: args.repo,
      })
  const version = parseVersionFromTag(release.tag_name)
  const selected = collectMacosCaskAssets(release.assets)
  const assets = {
    arm: {
      name: selected.arm.name,
      sha256: await sha256ForAsset(selected.arm),
    },
    intel: {
      name: selected.intel.name,
      sha256: await sha256ForAsset(selected.intel),
    },
  }

  const caskPath = path.join(args.tapDir, 'Casks', 'mcp-manager.rb')
  fs.mkdirSync(path.dirname(caskPath), { recursive: true })
  fs.writeFileSync(
    caskPath,
    renderCask({
      version,
      assets,
      owner: args.owner,
      repo: args.repo,
    }),
  )
  console.log(`Rendered Homebrew cask for ${release.tag_name}: ${caskPath}`)

  const cliAssets = collectCliFormulaAssets(release.assets)
  if (!cliAssets) {
    console.log(`No CLI archives in ${release.tag_name}; leaving the mcpmgr formula as is.`)
    return
  }
  for (const byArch of Object.values(cliAssets)) {
    for (const [arch, asset] of Object.entries(byArch)) {
      byArch[arch] = { name: asset.name, sha256: await sha256ForAsset(asset) }
    }
  }

  const formulaPath = path.join(args.tapDir, 'Formula', 'mcpmgr.rb')
  fs.mkdirSync(path.dirname(formulaPath), { recursive: true })
  fs.writeFileSync(
    formulaPath,
    renderFormula({
      version,
      assets: cliAssets,
      owner: args.owner,
      repo: args.repo,
    }),
  )
  console.log(`Rendered Homebrew formula for ${release.tag_name}: ${formulaPath}`)
}

async function main() {
  const args = parseArgs(process.argv)
  if (args.command !== 'publish') {
    throw new Error('Usage: node scripts/homebrew-cask.mjs publish --tap-dir tap --owner OWNER --repo REPO')
  }
  await publish(args)
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error(error.message)
    process.exit(1)
  })
}

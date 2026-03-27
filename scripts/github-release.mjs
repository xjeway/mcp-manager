import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

import { isPrereleaseRef } from './release-args.mjs'

const apiBaseUrl = (process.env.GITHUB_API_URL || 'https://api.github.com').replace(/\/$/, '')
const uploadsBaseUrl = 'https://uploads.github.com'
const apiVersion = '2022-11-28'

const installerDefinitions = [
  { suffix: '.app.tar.gz.sig', os: 'darwin', installer: 'app' },
  { suffix: '.AppImage.sig', os: 'linux', installer: 'appimage' },
  { suffix: '.deb.sig', os: 'linux', installer: 'deb' },
  { suffix: '.rpm.sig', os: 'linux', installer: 'rpm' },
  { suffix: '.msi.sig', os: 'windows', installer: 'msi' },
  { suffix: '.msi.zip.sig', os: 'windows', installer: 'msi' },
  { suffix: '-setup.exe.sig', os: 'windows', installer: 'nsis' },
  { suffix: '.exe.sig', os: 'windows', installer: 'nsis' },
  { suffix: '.nsis.zip.sig', os: 'windows', installer: 'nsis' },
]

const primaryInstallerPriority = {
  darwin: { app: 100 },
  linux: { appimage: 100, deb: 90, rpm: 80 },
  windows: { msi: 100, nsis: 90 },
}

function getRequiredEnv(name) {
  const value = process.env[name]
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`)
  }
  return value
}

function getRepositoryContext() {
  const repository = getRequiredEnv('GITHUB_REPOSITORY')
  const [owner, repo] = repository.split('/', 2)

  if (!owner || !repo) {
    throw new Error(`Invalid GITHUB_REPOSITORY value: ${repository}`)
  }

  return { owner, repo }
}

function githubHeaders(overrides = {}) {
  return {
    Accept: 'application/vnd.github+json',
    Authorization: `Bearer ${getRequiredEnv('GITHUB_TOKEN')}`,
    'User-Agent': 'mcp-manager-release-script',
    'X-GitHub-Api-Version': apiVersion,
    ...overrides,
  }
}

async function githubJson(pathname, { method = 'GET', body } = {}) {
  const response = await fetch(`${apiBaseUrl}${pathname}`, {
    method,
    headers: githubHeaders(
      body
        ? {
            'Content-Type': 'application/json',
          }
        : {},
    ),
    body: body ? JSON.stringify(body) : undefined,
  })

  if (!response.ok) {
    throw new Error(
      `GitHub API ${method} ${pathname} failed: ${response.status} ${await response.text()}`,
    )
  }

  if (response.status === 204) {
    return null
  }

  return response.json()
}

async function githubText(url, { accept = 'application/octet-stream' } = {}) {
  const response = await fetch(url, {
    headers: githubHeaders({ Accept: accept }),
  })

  if (!response.ok) {
    throw new Error(`GitHub download failed: ${response.status} ${await response.text()}`)
  }

  return response.text()
}

async function listReleases(owner, repo) {
  const releases = []

  for (let page = 1; ; page += 1) {
    const batch = await githubJson(`/repos/${owner}/${repo}/releases?per_page=100&page=${page}`)
    releases.push(...batch)

    if (batch.length < 100) {
      break
    }
  }

  return releases
}

function writeGithubOutputs(outputs) {
  const outputPath = process.env.GITHUB_OUTPUT
  const lines = Object.entries(outputs).map(([key, value]) => `${key}=${value}`)

  if (outputPath) {
    fs.appendFileSync(outputPath, `${lines.join('\n')}\n`)
  } else {
    for (const line of lines) {
      console.log(line)
    }
  }
}

export function buildDefaultReleaseBody(tagName) {
  return [
    `Automated release for ${tagName}.`,
    '',
    'Download the installer or archive that matches your platform from the assets below.',
    '',
    'macOS note: public macOS builds may still be unsigned while Apple signing is being prepared. If Gatekeeper blocks `MCP Manager.app`, move it to `/Applications`, try Finder `Open` once, or run `xattr -dr com.apple.quarantine "/Applications/MCP Manager.app"` and open it again.',
  ].join('\n')
}

export function normalizeArch(archToken) {
  if (!archToken) {
    return null
  }

  const normalized = archToken.toLowerCase()

  if (['x64', 'amd64', 'x86_64'].includes(normalized)) {
    return 'x86_64'
  }

  if (['arm64', 'aarch64'].includes(normalized)) {
    return 'aarch64'
  }

  if (['armv7', 'armhf', 'armhfp', 'arm'].includes(normalized)) {
    return 'armv7'
  }

  if (['x86', 'i386', 'i686'].includes(normalized)) {
    return 'i686'
  }

  return normalized
}

function extractArch(assetName) {
  const match = assetName.match(/(?:_|-)(x64|x86_64|amd64|arm64|aarch64|armv7|armhf|armhfp|arm|x86|i386|i686)(?:[_.-]|$)/i)
  return normalizeArch(match?.[1] ?? null)
}

function parseSignatureAsset(assetName) {
  for (const definition of installerDefinitions) {
    if (assetName.endsWith(definition.suffix)) {
      const arch = extractArch(assetName)

      if (!arch) {
        return null
      }

      return {
        os: definition.os,
        installer: definition.installer,
        arch,
        assetName: assetName.slice(0, -'.sig'.length),
        signatureAssetName: assetName,
      }
    }
  }

  return null
}

export function collectUpdaterEntries(assets) {
  const assetsByName = new Map(assets.map((asset) => [asset.name, asset]))
  const entries = []

  for (const asset of assets) {
    if (!asset.name.endsWith('.sig')) {
      continue
    }

    const parsed = parseSignatureAsset(asset.name)
    if (!parsed) {
      continue
    }

    const bundleAsset = assetsByName.get(parsed.assetName)
    if (!bundleAsset || typeof asset.signature !== 'string' || asset.signature.length === 0) {
      continue
    }

    entries.push({
      ...parsed,
      url: bundleAsset.browser_download_url,
      signature: asset.signature,
    })
  }

  return entries.sort((left, right) => {
    return `${left.os}-${left.arch}-${left.installer}`.localeCompare(
      `${right.os}-${right.arch}-${right.installer}`,
    )
  })
}

function choosePrimaryEntry(entries) {
  return [...entries].sort((left, right) => {
    const leftPriority = primaryInstallerPriority[left.os]?.[left.installer] ?? 0
    const rightPriority = primaryInstallerPriority[right.os]?.[right.installer] ?? 0

    if (leftPriority !== rightPriority) {
      return rightPriority - leftPriority
    }

    return left.installer.localeCompare(right.installer)
  })[0]
}

export function buildLatestManifest({ version, notes, pubDate, entries }) {
  if (!entries.length) {
    throw new Error('No updater entries available for latest.json')
  }

  const platforms = {}
  const groupedEntries = new Map()

  for (const entry of entries) {
    const groupKey = `${entry.os}-${entry.arch}`

    if (!groupedEntries.has(groupKey)) {
      groupedEntries.set(groupKey, [])
    }

    groupedEntries.get(groupKey).push(entry)
    platforms[`${groupKey}-${entry.installer}`] = {
      signature: entry.signature,
      url: entry.url,
    }
  }

  for (const [groupKey, groupEntries] of groupedEntries.entries()) {
    const primaryEntry = choosePrimaryEntry(groupEntries)

    platforms[groupKey] = {
      signature: primaryEntry.signature,
      url: primaryEntry.url,
    }
  }

  return {
    version,
    notes,
    pub_date: pubDate,
    platforms,
  }
}

async function ensureRelease(tagName) {
  const { owner, repo } = getRepositoryContext()
  const releases = await listReleases(owner, repo)
  const existingRelease = releases.find((release) => release.tag_name === tagName)

  if (existingRelease) {
    console.log(
      `Using existing ${existingRelease.draft ? 'draft' : 'published'} release for ${tagName}: ${existingRelease.html_url}`,
    )
    writeGithubOutputs({
      release_id: String(existingRelease.id),
      release_url: existingRelease.html_url,
      release_draft: String(existingRelease.draft),
    })
    return
  }

  const release = await githubJson(`/repos/${owner}/${repo}/releases`, {
    method: 'POST',
    body: {
      tag_name: tagName,
      name: `MCP Manager ${tagName}`,
      body: buildDefaultReleaseBody(tagName),
      draft: true,
      prerelease: isPrereleaseRef(tagName),
    },
  })

  console.log(`Created draft release for ${tagName}: ${release.html_url}`)
  writeGithubOutputs({
    release_id: String(release.id),
    release_url: release.html_url,
    release_draft: String(release.draft),
  })
}

async function deleteReleaseAsset(owner, repo, assetId) {
  await githubJson(`/repos/${owner}/${repo}/releases/assets/${assetId}`, {
    method: 'DELETE',
  })
}

async function uploadReleaseAsset(release, name, contents, contentType) {
  const uploadUrl = new URL(release.upload_url.replace(/\{.*$/, ''), uploadsBaseUrl)
  uploadUrl.searchParams.set('name', name)
  uploadUrl.searchParams.set('label', name)

  const response = await fetch(uploadUrl, {
    method: 'POST',
    headers: githubHeaders({
      Accept: 'application/vnd.github+json',
      'Content-Type': contentType,
      'Content-Length': String(Buffer.byteLength(contents)),
    }),
    body: contents,
  })

  if (!response.ok) {
    throw new Error(`GitHub upload failed: ${response.status} ${await response.text()}`)
  }
}

async function uploadUpdaterManifest(releaseId) {
  const { owner, repo } = getRepositoryContext()
  const release = await githubJson(`/repos/${owner}/${repo}/releases/${releaseId}`)
  const assets = await githubJson(`/repos/${owner}/${repo}/releases/${releaseId}/assets?per_page=100`)
  const hydratedAssets = await Promise.all(
    assets.map(async (asset) => {
      if (!asset.name.endsWith('.sig')) {
        return asset
      }

      return {
        ...asset,
        signature: await githubText(asset.url),
      }
    }),
  )

  const entries = collectUpdaterEntries(hydratedAssets)
  const version = release.tag_name.replace(/^v/, '')
  const notes = release.body ?? ''
  const pubDate = release.published_at ?? new Date().toISOString()
  const manifest = buildLatestManifest({
    version,
    notes,
    pubDate,
    entries,
  })
  const latestJson = `${JSON.stringify(manifest, null, 2)}\n`
  const existingAsset = assets.find((asset) => asset.name === 'latest.json')

  if (existingAsset) {
    console.log('Deleting existing latest.json...')
    await deleteReleaseAsset(owner, repo, existingAsset.id)
  }

  console.log('Uploading latest.json...')
  await uploadReleaseAsset(release, 'latest.json', latestJson, 'application/json')
}

function printUsage() {
  console.error('Usage:')
  console.error('  node scripts/github-release.mjs ensure <tag-name>')
  console.error('  node scripts/github-release.mjs upload-updater <release-id>')
}

async function main() {
  const [command, value] = process.argv.slice(2)

  try {
    if (command === 'ensure') {
      const tagName = value || process.env.GITHUB_REF_NAME

      if (!tagName) {
        throw new Error('Missing tag name for ensure command')
      }

      await ensureRelease(tagName)
      return
    }

    if (command === 'upload-updater') {
      const releaseId = Number(value || process.env.RELEASE_ID)

      if (!Number.isInteger(releaseId) || releaseId <= 0) {
        throw new Error('Missing valid release id for upload-updater command')
      }

      await uploadUpdaterManifest(releaseId)
      return
    }

    printUsage()
    process.exitCode = 1
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error))
    process.exitCode = 1
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main()
}

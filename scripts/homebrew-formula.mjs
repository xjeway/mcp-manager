import { cliArchiveName, cliTargets } from './cli-targets.mjs'

const formulaTargets = cliTargets.filter((entry) => entry.homebrew)

// Returns null when the release has no CLI archives at all (releases before
// the CLI shipped), and throws when only some of them are there.
export function collectCliFormulaAssets(assets) {
  const byName = new Map(assets.map((asset) => [asset.name, asset]))
  const found = formulaTargets.map((entry) => [entry, byName.get(cliArchiveName(entry.target))])

  if (found.every(([, asset]) => !asset)) {
    return null
  }

  const missing = found.filter(([, asset]) => !asset).map(([entry]) => cliArchiveName(entry.target))
  if (missing.length > 0) {
    throw new Error(`Missing CLI Homebrew assets: ${missing.join(', ')}`)
  }

  const selected = {}
  for (const [entry, asset] of found) {
    selected[entry.homebrew.os] ??= {}
    selected[entry.homebrew.os][entry.homebrew.arch] = asset
  }
  return selected
}

function renderArch(arch, asset, owner, repo) {
  return `    on_${arch} do
      url "https://github.com/${owner}/${repo}/releases/download/v#{version}/${asset.name}"
      sha256 "${asset.sha256}"
    end`
}

export function renderFormula({ assets, owner, repo, version }) {
  const platform = (os) =>
    `  on_${os} do
${renderArch('arm', assets[os].arm, owner, repo)}
${renderArch('intel', assets[os].intel, owner, repo)}
  end`

  return `class Mcpmgr < Formula
  desc "Add, list and remove MCP servers across Claude Code, Codex, Cursor and more"
  homepage "https://github.com/${owner}/${repo}"
  version "${version}"
  license "MIT"

  livecheck do
    url :stable
    strategy :github_latest
  end

${platform('macos')}

${platform('linux')}

  def install
    bin.install "mcpmgr"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/mcpmgr --version")
  end
end
`
}

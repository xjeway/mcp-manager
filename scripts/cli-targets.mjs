// Every platform the `mcpmgr` CLI ships for. release.yml builds each target,
// npm-packages.mjs turns them into npm packages and homebrew-formula.mjs into
// the tap formula, so the three stay in step through this one table.
export const cliTargets = Object.freeze([
  {
    target: 'aarch64-apple-darwin',
    npm: { os: 'darwin', cpu: 'arm64' },
    homebrew: { os: 'macos', arch: 'arm' },
  },
  {
    target: 'x86_64-apple-darwin',
    npm: { os: 'darwin', cpu: 'x64' },
    homebrew: { os: 'macos', arch: 'intel' },
  },
  {
    target: 'aarch64-unknown-linux-musl',
    npm: { os: 'linux', cpu: 'arm64' },
    homebrew: { os: 'linux', arch: 'arm' },
  },
  {
    target: 'x86_64-unknown-linux-musl',
    npm: { os: 'linux', cpu: 'x64' },
    homebrew: { os: 'linux', arch: 'intel' },
  },
  {
    target: 'aarch64-pc-windows-msvc',
    npm: { os: 'win32', cpu: 'arm64' },
    homebrew: null,
  },
  {
    target: 'x86_64-pc-windows-msvc',
    npm: { os: 'win32', cpu: 'x64' },
    homebrew: null,
  },
])

export const CLI_BIN_NAME = 'mcpmgr'

function isWindowsTarget(target) {
  return target.endsWith('-windows-msvc')
}

export function cliBinaryName(target) {
  return isWindowsTarget(target) ? `${CLI_BIN_NAME}.exe` : CLI_BIN_NAME
}

// Release asset name. The tag is already in the download URL, so the name
// stays stable across versions.
export function cliArchiveName(target) {
  return `${CLI_BIN_NAME}-${target}.${isWindowsTarget(target) ? 'zip' : 'tar.gz'}`
}

export function npmPlatformPackageName({ npm }) {
  return `@${CLI_BIN_NAME}/${npm.os}-${npm.cpu}`
}

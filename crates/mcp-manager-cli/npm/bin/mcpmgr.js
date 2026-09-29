#!/usr/bin/env node
'use strict'

// npm installs only the optional dependency that matches this machine; this
// launcher finds its native binary and hands the terminal over to it.
const { spawnSync } = require('node:child_process')

const platformPackages = {
  'darwin-arm64': '@mcpmgr/darwin-arm64',
  'darwin-x64': '@mcpmgr/darwin-x64',
  'linux-arm64': '@mcpmgr/linux-arm64',
  'linux-x64': '@mcpmgr/linux-x64',
  'win32-arm64': '@mcpmgr/win32-arm64',
  'win32-x64': '@mcpmgr/win32-x64',
}

function resolveBinary(platform = process.platform, arch = process.arch, resolve = require.resolve) {
  const key = `${platform}-${arch}`
  const packageName = platformPackages[key]
  if (!packageName) {
    throw new Error(`mcpmgr does not ship a binary for ${key}.`)
  }

  const binary = `${packageName}/bin/mcpmgr${platform === 'win32' ? '.exe' : ''}`
  try {
    return resolve(binary)
  } catch {
    throw new Error(
      `Could not find ${packageName}, which provides mcpmgr for ${key}. ` +
        'It is installed as an optional dependency, so reinstall without --no-optional or --omit=optional.',
    )
  }
}

function run() {
  let binary
  try {
    binary = resolveBinary()
  } catch (error) {
    console.error(error.message)
    process.exit(1)
  }

  const result = spawnSync(binary, process.argv.slice(2), { stdio: 'inherit' })
  if (result.error) {
    console.error(`Failed to start mcpmgr: ${result.error.message}`)
    process.exit(1)
  }
  if (result.signal) {
    process.kill(process.pid, result.signal)
    return
  }
  process.exit(result.status ?? 1)
}

module.exports = { platformPackages, resolveBinary }

if (require.main === module) {
  run()
}

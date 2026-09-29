import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { createRequire } from 'node:module'
import { afterEach, describe, expect, it } from 'vitest'

import { cliBinaryName, cliTargets, npmPlatformPackageName } from './cli-targets.mjs'
import {
  buildPackages,
  buildPlaceholders,
  npmDistTag,
  parseVersionFromTag,
  publishedVersion,
  renderMainPackageJson,
  renderPlatformPackageJson,
} from './npm-packages.mjs'

const require = createRequire(import.meta.url)
const launcher = require('../crates/mcp-manager-cli/npm/bin/mcpmgr.js')

const tempDirs = []

function makeTempDir() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'mcpmgr-npm-'))
  tempDirs.push(dir)
  return dir
}

afterEach(() => {
  for (const dir of tempDirs.splice(0)) {
    fs.rmSync(dir, { recursive: true, force: true })
  }
})

function fakeBinaries(dir) {
  for (const entry of cliTargets) {
    const binaryDir = path.join(dir, `mcpmgr-${entry.target}`)
    fs.mkdirSync(binaryDir, { recursive: true })
    fs.writeFileSync(path.join(binaryDir, cliBinaryName(entry.target)), entry.target)
  }
}

describe('npm package versions', () => {
  it('takes the version from the release tag', () => {
    expect(parseVersionFromTag('v0.1.8')).toBe('0.1.8')
    expect(parseVersionFromTag('v0.2.0-rc.1')).toBe('0.2.0-rc.1')
    expect(() => parseVersionFromTag('main')).toThrow(/semver tag/)
  })

  it('keeps prereleases off the latest dist-tag', () => {
    expect(npmDistTag('0.1.8')).toBe('latest')
    expect(npmDistTag('0.2.0-rc.1')).toBe('next')
  })
})

describe('npm package manifests', () => {
  it('restricts each platform package to its os and cpu', () => {
    const entry = cliTargets.find((item) => item.target === 'aarch64-apple-darwin')

    expect(renderPlatformPackageJson(entry, '0.1.8')).toEqual(
      expect.objectContaining({
        name: '@mcpmgr/darwin-arm64',
        version: '0.1.8',
        os: ['darwin'],
        cpu: ['arm64'],
      }),
    )
  })

  it('pins every platform package as an optional dependency of the same version', () => {
    const manifest = renderMainPackageJson('0.1.8')

    expect(manifest.name).toBe('@mcpmgr/cli')
    expect(manifest.bin).toEqual({ mcpmgr: 'bin/mcpmgr.js' })
    expect(manifest.optionalDependencies).toEqual(
      Object.fromEntries(cliTargets.map((entry) => [npmPlatformPackageName(entry), '0.1.8'])),
    )
    expect(manifest.repository.url).toBe('git+https://github.com/xjeway/mcp-manager.git')
  })
})

describe('npm launcher', () => {
  it('knows exactly the platforms the release builds', () => {
    expect(launcher.platformPackages).toEqual(
      Object.fromEntries(
        cliTargets.map((entry) => [`${entry.npm.os}-${entry.npm.cpu}`, npmPlatformPackageName(entry)]),
      ),
    )
  })

  it('resolves the binary inside the matching platform package', () => {
    const resolve = (request) => `/node_modules/${request}`

    expect(launcher.resolveBinary('linux', 'x64', resolve)).toBe(
      '/node_modules/@mcpmgr/linux-x64/bin/mcpmgr',
    )
    expect(launcher.resolveBinary('win32', 'arm64', resolve)).toBe(
      '/node_modules/@mcpmgr/win32-arm64/bin/mcpmgr.exe',
    )
  })

  it('explains unsupported platforms and missing optional dependencies', () => {
    const missing = () => {
      throw new Error('not found')
    }

    expect(() => launcher.resolveBinary('freebsd', 'x64', missing)).toThrow(/freebsd-x64/)
    expect(() => launcher.resolveBinary('linux', 'arm64', missing)).toThrow(/--omit=optional/)
  })
})

describe('building npm packages', () => {
  it('lays out one package per platform plus the launcher package', () => {
    const binariesDir = makeTempDir()
    const outDir = path.join(makeTempDir(), 'npm-dist')
    fakeBinaries(binariesDir)

    const dirs = buildPackages({ version: '0.1.8', binariesDir, outDir })

    expect(dirs.map((dir) => path.basename(dir))).toEqual([
      ...cliTargets.map((entry) => entry.target),
      'cli',
    ])
    for (const entry of cliTargets) {
      const binary = path.join(outDir, entry.target, 'bin', cliBinaryName(entry.target))
      expect(fs.readFileSync(binary, 'utf8')).toBe(entry.target)
      if (process.platform !== 'win32') {
        expect(fs.statSync(binary).mode & 0o111).not.toBe(0)
      }
    }
    expect(fs.existsSync(path.join(outDir, 'cli', 'bin', 'mcpmgr.js'))).toBe(true)
    expect(fs.existsSync(path.join(outDir, 'cli', 'README.md'))).toBe(true)
  })

  it('refuses to build when a platform binary is missing', () => {
    const binariesDir = makeTempDir()
    fakeBinaries(binariesDir)
    fs.rmSync(path.join(binariesDir, 'mcpmgr-x86_64-pc-windows-msvc'), { recursive: true })

    expect(() =>
      buildPackages({ version: '0.1.8', binariesDir, outDir: path.join(makeTempDir(), 'out') }),
    ).toThrow(/Missing x86_64-pc-windows-msvc binary/)
  })

  it('writes empty 0.0.0 placeholders to claim every package name', () => {
    const outDir = path.join(makeTempDir(), 'placeholders')

    const dirs = buildPlaceholders({ outDir })
    const names = dirs.map((dir) => JSON.parse(fs.readFileSync(path.join(dir, 'package.json'), 'utf8')))

    expect(names.map((manifest) => manifest.name)).toEqual([
      ...cliTargets.map((entry) => npmPlatformPackageName(entry)),
      '@mcpmgr/cli',
    ])
    expect(names.every((manifest) => manifest.version === '0.0.0' && !manifest.bin)).toBe(true)
  })

  it('leaves out names that are already claimed', () => {
    const outDir = path.join(makeTempDir(), 'placeholders')

    const dirs = buildPlaceholders({ outDir, skip: (name) => name !== '@mcpmgr/cli' })

    expect(dirs.map((dir) => path.basename(dir))).toEqual(['cli'])
  })
})

describe('checking npm', () => {
  const failing = (stderr) => () => {
    throw Object.assign(new Error('Command failed: npm view'), { stderr })
  }

  it('returns the published version', () => {
    expect(publishedVersion('@mcpmgr/cli', () => '0.0.0\n')).toBe('0.0.0')
  })

  it('treats only a confirmed 404 as not published', () => {
    expect(publishedVersion('@mcpmgr/cli', failing('npm error code E404\nnpm error 404 Not Found'))).toBe('')
  })

  it('refuses to guess when npm cannot be reached', () => {
    expect(() => publishedVersion('@mcpmgr/cli', failing('npm error code ENOTFOUND'))).toThrow(
      /Could not check @mcpmgr\/cli on npm: npm error code ENOTFOUND/,
    )
    expect(() => publishedVersion('@mcpmgr/cli', failing('npm error code E401'))).toThrow(/E401/)
  })
})

import { describe, expect, it } from 'vitest'

import { collectCliFormulaAssets, renderFormula } from './homebrew-formula.mjs'

const cliAssets = [
  'mcpmgr-aarch64-apple-darwin.tar.gz',
  'mcpmgr-x86_64-apple-darwin.tar.gz',
  'mcpmgr-aarch64-unknown-linux-musl.tar.gz',
  'mcpmgr-x86_64-unknown-linux-musl.tar.gz',
  'mcpmgr-x86_64-pc-windows-msvc.zip',
].map((name) => ({ name }))

describe('homebrew formula helpers', () => {
  it('picks the macOS and Linux archives by architecture', () => {
    const selected = collectCliFormulaAssets([{ name: 'MCP.Manager_x64.dmg' }, ...cliAssets])

    expect(selected.macos.arm.name).toBe('mcpmgr-aarch64-apple-darwin.tar.gz')
    expect(selected.macos.intel.name).toBe('mcpmgr-x86_64-apple-darwin.tar.gz')
    expect(selected.linux.arm.name).toBe('mcpmgr-aarch64-unknown-linux-musl.tar.gz')
    expect(selected.linux.intel.name).toBe('mcpmgr-x86_64-unknown-linux-musl.tar.gz')
  })

  it('skips releases that predate the CLI', () => {
    expect(collectCliFormulaAssets([{ name: 'MCP.Manager_aarch64.app.tar.gz' }])).toBeNull()
  })

  it('fails when only some CLI archives were uploaded', () => {
    expect(() => collectCliFormulaAssets(cliAssets.slice(1))).toThrow(
      /Missing CLI Homebrew assets: mcpmgr-aarch64-apple-darwin.tar.gz/,
    )
  })

  it('renders a formula with per-platform URLs and checksums', () => {
    const asset = (name, char) => ({ name, sha256: char.repeat(64) })
    const rendered = renderFormula({
      version: '0.1.8',
      owner: 'xjeway',
      repo: 'mcp-manager',
      assets: {
        macos: {
          arm: asset('mcpmgr-aarch64-apple-darwin.tar.gz', 'a'),
          intel: asset('mcpmgr-x86_64-apple-darwin.tar.gz', 'b'),
        },
        linux: {
          arm: asset('mcpmgr-aarch64-unknown-linux-musl.tar.gz', 'c'),
          intel: asset('mcpmgr-x86_64-unknown-linux-musl.tar.gz', 'd'),
        },
      },
    })

    expect(rendered).toContain('class Mcpmgr < Formula')
    expect(rendered).toContain('version "0.1.8"')
    expect(rendered).toContain(
      'url "https://github.com/xjeway/mcp-manager/releases/download/v#{version}/mcpmgr-aarch64-apple-darwin.tar.gz"',
    )
    for (const char of ['a', 'b', 'c', 'd']) {
      expect(rendered).toContain(`sha256 "${char.repeat(64)}"`)
    }
    expect(rendered).toContain('bin.install "mcpmgr"')
  })
})

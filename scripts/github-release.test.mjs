import { describe, expect, it } from 'vitest'

async function loadReleaseHelpers() {
  try {
    return await import('./github-release.mjs')
  } catch {
    return null
  }
}

const sampleAssets = [
  {
    name: 'MCP.Manager_x64.app.tar.gz',
    browser_download_url: 'https://example.invalid/MCP.Manager_x64.app.tar.gz',
  },
  {
    name: 'MCP.Manager_x64.app.tar.gz.sig',
    browser_download_url: 'https://example.invalid/MCP.Manager_x64.app.tar.gz.sig',
    signature: 'sig-darwin-x64',
  },
  {
    name: 'MCP.Manager_0.1.2_x64.AppImage',
    browser_download_url: 'https://example.invalid/MCP.Manager_0.1.2_x64.AppImage',
  },
  {
    name: 'MCP.Manager_0.1.2_x64.AppImage.sig',
    browser_download_url: 'https://example.invalid/MCP.Manager_0.1.2_x64.AppImage.sig',
    signature: 'sig-linux-x64-appimage',
  },
  {
    name: 'MCP.Manager_0.1.2_x64.deb',
    browser_download_url: 'https://example.invalid/MCP.Manager_0.1.2_x64.deb',
  },
  {
    name: 'MCP.Manager_0.1.2_x64.deb.sig',
    browser_download_url: 'https://example.invalid/MCP.Manager_0.1.2_x64.deb.sig',
    signature: 'sig-linux-x64-deb',
  },
  {
    name: 'MCP.Manager_0.1.2_x64-setup.exe',
    browser_download_url: 'https://example.invalid/MCP.Manager_0.1.2_x64-setup.exe',
  },
  {
    name: 'MCP.Manager_0.1.2_x64-setup.exe.sig',
    browser_download_url: 'https://example.invalid/MCP.Manager_0.1.2_x64-setup.exe.sig',
    signature: 'sig-windows-x64-nsis',
  },
  {
    name: 'MCP.Manager_0.1.2_x64_en-US.msi',
    browser_download_url: 'https://example.invalid/MCP.Manager_0.1.2_x64_en-US.msi',
  },
  {
    name: 'MCP.Manager_0.1.2_x64_en-US.msi.sig',
    browser_download_url: 'https://example.invalid/MCP.Manager_0.1.2_x64_en-US.msi.sig',
    signature: 'sig-windows-x64-msi',
  },
  {
    name: 'MCP.Manager_0.1.2_arm64-setup.exe',
    browser_download_url: 'https://example.invalid/MCP.Manager_0.1.2_arm64-setup.exe',
  },
  {
    name: 'MCP.Manager_0.1.2_arm64-setup.exe.sig',
    browser_download_url: 'https://example.invalid/MCP.Manager_0.1.2_arm64-setup.exe.sig',
    signature: 'sig-windows-arm64-nsis',
  },
]

describe('github release updater manifest', () => {
  it('classifies updater entries across supported platforms', async () => {
    const helpers = await loadReleaseHelpers()

    expect(helpers).not.toBeNull()

    const entries = helpers.collectUpdaterEntries(sampleAssets)

    expect(entries).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          assetName: 'MCP.Manager_x64.app.tar.gz',
          os: 'darwin',
          arch: 'x86_64',
          installer: 'app',
        }),
        expect.objectContaining({
          assetName: 'MCP.Manager_0.1.2_x64.AppImage',
          os: 'linux',
          arch: 'x86_64',
          installer: 'appimage',
        }),
        expect.objectContaining({
          assetName: 'MCP.Manager_0.1.2_x64.deb',
          os: 'linux',
          arch: 'x86_64',
          installer: 'deb',
        }),
        expect.objectContaining({
          assetName: 'MCP.Manager_0.1.2_x64_en-US.msi',
          os: 'windows',
          arch: 'x86_64',
          installer: 'msi',
        }),
        expect.objectContaining({
          assetName: 'MCP.Manager_0.1.2_arm64-setup.exe',
          os: 'windows',
          arch: 'aarch64',
          installer: 'nsis',
        }),
      ]),
    )
  })

  it('builds latest.json with primary and installer-specific keys', async () => {
    const helpers = await loadReleaseHelpers()

    expect(helpers).not.toBeNull()

    const entries = helpers.collectUpdaterEntries(sampleAssets)
    const manifest = helpers.buildLatestManifest({
      version: '0.1.2',
      notes: 'Release notes',
      pubDate: '2026-03-27T00:00:00.000Z',
      entries,
    })

    expect(manifest).toEqual(
      expect.objectContaining({
        version: '0.1.2',
        notes: 'Release notes',
        pub_date: '2026-03-27T00:00:00.000Z',
      }),
    )
    expect(manifest.platforms['darwin-x86_64']).toEqual({
      signature: 'sig-darwin-x64',
      url: 'https://example.invalid/MCP.Manager_x64.app.tar.gz',
    })
    expect(manifest.platforms['linux-x86_64']).toEqual({
      signature: 'sig-linux-x64-appimage',
      url: 'https://example.invalid/MCP.Manager_0.1.2_x64.AppImage',
    })
    expect(manifest.platforms['linux-x86_64-deb']).toEqual({
      signature: 'sig-linux-x64-deb',
      url: 'https://example.invalid/MCP.Manager_0.1.2_x64.deb',
    })
    expect(manifest.platforms['windows-x86_64']).toEqual({
      signature: 'sig-windows-x64-msi',
      url: 'https://example.invalid/MCP.Manager_0.1.2_x64_en-US.msi',
    })
    expect(manifest.platforms['windows-x86_64-nsis']).toEqual({
      signature: 'sig-windows-x64-nsis',
      url: 'https://example.invalid/MCP.Manager_0.1.2_x64-setup.exe',
    })
    expect(manifest.platforms['windows-aarch64']).toEqual({
      signature: 'sig-windows-arm64-nsis',
      url: 'https://example.invalid/MCP.Manager_0.1.2_arm64-setup.exe',
    })
  })
})

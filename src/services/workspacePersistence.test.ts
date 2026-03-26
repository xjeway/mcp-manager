import { describe, expect, it, vi } from 'vitest'
import { persistImportedConfig, saveAndSyncConfig } from './workspacePersistence'

const previousConfig = {
  version: 1,
  servers: [
    {
      id: 'server-1',
      name: 'Server 1',
      enabled: true,
      transport: { type: 'stdio' as const },
      command: { program: 'npx', args: ['old'], env: {} },
      apps: {
        vscode: true,
        cursor: false,
        claudeCode: false,
        claudeDesktop: false,
        codex: false,
        openCode: false,
        githubCopilot: false,
        geminiCli: false,
        antigravity: false,
        iFlow: false,
        qwenCode: false,
        cline: false,
        windsurf: false,
        kiro: false,
      },
    },
  ],
}

const nextConfig = {
  version: 1,
  servers: [
    {
      ...previousConfig.servers[0],
      command: { program: 'npx', args: ['new'], env: {} },
    },
  ],
}

describe('workspacePersistence', () => {
  it('saves first, then applies, and returns backups', async () => {
    const saveConfig = vi.fn().mockResolvedValue(undefined)
    const applyConfig = vi.fn().mockResolvedValue({ backups: ['backup-1'] })

    await expect(
      saveAndSyncConfig({
        previousConfig,
        nextConfig,
        saveConfig,
        applyConfig,
      }),
    ).resolves.toEqual({ backups: ['backup-1'] })

    expect(saveConfig).toHaveBeenCalledTimes(1)
    expect(saveConfig).toHaveBeenNthCalledWith(1, nextConfig)
    expect(applyConfig).toHaveBeenCalledTimes(1)
    expect(applyConfig).toHaveBeenNthCalledWith(1, nextConfig)
  })

  it('restores the previous config when apply fails after saving', async () => {
    const saveConfig = vi.fn().mockResolvedValue(undefined)
    const applyConfig = vi.fn().mockRejectedValue(new Error('apply failed'))

    await expect(
      saveAndSyncConfig({
        previousConfig,
        nextConfig,
        saveConfig,
        applyConfig,
      }),
    ).rejects.toThrow('apply failed')

    expect(saveConfig).toHaveBeenCalledTimes(2)
    expect(saveConfig).toHaveBeenNthCalledWith(1, nextConfig)
    expect(saveConfig).toHaveBeenNthCalledWith(2, previousConfig)
  })

  it('persists imported config without applying it again', async () => {
    const saveConfig = vi.fn().mockResolvedValue(undefined)

    await expect(persistImportedConfig(nextConfig, saveConfig)).resolves.toBeUndefined()

    expect(saveConfig).toHaveBeenCalledTimes(1)
    expect(saveConfig).toHaveBeenCalledWith(nextConfig)
  })
})

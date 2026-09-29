import { beforeEach, describe, expect, it, vi } from 'vitest'

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
  isTauri: () => true,
}))

const EMPTY = { version: 1, servers: [] }

// The module keeps the last-seen fingerprint, so each test gets a fresh copy.
async function freshService() {
  vi.resetModules()
  return import('./configService')
}

describe('configService fingerprint checks', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('saves against the fingerprint it loaded, then against the one it saved', async () => {
    const { loadConfig, saveConfig } = await freshService()
    invokeMock.mockResolvedValueOnce({ content: 'version: 1\nservers: []\n', fingerprint: 'f1' })
    await loadConfig()

    invokeMock.mockResolvedValueOnce('f2')
    await saveConfig(EMPTY)
    invokeMock.mockResolvedValueOnce('f3')
    await saveConfig(EMPTY)

    expect(invokeMock.mock.calls[1]).toEqual([
      'save_yaml_config',
      expect.objectContaining({ expectedFingerprint: 'f1' }),
    ])
    expect(invokeMock.mock.calls[2][1]).toMatchObject({ expectedFingerprint: 'f2' })
  })

  it('turns a backend conflict into ConfigConflictError', async () => {
    const { ConfigConflictError, loadConfig, saveConfig } = await freshService()
    invokeMock.mockResolvedValueOnce({ content: 'version: 1\nservers: []\n', fingerprint: 'f1' })
    await loadConfig()

    invokeMock.mockRejectedValueOnce('CONFIG_CONFLICT: servers.yaml was changed by another program')
    await expect(saveConfig(EMPTY)).rejects.toBeInstanceOf(ConfigConflictError)

    invokeMock.mockRejectedValueOnce('disk full')
    await expect(saveConfig(EMPTY)).rejects.toBe('disk full')
  })

  it('turns a newer-format refusal into ConfigTooNewError', async () => {
    const { ConfigTooNewError, saveConfig } = await freshService()

    invokeMock.mockRejectedValueOnce('CONFIG_TOO_NEW: servers.yaml was saved by a newer version')
    await expect(saveConfig(EMPTY)).rejects.toBeInstanceOf(ConfigTooNewError)
  })

  it('reports an outside change only when the fingerprint moved', async () => {
    const { hasExternalConfigChange, loadConfig } = await freshService()
    expect(await hasExternalConfigChange()).toBe(false)
    expect(invokeMock).not.toHaveBeenCalled()

    invokeMock.mockResolvedValueOnce({ content: 'version: 1\nservers: []\n', fingerprint: 'f1' })
    await loadConfig()

    invokeMock.mockResolvedValueOnce('f1')
    expect(await hasExternalConfigChange()).toBe(false)
    invokeMock.mockResolvedValueOnce('f9')
    expect(await hasExternalConfigChange()).toBe(true)
  })
})

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

  it('applies against the fingerprint it just saved, and reports a conflict', async () => {
    const { ConfigConflictError, applyConfig, saveConfig } = await freshService()
    invokeMock.mockResolvedValueOnce('f2')
    await saveConfig(EMPTY)

    invokeMock.mockResolvedValueOnce({ backups: [] })
    await applyConfig(EMPTY, EMPTY)
    expect(invokeMock.mock.calls[1]).toEqual([
      'apply_config',
      expect.objectContaining({ expectedFingerprint: 'f2' }),
    ])

    invokeMock.mockRejectedValueOnce('CONFIG_CONFLICT: servers.yaml was changed by another program')
    await expect(applyConfig(EMPTY, EMPTY)).rejects.toBeInstanceOf(ConfigConflictError)
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

const PATH = '/data/mcp-manager/config/servers.yaml'

// Answers the backend commands `loadConfig` and friends call, reading the
// file's state from `disk` at call time.
function fakeBackend(disk: { content: string; fingerprint: string } | Error) {
  invokeMock.mockImplementation(async (command: string, args: Record<string, unknown>) => {
    switch (command) {
      case 'load_yaml_config':
        if (disk instanceof Error) {
          throw disk.message
        }
        return disk
      case 'yaml_config_path':
        return PATH
      case 'save_yaml_config':
        return `saved-over-${String(args.expectedFingerprint)}`
      case 'reset_yaml_config':
        return { backupPath: `${PATH}.broken-20260929-104740`, fingerprint: 'fresh' }
      default:
        throw new Error(`unexpected command ${command}`)
    }
  })
}

const commandsCalled = () => invokeMock.mock.calls.map(([command]) => command)

describe('configService with an unreadable servers.yaml', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it.each([
    ['is not valid YAML', 'servers: [oops\n'],
    ['has no version', 'servers: []\n'],
    ['has no server list', 'version: 1\nservers: nope\n'],
    ['is a bare list', '- a\n- b\n'],
  ])('refuses to load or save when the file %s', async (_label, content) => {
    const { ConfigUnreadableError, loadConfig, saveConfig } = await freshService()
    fakeBackend({ content, fingerprint: 'broken' })

    const loaded = await loadConfig().catch((error: unknown) => error)
    expect(loaded).toBeInstanceOf(ConfigUnreadableError)
    expect(loaded).toMatchObject({ path: PATH })

    await expect(saveConfig(EMPTY)).rejects.toBeInstanceOf(ConfigUnreadableError)
    expect(commandsCalled()).not.toContain('save_yaml_config')
  })

  it('starts empty when the file does not exist yet, or is empty', async () => {
    const { loadConfig, saveConfig } = await freshService()
    fakeBackend({ content: 'version: 1\nservers: []\n', fingerprint: 'absent' })
    expect(await loadConfig()).toEqual(EMPTY)
    await saveConfig(EMPTY)
    expect(invokeMock).toHaveBeenLastCalledWith(
      'save_yaml_config',
      expect.objectContaining({ expectedFingerprint: 'absent' }),
    )

    fakeBackend({ content: '  \n', fingerprint: 'blank' })
    expect(await loadConfig()).toEqual(EMPTY)
  })

  it('allows saving again once the file is fixed', async () => {
    const { hasExternalConfigChange, loadConfig, saveConfig } = await freshService()
    const disk = { content: 'servers: [oops\n', fingerprint: 'broken' }
    fakeBackend(disk)
    await expect(loadConfig()).rejects.toThrow(PATH)

    invokeMock.mockResolvedValueOnce('broken')
    expect(await hasExternalConfigChange()).toBe(false)
    invokeMock.mockResolvedValueOnce('fixed')
    expect(await hasExternalConfigChange()).toBe(true)

    Object.assign(disk, { content: 'version: 1\nservers: []\n', fingerprint: 'fixed' })
    expect(await loadConfig()).toEqual(EMPTY)
    await saveConfig(EMPTY)
    expect(invokeMock).toHaveBeenLastCalledWith(
      'save_yaml_config',
      expect.objectContaining({ expectedFingerprint: 'fixed' }),
    )
  })

  it('blocks saves when a file that loaded fine breaks later', async () => {
    const { ConfigUnreadableError, loadConfig, saveConfig } = await freshService()
    const disk = { content: 'version: 1\nservers: []\n', fingerprint: 'f1' }
    fakeBackend(disk)
    await loadConfig()

    Object.assign(disk, { content: 'version: 1\nservers: [\n', fingerprint: 'broken' })
    await expect(loadConfig()).rejects.toBeInstanceOf(ConfigUnreadableError)
    await expect(saveConfig(EMPTY)).rejects.toBeInstanceOf(ConfigUnreadableError)
    expect(commandsCalled()).not.toContain('save_yaml_config')
  })

  it('treats a file it cannot read at all as unreadable, and keeps checking it', async () => {
    const { ConfigUnreadableError, hasExternalConfigChange, loadConfig, saveConfig } = await freshService()
    fakeBackend(new Error('permission denied'))

    await expect(loadConfig()).rejects.toMatchObject({ detail: 'permission denied' })
    await expect(saveConfig(EMPTY)).rejects.toBeInstanceOf(ConfigUnreadableError)
    expect(await hasExternalConfigChange()).toBe(true)
  })

  it('starts over by backing up the broken file, then saves normally', async () => {
    const { loadConfig, saveConfig, startOverConfig } = await freshService()
    fakeBackend({ content: 'servers: [oops\n', fingerprint: 'broken' })
    await expect(loadConfig()).rejects.toThrow()

    const result = await startOverConfig()
    expect(result).toEqual({ config: EMPTY, backupPath: `${PATH}.broken-20260929-104740` })
    expect(invokeMock).toHaveBeenLastCalledWith(
      'reset_yaml_config',
      expect.objectContaining({ expectedFingerprint: 'broken' }),
    )

    await saveConfig(EMPTY)
    expect(invokeMock).toHaveBeenLastCalledWith(
      'save_yaml_config',
      expect.objectContaining({ expectedFingerprint: 'fresh' }),
    )
  })

  it('can start over from the list it last loaded', async () => {
    const { loadConfig, startOverConfig } = await freshService()
    fakeBackend({ content: 'servers: [oops\n', fingerprint: 'broken' })
    await expect(loadConfig()).rejects.toThrow()

    const kept = { version: 1, servers: [{ id: 'a' }] } as unknown as typeof EMPTY
    expect((await startOverConfig(kept)).config).toBe(kept)
    expect(invokeMock).toHaveBeenLastCalledWith(
      'reset_yaml_config',
      expect.objectContaining({ content: 'version: 1\nservers:\n  - id: a\n' }),
    )
  })

  it('does not start over a file that changed since it was read', async () => {
    const { ConfigConflictError, ConfigUnreadableError, loadConfig, saveConfig, startOverConfig } =
      await freshService()
    fakeBackend({ content: 'servers: [oops\n', fingerprint: 'broken' })
    await expect(loadConfig()).rejects.toThrow()

    invokeMock.mockRejectedValueOnce('CONFIG_CONFLICT: servers.yaml was changed by another program')
    await expect(startOverConfig()).rejects.toBeInstanceOf(ConfigConflictError)
    await expect(saveConfig(EMPTY)).rejects.toBeInstanceOf(ConfigUnreadableError)
  })
})

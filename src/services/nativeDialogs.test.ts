import { beforeEach, describe, expect, it, vi } from 'vitest'
import { confirmDialog } from './nativeDialogs'

const { invokeMock, isTauriMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  isTauriMock: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
  isTauri: isTauriMock,
}))

describe('nativeDialogs', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    isTauriMock.mockReset()
    vi.unstubAllGlobals()
  })

  it('uses the Tauri dialog command when running in Tauri', async () => {
    isTauriMock.mockReturnValue(true)
    invokeMock.mockResolvedValue('Delete')
    vi.stubGlobal('window', {
      confirm: vi.fn(),
    })

    await expect(
      confirmDialog({
        cancelLabel: 'Cancel',
        kind: 'warning',
        message: 'Delete server?',
        okLabel: 'Delete',
        title: 'Delete',
      }),
    ).resolves.toBe(true)

    expect(invokeMock).toHaveBeenCalledTimes(1)
    expect(invokeMock).toHaveBeenCalledWith('plugin:dialog|message', {
      buttons: { OkCancelCustom: ['Delete', 'Cancel'] },
      kind: 'warning',
      message: 'Delete server?',
      title: 'Delete',
    })
  })

  it('treats the cancel button as declining', async () => {
    isTauriMock.mockReturnValue(true)
    invokeMock.mockResolvedValue('Keep editing')
    vi.stubGlobal('window', { confirm: vi.fn() })

    await expect(
      confirmDialog({ cancelLabel: 'Keep editing', message: 'Discard?', okLabel: 'Discard' }),
    ).resolves.toBe(false)

    invokeMock.mockResolvedValue({ Custom: 'Discard' })
    await expect(
      confirmDialog({ cancelLabel: 'Keep editing', message: 'Discard?', okLabel: 'Discard' }),
    ).resolves.toBe(true)
  })

  it('falls back to window.confirm if the Tauri dialog command fails', async () => {
    isTauriMock.mockReturnValue(true)
    invokeMock.mockRejectedValue(new Error('dialog unavailable'))
    const confirmMock = vi.fn().mockReturnValue(true)
    vi.stubGlobal('window', {
      confirm: confirmMock,
    })

    await expect(
      confirmDialog({
        message: 'Discard this edit?',
      }),
    ).resolves.toBe(true)

    expect(confirmMock).toHaveBeenCalledTimes(1)
    expect(confirmMock).toHaveBeenCalledWith('Discard this edit?')
  })

  it('returns false if both the Tauri dialog and fallback confirm fail', async () => {
    isTauriMock.mockReturnValue(true)
    invokeMock.mockRejectedValue(new Error('dialog unavailable'))
    const confirmMock = vi.fn().mockRejectedValue(new Error('confirm unavailable'))
    vi.stubGlobal('window', {
      confirm: confirmMock,
    })

    await expect(
      confirmDialog({
        message: 'Discard this edit?',
      }),
    ).resolves.toBe(false)
  })
})

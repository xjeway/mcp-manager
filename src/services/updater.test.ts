import { beforeEach, describe, expect, it, vi } from 'vitest'
import { checkForUpdatesAndPrompt } from './updater'

const { checkMock, isDesktopRuntimeMock } = vi.hoisted(() => ({
  checkMock: vi.fn(),
  isDesktopRuntimeMock: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: checkMock,
}))

vi.mock('./runtime', () => ({
  isDesktopRuntime: isDesktopRuntimeMock,
}))

describe('updater', () => {
  beforeEach(() => {
    checkMock.mockReset()
    isDesktopRuntimeMock.mockReset()
    vi.unstubAllGlobals()
  })

  it('does not alert on startup when silent check fails', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    checkMock.mockRejectedValue(new Error('updater.check not allowed'))
    const alertMock = vi.fn()
    const confirmMock = vi.fn()
    vi.stubGlobal('window', {
      alert: alertMock,
      confirm: confirmMock,
    })

    await expect(checkForUpdatesAndPrompt({ silentIfNoUpdate: true })).resolves.toBeUndefined()

    expect(alertMock).not.toHaveBeenCalled()
    expect(confirmMock).not.toHaveBeenCalled()
  })
})

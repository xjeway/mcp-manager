import { beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n'
import { checkForUpdatesAndPrompt } from './updater'

const { checkMock, isDesktopRuntimeMock, openReleasesLinkMock } = vi.hoisted(() => ({
  checkMock: vi.fn(),
  isDesktopRuntimeMock: vi.fn(),
  openReleasesLinkMock: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: checkMock,
}))

vi.mock('./runtime', () => ({
  isDesktopRuntime: isDesktopRuntimeMock,
}))

vi.mock('./externalLinks', () => ({
  RELEASES_URL: 'https://github.com/xjeway/mcp-manager/releases',
  openReleasesLink: openReleasesLinkMock,
}))

describe('updater', () => {
  beforeEach(() => {
    checkMock.mockReset()
    isDesktopRuntimeMock.mockReset()
    openReleasesLinkMock.mockReset()
    vi.unstubAllGlobals()
    void i18n.changeLanguage('en-US')
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

  it('offers the releases page when the updater feed is missing', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    checkMock.mockRejectedValue(new Error('Could not fetch a valid release JSON from the remote'))
    openReleasesLinkMock.mockResolvedValue(undefined)
    const alertMock = vi.fn()
    const confirmMock = vi.fn(() => true)
    vi.stubGlobal('window', {
      alert: alertMock,
      confirm: confirmMock,
    })

    await expect(checkForUpdatesAndPrompt()).resolves.toBeUndefined()

    expect(confirmMock).toHaveBeenCalledWith(
      'Automatic updates are not available for this build yet. Open the Releases page instead?',
    )
    expect(openReleasesLinkMock).toHaveBeenCalledTimes(1)
    expect(alertMock).not.toHaveBeenCalled()
  })

  it('localizes the missing feed prompt in Chinese', async () => {
    await i18n.changeLanguage('zh-CN')
    isDesktopRuntimeMock.mockReturnValue(true)
    checkMock.mockRejectedValue(new Error('Could not fetch a valid release JSON from the remote'))
    const alertMock = vi.fn()
    const confirmMock = vi.fn(() => false)
    vi.stubGlobal('window', {
      alert: alertMock,
      confirm: confirmMock,
    })

    await expect(checkForUpdatesAndPrompt()).resolves.toBeUndefined()

    expect(confirmMock).toHaveBeenCalledWith('当前构建暂时不支持自动更新。要改为打开 Releases 发布页吗？')
    expect(alertMock).not.toHaveBeenCalled()
    expect(openReleasesLinkMock).not.toHaveBeenCalled()
  })

  it('shows the original error for other update failures', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    checkMock.mockRejectedValue(new Error('network timeout'))
    const alertMock = vi.fn()
    const confirmMock = vi.fn()
    vi.stubGlobal('window', {
      alert: alertMock,
      confirm: confirmMock,
    })

    await expect(checkForUpdatesAndPrompt()).resolves.toBeUndefined()

    expect(confirmMock).not.toHaveBeenCalled()
    expect(alertMock).toHaveBeenCalledWith('Update check failed: Error: network timeout')
  })
})

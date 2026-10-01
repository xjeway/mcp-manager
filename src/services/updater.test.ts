import { beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../i18n'

const {
  checkMock,
  confirmDialogMock,
  invokeMock,
  isDesktopRuntimeMock,
  openReleasesLinkMock,
  showUpdateProgressWindowMock,
} = vi.hoisted(() => ({
  checkMock: vi.fn(),
  confirmDialogMock: vi.fn(),
  invokeMock: vi.fn(),
  isDesktopRuntimeMock: vi.fn(),
  openReleasesLinkMock: vi.fn(),
  showUpdateProgressWindowMock: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: checkMock,
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

vi.mock('./nativeDialogs', () => ({
  confirmDialog: confirmDialogMock,
}))

vi.mock('./runtime', () => ({
  isDesktopRuntime: isDesktopRuntimeMock,
}))

vi.mock('./externalLinks', () => ({
  RELEASES_URL: 'https://github.com/xjeway/mcp-manager/releases',
  openReleasesLink: openReleasesLinkMock,
}))

vi.mock('./updateProgressWindow', () => ({
  showUpdateProgressWindow: showUpdateProgressWindowMock,
}))

async function loadUpdaterModule() {
  return await import('./updater')
}

function stubWindow(overrides?: Partial<Window>) {
  vi.stubGlobal('window', {
    alert: vi.fn(),
    confirm: vi.fn(),
    localStorage: {
      getItem: vi.fn(() => null),
      removeItem: vi.fn(),
      setItem: vi.fn(),
    },
    navigator: {
      language: 'en-US',
    },
    ...overrides,
  })
}

describe('updater', () => {
  beforeEach(() => {
    vi.resetModules()
    checkMock.mockReset()
    confirmDialogMock.mockReset()
    invokeMock.mockReset()
    isDesktopRuntimeMock.mockReset()
    openReleasesLinkMock.mockReset()
    showUpdateProgressWindowMock.mockReset()
    vi.unstubAllGlobals()
    void i18n.changeLanguage('en-US')
  })

  it('makes no update request when automatic checks are disabled', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    stubWindow({
      localStorage: {
        getItem: vi.fn((key: string) => (key === 'ui-automatic-update-checks' ? 'false' : null)),
        removeItem: vi.fn(),
        setItem: vi.fn(),
      } as unknown as Storage,
    })
    const { checkForUpdatesAndPrompt } = await loadUpdaterModule()

    await checkForUpdatesAndPrompt({ silentIfNoUpdate: true })
    expect(checkMock).not.toHaveBeenCalled()
    expect(invokeMock).not.toHaveBeenCalled()

    await checkForUpdatesAndPrompt()
    expect(checkMock).toHaveBeenCalledTimes(1)
  })

  it('does not alert on startup when silent check fails', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    checkMock.mockRejectedValue(new Error('updater.check not allowed'))
    const alertMock = vi.fn()
    stubWindow({ alert: alertMock as unknown as Window['alert'] })
    const { checkForUpdatesAndPrompt } = await loadUpdaterModule()

    await expect(checkForUpdatesAndPrompt({ silentIfNoUpdate: true })).resolves.toBeUndefined()

    expect(alertMock).not.toHaveBeenCalled()
    expect(confirmDialogMock).not.toHaveBeenCalled()
  })

  it('offers the releases page when the updater feed is missing', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    checkMock.mockRejectedValue(new Error('Could not fetch a valid release JSON from the remote'))
    openReleasesLinkMock.mockResolvedValue(undefined)
    confirmDialogMock.mockResolvedValue(true)
    const alertMock = vi.fn()
    stubWindow({ alert: alertMock as unknown as Window['alert'] })
    const { checkForUpdatesAndPrompt } = await loadUpdaterModule()

    await expect(checkForUpdatesAndPrompt()).resolves.toBeUndefined()

    expect(confirmDialogMock).toHaveBeenCalledWith({
      message: 'Automatic updates are not available for this build yet. Open the Releases page instead?',
    })
    expect(openReleasesLinkMock).toHaveBeenCalledTimes(1)
    expect(alertMock).not.toHaveBeenCalled()
  })

  it('localizes the missing feed prompt in Chinese', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    checkMock.mockRejectedValue(new Error('Could not fetch a valid release JSON from the remote'))
    confirmDialogMock.mockResolvedValue(false)
    const alertMock = vi.fn()
    stubWindow({
      alert: alertMock as unknown as Window['alert'],
      navigator: { language: 'zh-CN' } as Window['navigator'],
    })
    const { checkForUpdatesAndPrompt } = await loadUpdaterModule()

    await expect(checkForUpdatesAndPrompt()).resolves.toBeUndefined()

    expect(confirmDialogMock).toHaveBeenCalledWith({
      message: '当前构建暂时不支持自动更新。要改为打开 Releases 发布页吗？',
    })
    expect(alertMock).not.toHaveBeenCalled()
    expect(openReleasesLinkMock).not.toHaveBeenCalled()
  })

  it('downloads the update and leaves installation for an explicit later step', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    confirmDialogMock.mockResolvedValue(true)
    showUpdateProgressWindowMock.mockResolvedValue(undefined)
    const downloadMock = vi.fn(async (onEvent?: (event: unknown) => void) => {
      onEvent?.({ event: 'Started', data: { contentLength: 100 } })
      onEvent?.({ event: 'Progress', data: { chunkLength: 25 } })
      onEvent?.({ event: 'Progress', data: { chunkLength: 75 } })
      onEvent?.({ event: 'Finished' })
    })
    const installMock = vi.fn(async () => undefined)
    checkMock.mockResolvedValue({
      body: '',
      currentVersion: '0.1.5',
      date: '2026-04-03',
      download: downloadMock,
      install: installMock,
      version: '0.1.6',
    })
    stubWindow()
    const { checkForUpdatesAndPrompt, getUpdateState } = await loadUpdaterModule()

    await expect(checkForUpdatesAndPrompt()).resolves.toBeUndefined()

    expect(confirmDialogMock).toHaveBeenCalledWith({
      message: 'New version 0.1.6 available. Download now?',
    })
    expect(showUpdateProgressWindowMock).toHaveBeenCalledTimes(1)
    expect(downloadMock).toHaveBeenCalledTimes(1)
    expect(installMock).not.toHaveBeenCalled()
    expect(getUpdateState()).toMatchObject({
      currentVersion: '0.1.5',
      downloadedBytes: 100,
      phase: 'downloaded',
      targetVersion: '0.1.6',
      totalBytes: 100,
    })
  })

  it('installs and requests restart only after the explicit install action', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    confirmDialogMock.mockResolvedValue(true)
    showUpdateProgressWindowMock.mockResolvedValue(undefined)
    const downloadMock = vi.fn(async () => undefined)
    const installMock = vi.fn(async () => undefined)
    checkMock.mockResolvedValue({
      body: '',
      currentVersion: '0.1.5',
      date: '2026-04-03',
      download: downloadMock,
      install: installMock,
      version: '0.1.6',
    })
    stubWindow()
    const { checkForUpdatesAndPrompt, installDownloadedUpdate } = await loadUpdaterModule()

    await checkForUpdatesAndPrompt()
    await installDownloadedUpdate()

    expect(installMock).toHaveBeenCalledTimes(1)
    expect(invokeMock).toHaveBeenCalledWith('restart_app')
  })

  it('shows the original error for other update failures', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    checkMock.mockRejectedValue(new Error('network timeout'))
    const alertMock = vi.fn()
    stubWindow({ alert: alertMock as unknown as Window['alert'] })
    const { checkForUpdatesAndPrompt } = await loadUpdaterModule()

    await expect(checkForUpdatesAndPrompt()).resolves.toBeUndefined()

    expect(confirmDialogMock).not.toHaveBeenCalled()
    expect(alertMock).toHaveBeenCalledWith('Update check failed: Error: network timeout')
  })
})

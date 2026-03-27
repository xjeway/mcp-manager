import { beforeEach, describe, expect, it, vi } from 'vitest'

const getVersionMock = vi.fn<() => Promise<string>>()
const isDesktopRuntimeMock = vi.fn<() => boolean>()

vi.mock('@tauri-apps/api/app', () => ({
  getVersion: () => getVersionMock(),
}))

vi.mock('./runtime', () => ({
  isDesktopRuntime: () => isDesktopRuntimeMock(),
}))

describe('loadAppVersion', () => {
  beforeEach(() => {
    getVersionMock.mockReset()
    isDesktopRuntimeMock.mockReset()
  })

  it('returns the desktop runtime version when available', async () => {
    isDesktopRuntimeMock.mockReturnValue(true)
    getVersionMock.mockResolvedValue('3.4.5')

    const { loadAppVersion } = await import('./appVersion')

    await expect(loadAppVersion()).resolves.toBe('3.4.5')
  })

  it('falls back to the packaged version outside desktop runtime', async () => {
    isDesktopRuntimeMock.mockReturnValue(false)

    const { FALLBACK_APP_VERSION, loadAppVersion } = await import('./appVersion')

    await expect(loadAppVersion()).resolves.toBe(FALLBACK_APP_VERSION)
  })
})

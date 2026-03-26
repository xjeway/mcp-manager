import { check } from '@tauri-apps/plugin-updater'
import i18n from '../i18n'
import { RELEASES_URL, openReleasesLink } from './externalLinks'
import { isDesktopRuntime } from './runtime'

const MISSING_RELEASE_FEED_PATTERN = /valid release json from the remote/i

function isMissingReleaseFeedError(error: unknown): boolean {
  return MISSING_RELEASE_FEED_PATTERN.test(String(error))
}

async function offerReleasesPageFallback(): Promise<void> {
  const shouldOpenReleases = window.confirm(i18n.t('updateFeedUnavailablePrompt'))

  if (!shouldOpenReleases) {
    return
  }

  try {
    await openReleasesLink()
  } catch {
    window.alert(i18n.t('updateFeedUnavailableDownload', { url: RELEASES_URL }))
  }
}

export async function checkForUpdatesAndPrompt(options?: { silentIfNoUpdate?: boolean }): Promise<void> {
  if (!isDesktopRuntime()) {
    if (!options?.silentIfNoUpdate) {
      window.alert(i18n.t('updateDesktopOnly'))
    }
    return
  }

  try {
    const update = await check()
    if (!update) {
      if (!options?.silentIfNoUpdate) {
        window.alert(i18n.t('updateUnavailable'))
      }
      return
    }

    const confirmed = window.confirm(i18n.t('updateInstallPrompt', { version: update.version }))
    if (!confirmed) return

    await update.downloadAndInstall()
    window.alert(i18n.t('updateInstalled'))
  } catch (error) {
    if (!options?.silentIfNoUpdate) {
      if (isMissingReleaseFeedError(error)) {
        await offerReleasesPageFallback()
        return
      }

      window.alert(i18n.t('updateCheckFailedDetail', { error: String(error) }))
    }
  }
}

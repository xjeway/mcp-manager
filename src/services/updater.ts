import { check } from '@tauri-apps/plugin-updater'
import { isDesktopRuntime } from './runtime'

export async function checkForUpdatesAndPrompt(options?: { silentIfNoUpdate?: boolean }): Promise<void> {
  if (!isDesktopRuntime()) {
    if (!options?.silentIfNoUpdate) {
      window.alert('Update checks are only available in the desktop app.')
    }
    return
  }

  try {
    const update = await check()
    if (!update) {
      if (!options?.silentIfNoUpdate) {
        window.alert('No updates available')
      }
      return
    }

    const confirmed = window.confirm(`New version ${update.version} available. Install now?`)
    if (!confirmed) return

    await update.downloadAndInstall()
    window.alert('Update installed. Please restart app.')
  } catch (error) {
    if (!options?.silentIfNoUpdate) {
      window.alert(`Update check failed: ${String(error)}`)
    }
  }
}

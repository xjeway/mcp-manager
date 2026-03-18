import { check } from '@tauri-apps/plugin-updater'

function isTauriRuntime(): boolean {
  if (typeof window === 'undefined') {
    return false
  }

  const tauriWindow = window as {
    __TAURI__?: { invoke?: unknown }
    __TAURI_INTERNALS__?: { invoke?: unknown }
  }

  return typeof tauriWindow.__TAURI__?.invoke === 'function' || typeof tauriWindow.__TAURI_INTERNALS__?.invoke === 'function'
}

export async function checkForUpdatesAndPrompt(options?: { silentIfNoUpdate?: boolean }): Promise<void> {
  if (!isTauriRuntime()) {
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
    window.alert(`Update check failed: ${String(error)}`)
  }
}

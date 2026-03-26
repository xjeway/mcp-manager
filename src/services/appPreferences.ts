const AUTO_SYNC_ON_LAUNCH_KEY = 'ui-auto-sync-on-launch'

export function readAutoSyncOnLaunchPreference(): boolean {
  if (typeof window === 'undefined') {
    return true
  }

  return window.localStorage.getItem(AUTO_SYNC_ON_LAUNCH_KEY) !== 'false'
}

export function saveAutoSyncOnLaunchPreference(enabled: boolean): void {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.setItem(AUTO_SYNC_ON_LAUNCH_KEY, String(enabled))
}

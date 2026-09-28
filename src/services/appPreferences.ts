const AUTO_IMPORT_ON_LAUNCH_KEY = 'ui-auto-sync-on-launch'

export function readAutoImportOnLaunchPreference(): boolean {
  if (typeof window === 'undefined') {
    return true
  }

  return window.localStorage.getItem(AUTO_IMPORT_ON_LAUNCH_KEY) !== 'false'
}

export function saveAutoImportOnLaunchPreference(enabled: boolean): void {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.setItem(AUTO_IMPORT_ON_LAUNCH_KEY, String(enabled))
}

const MARKETPLACE_ENABLED_KEY = 'ui-marketplace-enabled'

export function readMarketplaceEnabledPreference(): boolean {
  if (typeof window === 'undefined') {
    return true
  }

  return window.localStorage.getItem(MARKETPLACE_ENABLED_KEY) !== 'false'
}

export function saveMarketplaceEnabledPreference(enabled: boolean): void {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.setItem(MARKETPLACE_ENABLED_KEY, String(enabled))
}

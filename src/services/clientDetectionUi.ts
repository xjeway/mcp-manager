import { SUPPORTED_APPS, type SupportedApp } from '../types/config'

export function shouldShowClientRedetectHint(visibleApps: SupportedApp[]): boolean {
  return visibleApps.length < SUPPORTED_APPS.length
}

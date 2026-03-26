import { SUPPORTED_APPS, type SupportedApp } from '../types/config'

export function deriveVisibleApps(installedApps: Iterable<string>): SupportedApp[] {
  const installed = new Set(installedApps)
  return SUPPORTED_APPS.filter((app) => installed.has(app))
}

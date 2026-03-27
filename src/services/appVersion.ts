import { getVersion } from '@tauri-apps/api/app'
import packageJson from '../../package.json'
import { isDesktopRuntime } from './runtime'

export const FALLBACK_APP_VERSION = packageJson.version

export async function loadAppVersion(): Promise<string> {
  if (!isDesktopRuntime()) {
    return FALLBACK_APP_VERSION
  }

  try {
    return await getVersion()
  } catch {
    return FALLBACK_APP_VERSION
  }
}

import { invoke } from '@tauri-apps/api/core'
import { isDesktopRuntime } from './runtime'

export const REPOSITORY_URL = 'https://github.com/xjeway/mcp-manager'
export const RELEASES_URL = `${REPOSITORY_URL}/releases`

export async function openRepositoryLink(): Promise<void> {
  if (!isDesktopRuntime()) {
    window.open(REPOSITORY_URL, '_blank', 'noopener,noreferrer')
    return
  }

  await invoke('open_repository_link')
}

export async function openReleasesLink(): Promise<void> {
  if (!isDesktopRuntime()) {
    window.open(RELEASES_URL, '_blank', 'noopener,noreferrer')
    return
  }

  await invoke('open_releases_link')
}

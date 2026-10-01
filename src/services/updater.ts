import { invoke } from '@tauri-apps/api/core'
import { emitTo } from '@tauri-apps/api/event'
import { check, type DownloadEvent, type Update } from '@tauri-apps/plugin-updater'
import i18n from '../i18n'
import { RELEASES_URL, openReleasesLink } from './externalLinks'
import { confirmDialog } from './nativeDialogs'
import { readAutomaticUpdateChecks } from './appPreferences'
import { isDesktopRuntime } from './runtime'
import { isUpdateProgressWindowVisible, showUpdateProgressWindow } from './updateProgressWindow'
import { UPDATE_PROGRESS_WINDOW_LABEL, UPDATE_STATE_EVENT } from './updaterEvents'

const MISSING_RELEASE_FEED_PATTERN = /valid release json from the remote/i

export type UpdatePhase = 'idle' | 'downloading' | 'downloaded' | 'installing' | 'error'

export interface UpdateState {
  currentVersion: string | null
  downloadedBytes: number
  error: string | null
  percent: number | null
  phase: UpdatePhase
  targetVersion: string | null
  totalBytes: number | null
}

const INITIAL_UPDATE_STATE: UpdateState = {
  currentVersion: null,
  downloadedBytes: 0,
  error: null,
  percent: null,
  phase: 'idle',
  targetVersion: null,
  totalBytes: null,
}

let activeUpdate: Update | null = null
let updateState: UpdateState = { ...INITIAL_UPDATE_STATE }
const listeners = new Set<(state: UpdateState) => void>()

function isMissingReleaseFeedError(error: unknown): boolean {
  return MISSING_RELEASE_FEED_PATTERN.test(String(error))
}

function isUpdateSessionActive(): boolean {
  return updateState.phase === 'downloading' || updateState.phase === 'downloaded' || updateState.phase === 'installing'
}

function calculatePercent(downloadedBytes: number, totalBytes: number | null): number | null {
  if (!totalBytes || totalBytes <= 0) {
    return null
  }

  return Math.min(100, Math.round((downloadedBytes / totalBytes) * 100))
}

function setUpdateState(nextState: UpdateState): void {
  updateState = nextState
  for (const listener of listeners) {
    listener(updateState)
  }

  void syncUpdateStateToWindow()
}

async function resetActiveUpdate(): Promise<void> {
  const currentUpdate = activeUpdate
  activeUpdate = null

  if (!currentUpdate) {
    return
  }

  try {
    await currentUpdate.close()
  } catch {
    // Ignore updater resource cleanup failures.
  }
}

async function offerReleasesPageFallback(): Promise<void> {
  const shouldOpenReleases = await confirmDialog({
    message: i18n.t('updateFeedUnavailablePrompt'),
  })

  if (!shouldOpenReleases) {
    return
  }

  try {
    await openReleasesLink()
  } catch {
    window.alert(i18n.t('updateFeedUnavailableDownload', { url: RELEASES_URL }))
  }
}

function handleDownloadEvent(event: DownloadEvent): void {
  if (event.event === 'Started') {
    setUpdateState({
      ...updateState,
      error: null,
      percent: 0,
      totalBytes: event.data.contentLength ?? null,
    })
    return
  }

  if (event.event === 'Progress') {
    const downloadedBytes = updateState.downloadedBytes + event.data.chunkLength
    setUpdateState({
      ...updateState,
      downloadedBytes,
      percent: calculatePercent(downloadedBytes, updateState.totalBytes),
    })
    return
  }

  setUpdateState({
    ...updateState,
    downloadedBytes: updateState.totalBytes ?? updateState.downloadedBytes,
    percent: 100,
  })
}

async function prepareUpdateDownload(update: Update): Promise<void> {
  activeUpdate = update
  setUpdateState({
    currentVersion: update.currentVersion,
    downloadedBytes: 0,
    error: null,
    percent: 0,
    phase: 'downloading',
    targetVersion: update.version,
    totalBytes: null,
  })
  await showUpdateProgressWindow()
}

async function handleInteractiveUpdateError(error: unknown): Promise<void> {
  setUpdateState({
    ...updateState,
    error: String(error),
    phase: 'error',
  })
  await showUpdateProgressWindow().catch(() => undefined)
}

export function getUpdateState(): UpdateState {
  return updateState
}

export function subscribeToUpdateState(listener: (state: UpdateState) => void): () => void {
  listeners.add(listener)
  listener(updateState)

  return () => {
    listeners.delete(listener)
  }
}

export async function syncUpdateStateToWindow(target = UPDATE_PROGRESS_WINDOW_LABEL): Promise<void> {
  await emitTo(target, UPDATE_STATE_EVENT, updateState).catch(() => undefined)
}

export async function hasVisibleUpdateProgressWindow(): Promise<boolean> {
  return await isUpdateProgressWindowVisible()
}

export async function checkForUpdatesAndPrompt(options?: { silentIfNoUpdate?: boolean }): Promise<void> {
  if (!isDesktopRuntime()) {
    if (!options?.silentIfNoUpdate) {
      window.alert(i18n.t('updateDesktopOnly'))
    }
    return
  }

  if (isUpdateSessionActive()) {
    await showUpdateProgressWindow()
    await syncUpdateStateToWindow()
    return
  }

  if (updateState.phase === 'error') {
    setUpdateState({ ...INITIAL_UPDATE_STATE })
    await resetActiveUpdate()
  }

  const automatic = options?.silentIfNoUpdate === true
  if (automatic && !readAutomaticUpdateChecks()) {
    return
  }
  if (isDesktopRuntime()) {
    try {
      await invoke('authorize_update_check', { automatic })
    } catch {
      // A manual check still runs. An automatic check stops so a refused
      // authorisation cannot fall through into a network request.
      if (automatic) {
        return
      }
    }
  }

  try {
    const update = await check()
    if (!update) {
      if (!options?.silentIfNoUpdate) {
        window.alert(i18n.t('updateUnavailable'))
      }
      return
    }

    const confirmed = await confirmDialog({
      message: i18n.t('updateDownloadPrompt', { version: update.version }),
    })
    if (!confirmed) {
      await update.close().catch(() => undefined)
      return
    }

    await prepareUpdateDownload(update)
    await update.download(handleDownloadEvent)

    setUpdateState({
      ...updateState,
      downloadedBytes: updateState.totalBytes ?? updateState.downloadedBytes,
      percent: 100,
      phase: 'downloaded',
    })
  } catch (error) {
    if (isUpdateSessionActive()) {
      await handleInteractiveUpdateError(error)
      return
    }

    if (!options?.silentIfNoUpdate) {
      if (isMissingReleaseFeedError(error)) {
        await offerReleasesPageFallback()
        return
      }

      window.alert(i18n.t('updateCheckFailedDetail', { error: String(error) }))
    }
  }
}

export async function installDownloadedUpdate(): Promise<void> {
  if (!activeUpdate || updateState.phase !== 'downloaded') {
    return
  }

  setUpdateState({
    ...updateState,
    error: null,
    percent: 100,
    phase: 'installing',
  })

  try {
    await activeUpdate.install()
    await invoke('restart_app')
  } catch (error) {
    await handleInteractiveUpdateError(error)
    return
  }

  await resetActiveUpdate()
}

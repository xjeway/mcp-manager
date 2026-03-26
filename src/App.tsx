import { useEffect, useMemo, useRef, useState } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useTranslation } from 'react-i18next'
import { Dashboard } from './components/Dashboard'
import { SettingsPage } from './components/SettingsPage'
import { ServerEditor } from './components/ServerEditor'
import { ToastViewport } from './components/ToastViewport'
import { mapConfigToWorkspaceView, type FeedbackItem } from './view-models/workspace'
import './i18n'
import { SUPPORTED_APPS, type ImportDetectedResult, type MCPConfig, type MCPServer, type SupportedApp } from './types/config'
import { readAutoSyncOnLaunchPreference, saveAutoSyncOnLaunchPreference } from './services/appPreferences'
import { applyConfig, detectInstalledApps, importDetectedConfigs, loadConfig, rollback, saveConfig } from './services/configService'
import { areConfigsEquivalent } from './services/configSync'
import { openRepositoryLink } from './services/externalLinks'
import { confirmDialog } from './services/nativeDialogs'
import { mergeServerIntoConfig, shouldPromptForPendingChanges } from './services/pendingChanges'
import { evaluateApplyRisks } from './services/risk'
import { isDesktopRuntime } from './services/runtime'
import { checkForUpdatesAndPrompt } from './services/updater'
import { deriveVisibleApps } from './services/visibleApps'
import {
  deleteServerFromConfig,
  persistImportedConfig,
  saveAndSyncConfig,
  toggleServerAppInConfig,
} from './services/workspacePersistence'

type View = 'dashboard' | 'editor' | 'settings'
type ThemeMode = 'light' | 'dark' | 'system'
type ActionState =
  | 'idle'
  | 'loading'
  | 'saving'
  | 'importing'
  | 'rolling-back'
  | 'checking-updates'

function readThemePreference(): ThemeMode {
  if (typeof window === 'undefined') {
    return 'system'
  }

  const stored = window.localStorage.getItem('ui-theme')
  return stored === 'light' || stored === 'dark' || stored === 'system' ? stored : 'system'
}

function readSystemTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') {
    return 'dark'
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function detectPlatform(): 'macos' | 'windows' | 'linux' | 'unknown' {
  if (typeof window === 'undefined') {
    return 'unknown'
  }

  const ua = window.navigator.userAgent.toLowerCase()
  if (ua.includes('mac os x') || ua.includes('macintosh')) {
    return 'macos'
  }
  if (ua.includes('windows')) {
    return 'windows'
  }
  if (ua.includes('linux')) {
    return 'linux'
  }
  return 'unknown'
}

export default function App() {
  const { t, i18n } = useTranslation()
  const [autoSyncOnLaunch, setAutoSyncOnLaunch] = useState(readAutoSyncOnLaunchPreference)
  const [theme, setTheme] = useState<ThemeMode>(readThemePreference)
  const [systemTheme, setSystemTheme] = useState<'light' | 'dark'>(readSystemTheme)
  const [platform] = useState(detectPlatform)
  const [view, setView] = useState<View>('dashboard')
  const [actionState, setActionState] = useState<ActionState>('loading')
  const [config, setConfig] = useState<MCPConfig>({ version: 1, servers: [] })
  const [editingServer, setEditingServer] = useState<MCPServer | null>(null)
  const [feedbacks, setFeedbacks] = useState<FeedbackItem[]>([])
  const [backups, setBackups] = useState<string[]>([])
  const [visibleApps, setVisibleApps] = useState<SupportedApp[]>([...SUPPORTED_APPS])
  const [editorDirty, setEditorDirty] = useState(false)
  const closeConfirmedRef = useRef(false)

  useEffect(() => {
    if (typeof window === 'undefined') {
      return
    }

    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = () => setSystemTheme(media.matches ? 'dark' : 'light')
    handleChange()
    media.addEventListener('change', handleChange)
    return () => media.removeEventListener('change', handleChange)
  }, [])

  useEffect(() => {
    const nextTheme = theme === 'system' ? systemTheme : theme
    document.documentElement.setAttribute('data-theme', nextTheme)
    document.documentElement.setAttribute('data-platform', platform)
    window.localStorage.setItem('ui-theme', theme)
  }, [platform, theme, systemTheme])

  const pushFeedback = (kind: FeedbackItem['kind'], message: string) => {
    setFeedbacks((current) => [
      { id: `${Date.now()}-${Math.random().toString(36).slice(2)}`, kind, message },
      ...current,
    ].slice(0, 4))
  }

  const dismissFeedback = (id: string) => {
    setFeedbacks((current) => current.filter((item) => item.id !== id))
  }

  const clearEditorState = () => {
    setEditingServer(null)
    setEditorDirty(false)
  }

  const navigateToView = (nextView: View) => {
    if (nextView !== 'editor') {
      clearEditorState()
    }
    setView(nextView)
  }

  const applyImportResult = async (
    result: ImportDetectedResult,
    baselineConfig: MCPConfig,
    mode: 'manual' | 'startup',
  ) => {
    const detectedCount = result.sources.filter((source) => source.exists).length
    const hasImportableServers = result.config.servers.length > 0
    const hasChanged = hasImportableServers && !areConfigsEquivalent(result.config, baselineConfig)
    const shouldCollapseErrors = mode === 'manual' && !hasImportableServers && result.errors.length > 0

    if (hasChanged) {
      try {
        await persistImportedConfig(result.config, saveConfig)
        setConfig(result.config)
        pushFeedback(
          mode === 'manual' ? 'success' : 'info',
          t(mode === 'manual' ? 'importSummary' : 'autoImportSummary', {
            count: detectedCount,
            servers: result.config.servers.length,
          }),
        )
      } catch (error) {
        pushFeedback('error', t('saveFailedDetail', { error: String(error) }))
      }
    } else if (mode === 'manual') {
      pushFeedback(
        result.errors.length > 0 ? 'error' : 'info',
        hasImportableServers
          ? t('importUpToDate')
          : result.errors.length > 0
            ? `${t('importFailed')}: ${result.errors.join(' | ')}`
            : t('importEmpty'),
      )
    }

    for (const warning of result.warnings) {
      pushFeedback('warning', warning)
    }
    if (!shouldCollapseErrors) {
      for (const error of result.errors) {
        pushFeedback('error', error)
      }
    }
  }

  useEffect(() => {
    let alive = true

    const refreshVisibleApps = async () => {
      try {
        const detected = await detectInstalledApps()
        if (!alive) {
          return
        }
        setVisibleApps(deriveVisibleApps(detected))
      } catch (error) {
        if (!alive) {
          return
        }
        setVisibleApps([...SUPPORTED_APPS])
        pushFeedback('warning', t('detectInstalledAppsFailedDetail', { error: String(error) }))
      }
    }

    const bootstrap = async () => {
      setActionState('loading')
      try {
        await refreshVisibleApps()
        const loaded = await loadConfig()
        if (!alive) {
          return
        }
        setConfig(loaded)
        setFeedbacks([])

        if (readAutoSyncOnLaunchPreference()) {
          try {
            const result = await importDetectedConfigs()
            if (!alive) {
              return
            }
            await applyImportResult(result, loaded, 'startup')
          } catch (error) {
            if (!alive) {
              return
            }
            pushFeedback('error', t('importFailedDetail', { error: String(error) }))
          }
        }
      } catch (error) {
        if (!alive) {
          return
        }
        pushFeedback('error', t('loadFailedDetail', { error: String(error) }))
      } finally {
        if (alive) {
          setActionState('idle')
        }
      }

      void checkForUpdatesAndPrompt({ silentIfNoUpdate: true })
    }

    void bootstrap()

    return () => {
      alive = false
    }
  }, [])

  useEffect(() => {
    if (typeof window === 'undefined') {
      return
    }

    const handleBeforeUnload = (event: BeforeUnloadEvent) => {
      if (!editorDirty) {
        return
      }

      event.preventDefault()
      event.returnValue = ''
    }

    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => window.removeEventListener('beforeunload', handleBeforeUnload)
  }, [editorDirty])

  useEffect(() => {
    if (!isDesktopRuntime()) {
      return
    }

    let disposed = false
    let unlisten: (() => void) | undefined

    void getCurrentWindow()
      .onCloseRequested(async (event) => {
        if (closeConfirmedRef.current) {
          return
        }

        if (
          !shouldPromptForPendingChanges({
            currentView: view,
            nextAction: { kind: 'close' },
            workspaceDirty: false,
            editorDirty,
          })
        ) {
          return
        }

        event.preventDefault()

        const shouldDiscard = await confirmDialog({
          cancelLabel: t('pendingChangesKeepEditing'),
          kind: 'warning',
          message: t('discardEditorChangesDescription'),
          okLabel: t('pendingChangesDiscard'),
          title: t('discardEditorChangesTitle'),
        })

        if (!shouldDiscard || disposed) {
          return
        }

        closeConfirmedRef.current = true
        await getCurrentWindow().destroy()
      })
      .then((cleanup) => {
        if (disposed) {
          cleanup()
          return
        }
        unlisten = cleanup
      })

    return () => {
      disposed = true
      unlisten?.()
    }
  }, [editorDirty, t, view])

  const workspace = useMemo(() => mapConfigToWorkspaceView(config, visibleApps, t), [config, t, visibleApps])

  const commitConfigChange = async ({
    nextConfig,
    successKind = 'success',
    successMessage,
  }: {
    nextConfig: MCPConfig
    successKind?: FeedbackItem['kind']
    successMessage?: string
  }): Promise<boolean> => {
    const summary = evaluateApplyRisks(nextConfig.servers)
    if (summary.blockingErrors.length > 0) {
      for (const error of summary.blockingErrors) {
        pushFeedback('error', error)
      }
      return false
    }

    setActionState('saving')
    try {
      const result = await saveAndSyncConfig({
        applyConfig,
        nextConfig,
        previousConfig: config,
        saveConfig,
      })
      setConfig(nextConfig)
      setBackups(result.backups)
      if (successMessage) {
        pushFeedback(successKind, successMessage)
      }
      return true
    } catch (error) {
      pushFeedback('error', t('syncFailedDetail', { error: String(error) }))
      return false
    } finally {
      setActionState('idle')
    }
  }

  const openCreate = () => {
    setEditingServer(null)
    setEditorDirty(false)
    setView('editor')
  }

  const openEdit = (serverId: string) => {
    const server = config.servers.find((item) => item.id === serverId)
    if (!server) {
      return
    }
    setEditingServer(server)
    setEditorDirty(false)
    setView('editor')
  }

  const handleDelete = async (serverId: string) => {
    const row = workspace.rows.find((item) => item.id === serverId)
    if (!row) {
      return
    }

    const confirmed = await confirmDialog({
      cancelLabel: t('cancel'),
      kind: 'warning',
      message: t('deleteConfirm', { name: row.name }),
      okLabel: t('delete'),
      title: t('delete'),
    })
    if (!confirmed) {
      return
    }

    const saved = await commitConfigChange({
      nextConfig: deleteServerFromConfig(config, serverId),
      successMessage: t('serverDeleted', { name: row.name }),
    })
    if (saved) {
      clearEditorState()
    }
  }

  const handleToggleApp = async (serverId: string, app: SupportedApp) => {
    await commitConfigChange({
      nextConfig: toggleServerAppInConfig(config, serverId, app),
    })
  }

  const handleImport = async () => {
    setActionState('importing')
    try {
      try {
        const detected = await detectInstalledApps()
        setVisibleApps(deriveVisibleApps(detected))
      } catch (error) {
        setVisibleApps([...SUPPORTED_APPS])
        pushFeedback('warning', t('detectInstalledAppsFailedDetail', { error: String(error) }))
      }

      const result = await importDetectedConfigs()
      await applyImportResult(result, config, 'manual')
    } catch (error) {
      pushFeedback('error', t('importFailedDetail', { error: String(error) }))
    } finally {
      setActionState('idle')
    }
  }

  const handleRollback = async () => {
    if (backups.length === 0) {
      pushFeedback('info', t('rollbackUnavailable'))
      return
    }

    setActionState('rolling-back')
    try {
      await rollback(backups)
      pushFeedback('success', t('rollbackSuccess'))
    } catch (error) {
      pushFeedback('error', t('rollbackFailedDetail', { error: String(error) }))
    } finally {
      setActionState('idle')
    }
  }

  const handleCheckUpdates = async () => {
    setActionState('checking-updates')
    try {
      await checkForUpdatesAndPrompt()
    } finally {
      setActionState('idle')
    }
  }

  const handleOpenRepository = async () => {
    try {
      await openRepositoryLink()
    } catch (error) {
      pushFeedback('error', t('openRepositoryFailedDetail', { error: String(error) }))
    }
  }

  const handleCopyCommand = async (serverId: string) => {
    const row = workspace.rows.find((item) => item.id === serverId)
    if (!row) {
      return
    }

    try {
      await navigator.clipboard.writeText(row.copyValue)
      pushFeedback('success', t('copySuccess', { name: row.name }))
    } catch (error) {
      pushFeedback('error', t('copyFailedDetail', { error: String(error) }))
    }
  }

  const handleSaveServer = async (server: MCPServer) => {
    const nextConfig = mergeServerIntoConfig(config, editingServer?.id, server)
    const saved = await commitConfigChange({
      nextConfig,
      successMessage: t(editingServer ? 'serverUpdated' : 'serverCreated', { name: server.name }),
    })

    if (!saved) {
      return
    }

    clearEditorState()
    setView('dashboard')
  }

  const handleEditorCancel = async () => {
    if (
      !shouldPromptForPendingChanges({
        currentView: view,
        nextAction: { kind: 'view', view: 'dashboard' },
        workspaceDirty: false,
        editorDirty,
      })
    ) {
      navigateToView('dashboard')
      return
    }

    const shouldDiscard = await confirmDialog({
      cancelLabel: t('pendingChangesKeepEditing'),
      kind: 'warning',
      message: t('discardEditorChangesDescription'),
      okLabel: t('pendingChangesDiscard'),
      title: t('discardEditorChangesTitle'),
    })

    if (!shouldDiscard) {
      return
    }

    navigateToView('dashboard')
  }

  const handleAutoSyncOnLaunchChange = (enabled: boolean) => {
    setAutoSyncOnLaunch(enabled)
    saveAutoSyncOnLaunchPreference(enabled)
  }

  const isBusy = actionState !== 'idle'

  return (
    <>
      <ToastViewport items={feedbacks} onDismiss={dismissFeedback} />
      {view === 'editor' ? (
        <ServerEditor
          server={editingServer}
          busy={isBusy}
          visibleApps={visibleApps}
          onDraftChange={(_draft, dirty) => setEditorDirty(dirty)}
          onSave={(server) => void handleSaveServer(server)}
          onCancel={() => void handleEditorCancel()}
        />
      ) : view === 'settings' ? (
        <SettingsPage
          autoSyncOnLaunch={autoSyncOnLaunch}
          busy={isBusy}
          language={i18n.language}
          onOpenRepository={() => void handleOpenRepository()}
          onAutoSyncOnLaunchChange={handleAutoSyncOnLaunchChange}
          theme={theme}
          onBack={() => navigateToView('dashboard')}
          onCheckUpdates={() => void handleCheckUpdates()}
          onLanguageChange={(language) => void i18n.changeLanguage(language)}
          onThemeChange={setTheme}
        />
      ) : (
        <Dashboard
          workspace={workspace}
          busy={actionState}
          canRollback={backups.length > 0}
          visibleApps={visibleApps}
          onOpenRepository={() => void handleOpenRepository()}
          onSyncLocalConfig={() => void handleImport()}
          onOpenSettings={() => navigateToView('settings')}
          onCopyCommand={(serverId) => void handleCopyCommand(serverId)}
          onAdd={openCreate}
          onEdit={openEdit}
          onDelete={(serverId) => void handleDelete(serverId)}
          onToggleApp={(serverId, app) => void handleToggleApp(serverId, app)}
          onRollback={() => void handleRollback()}
        />
      )}
    </>
  )
}

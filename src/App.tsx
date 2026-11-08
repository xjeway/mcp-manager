import { useEffect, useMemo, useRef, useState } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useTranslation } from 'react-i18next'
import { CLIENTS } from './components/clientMeta'
import { ConfigProblemDialog } from './components/ConfigProblemDialog'
import { Dashboard } from './components/Dashboard'
import { ProjectSelector } from './components/ProjectSelector'
import { configForProject, mergeProjectChange, projectContextChanged, readProjectSelection, readRecentProjects, saveProjectSelection } from './services/projectContext'
import { MarketplacePage } from './components/MarketplacePage'
import { SettingsPage } from './components/SettingsPage'
import { ServerEditor } from './components/ServerEditor'
import { ToastViewport } from './components/ToastViewport'
import { UpdateIndicator } from './components/UpdateIndicator'
import { UpdateProgressWindow } from './components/UpdateProgressWindow'
import { applyWarningMessages, mapConfigToWorkspaceView, type FeedbackItem } from './view-models/workspace'
import { getLanguagePreference, setLanguagePreference } from './i18n'
import {
  EMPTY_WORKSPACE,
  SUPPORTED_APPS,
  type ImportDetectedResult,
  type MCPConfig,
  type MCPServer,
  type SupportedApp,
  type WorkspaceContext,
} from './types/config'
import {
  readAutoImportOnLaunchPreference,
  readMarketplaceEnabledPreference,
  saveAutoImportOnLaunchPreference,
  saveMarketplaceEnabledPreference,
} from './services/appPreferences'
import { setMarketplaceEnabled as syncMarketplaceEnabled } from './services/marketplaceService'
import {
  applyConfig,
  ConfigConflictError,
  ConfigTooNewError,
  projectSecretWarnings,
  detectInstalledApps,
  hasExternalConfigChange,
  getCurrentWorkspace,
  importDetectedConfigs,
  loadConfig,
  getRollbackDepth,
  rollback,
  saveConfig,
  startOverConfig,
  ConfigUnreadableError,
} from './services/configService'
import { areConfigsEquivalent } from './services/configSync'
import { openPath, openRepositoryLink } from './services/externalLinks'
import { chooseProjectDirectory, confirmDialog } from './services/nativeDialogs'
import { mergeServerIntoConfig, shouldPromptForPendingChanges } from './services/pendingChanges'
import { evaluateApplyRisks, sharedProjectSecretWarnings } from './services/risk'
import { isDesktopRuntime } from './services/runtime'
import { FALLBACK_APP_VERSION, loadAppVersion } from './services/appVersion'
import {
  checkForUpdatesAndPrompt,
  getUpdateState,
  hasVisibleUpdateProgressWindow,
  installDownloadedUpdate,
  subscribeToUpdateState,
  syncUpdateStateToWindow,
} from './services/updater'
import { showUpdateProgressWindow } from './services/updateProgressWindow'
import { UPDATE_INSTALL_REQUEST_EVENT, UPDATE_SYNC_REQUEST_EVENT } from './services/updaterEvents'
import { deriveVisibleApps } from './services/visibleApps'
import { resolveWindowSurface } from './services/windowSurface'
import {
  inheritedAgentsRemovalPath,
  refreshMiMoProjectPaths,
  deleteServerFromConfig,
  importServersIntoConfig,
  importConfigOnLaunch,
  persistImportedConfig,
  saveAndSyncConfig,
  setServersAppInConfig,
  toggleServerAppInConfig,
} from './services/workspacePersistence'

type View = 'dashboard' | 'editor' | 'settings' | 'marketplace'
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

function MainApp() {
  const { t } = useTranslation()
  const [autoImportOnLaunch, setAutoImportOnLaunch] = useState(readAutoImportOnLaunchPreference)
  const [theme, setTheme] = useState<ThemeMode>(readThemePreference)
  const [languagePreference, setLanguagePreferenceState] = useState(getLanguagePreference)
  const [systemTheme, setSystemTheme] = useState<'light' | 'dark'>(readSystemTheme)
  const [platform] = useState(detectPlatform)
  const [appVersion, setAppVersion] = useState(FALLBACK_APP_VERSION)
  const [currentWorkspace, setCurrentWorkspace] = useState<WorkspaceContext>(EMPTY_WORKSPACE)
  const [recentProjects, setRecentProjects] = useState(readRecentProjects)
  const [workspaceProblem, setWorkspaceProblem] = useState<string | null>(null)
  const [view, setView] = useState<View>('dashboard')
  const [actionState, setActionState] = useState<ActionState>('loading')
  const [config, setConfig] = useState<MCPConfig>({ version: 1, servers: [] })
  // Set while servers.yaml cannot be read; the service refuses saves meanwhile.
  const [configProblem, setConfigProblem] = useState<ConfigUnreadableError | null>(null)
  const [editingServer, setEditingServer] = useState<MCPServer | null>(null)
  // A new server prefilled from the marketplace; the editor still treats it as "add".
  const [prefillServer, setPrefillServer] = useState<MCPServer | null>(null)
  // Where the editor's cancel button returns to.
  const [editorOrigin, setEditorOrigin] = useState<'dashboard' | 'marketplace'>('dashboard')
  const [marketplaceEnabled, setMarketplaceEnabled] = useState(readMarketplaceEnabledPreference)
  const [feedbacks, setFeedbacks] = useState<FeedbackItem[]>([])
  // How many changes can be undone one after another (kept on disk, so it survives restarts).
  const [rollbackDepth, setRollbackDepth] = useState(0)
  const [visibleApps, setVisibleApps] = useState<SupportedApp[]>([...SUPPORTED_APPS])
  const [editorDirty, setEditorDirty] = useState(false)
  const configCommitInFlightRef = useRef(false)
  const closeConfirmedRef = useRef(false)
  const updateStateRef = useRef(getUpdateState())
  const [updateState, setUpdateState] = useState(getUpdateState)
  const updateReadyToastRef = useRef<string | null>(null)

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

  useEffect(() => {
    let alive = true

    void loadAppVersion().then((version) => {
      if (alive) {
        setAppVersion(version)
      }
    })

    return () => {
      alive = false
    }
  }, [])

  const pushFeedback = (
    kind: FeedbackItem['kind'],
    message: string,
    options?: Pick<FeedbackItem, 'persistent' | 'action'>,
  ): string => {
    const id = `${Date.now()}-${Math.random().toString(36).slice(2)}`
    setFeedbacks((current) => [{ id, kind, message, ...options }, ...current].slice(0, 4))
    return id
  }

  const dismissFeedback = (id: string) => {
    setFeedbacks((current) => current.filter((item) => item.id !== id))
  }

  useEffect(() => {
    const unsubscribe = subscribeToUpdateState((state) => {
      const previous = updateStateRef.current
      updateStateRef.current = state
      setUpdateState(state)

      if (state.phase === 'downloaded' && previous.phase !== 'downloaded' && state.targetVersion) {
        void hasVisibleUpdateProgressWindow().then((visible) => {
          // The update stays pending until the user restarts, so this toast does too.
          if (!visible && updateStateRef.current.phase === 'downloaded') {
            updateReadyToastRef.current = pushFeedback(
              'success',
              t('updateReadyBackground', { version: state.targetVersion }),
              { persistent: true, action: { label: t('updateWindowInstallNow'), onClick: () => void installDownloadedUpdate() } },
            )
          }
        })
      }

      const readyToast = updateReadyToastRef.current
      if (state.phase !== 'downloaded' && readyToast) {
        updateReadyToastRef.current = null
        setFeedbacks((current) => current.filter((item) => item.id !== readyToast))
      }

      if (state.phase === 'error' && state.error && previous.error !== state.error) {
        void hasVisibleUpdateProgressWindow().then((visible) => {
          if (!visible) {
            pushFeedback('error', t('updateCheckFailedDetail', { error: state.error }))
          }
        })
      }
    })

    return unsubscribe
  }, [t])

  const clearEditorState = () => {
    setEditingServer(null)
    setPrefillServer(null)
    setEditorOrigin('dashboard')
    setEditorDirty(false)
  }

  // The backend refuses marketplace requests until told the user allows them.
  useEffect(() => {
    void syncMarketplaceEnabled(marketplaceEnabled)
  }, [marketplaceEnabled])

  const navigateToView = (nextView: View) => {
    if (nextView !== 'editor') {
      clearEditorState()
    }
    setView(nextView)
  }

  /** Replaces the in-memory config with servers.yaml as another program (e.g. the CLI) left it. */
  const refreshRollbackDepth = async () => {
    try {
      setRollbackDepth(await getRollbackDepth())
    } catch {
      setRollbackDepth(0)
    }
  }

  const reloadConfigFromDisk = async (kind: FeedbackItem['kind'], message: string) => {
    try {
      const loaded = await loadConfig()
      setConfig(loaded)
      try { setCurrentWorkspace(await getCurrentWorkspace(currentWorkspace.root)); setWorkspaceProblem(null) }
      catch (error) { setWorkspaceProblem(String(error)) }
      setConfigProblem(null)
      pushFeedback(kind, message)
    } catch (error) {
      if (error instanceof ConfigUnreadableError) {
        setConfigProblem(error)
      } else {
        pushFeedback('error', t('loadFailedDetail', { error: String(error) }))
      }
    }
  }

  const applyImportResult = async (
    result: ImportDetectedResult,
    baselineConfig: MCPConfig,
    mode: 'manual' | 'startup',
    importWorkspace: WorkspaceContext = currentWorkspace,
  ) => {
    const detectedCount = result.sources.filter((source) => source.exists).length
    const hasImportableServers = result.config.servers.length > 0
    const hasChanged = hasImportableServers && !areConfigsEquivalent(result.config, baselineConfig)
    const shouldCollapseErrors = mode === 'manual' && !hasImportableServers && result.errors.length > 0

    if (hasChanged) {
      try {
        const imported = mergeProjectChange(baselineConfig, result.config, importWorkspace)
        await persistImportedConfig(imported, (next) => saveConfig(next, importWorkspace))
        setConfig(imported)
        try { setCurrentWorkspace(await getCurrentWorkspace(importWorkspace.root)); setWorkspaceProblem(null) }
        catch (error) { setWorkspaceProblem(String(error)) }
        pushFeedback(
          mode === 'manual' ? 'success' : 'info',
          t(mode === 'manual' ? 'importSummary' : 'autoImportSummary', {
            count: detectedCount,
            servers: result.config.servers.length,
          }),
        )
      } catch (error) {
        if (error instanceof ConfigUnreadableError) {
          setConfigProblem(error)
          pushFeedback('error', t('configUnreadableNotSaved'))
        } else if (error instanceof ConfigConflictError) {
          await reloadConfigFromDisk('warning', t('configConflictReloaded'))
        } else if (error instanceof ConfigTooNewError) {
          pushFeedback('error', t('configTooNew'))
        } else {
          pushFeedback('error', t('saveFailedDetail', { error: String(error) }))
        }
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

    const refreshVisibleApps = async (project: WorkspaceContext) => {
      try {
        const detected = await detectInstalledApps(project)
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
        let initialWorkspace = EMPTY_WORKSPACE
        let projectAvailable = true
        const selectedRoot = readProjectSelection()
        try {
          initialWorkspace = await getCurrentWorkspace(selectedRoot)
          setCurrentWorkspace(initialWorkspace)
        } catch (error) {
          projectAvailable = false
          // Retain the unavailable selection so choosing My computer can recover.
          setCurrentWorkspace({ ...EMPTY_WORKSPACE, root: selectedRoot ?? '' })
          setWorkspaceProblem(String(error))
          pushFeedback('error', t('projectSelectionFailed', { error: String(error) }))
        }
        await refreshVisibleApps(initialWorkspace)
        const loaded = await loadConfig()
        if (!alive) {
          return
        }
        setConfig(loaded)
        setFeedbacks([])
        void refreshRollbackDepth()

        if (projectAvailable && readAutoImportOnLaunchPreference()) {
          try {
            const result = await importConfigOnLaunch({
              autoImportOnLaunch: true,
              importDetectedConfigs: () => importDetectedConfigs(initialWorkspace),
            })
            if (!alive) {
              return
            }
            if (result) {
              await applyImportResult(result, loaded, 'startup', initialWorkspace)
            }
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
        if (error instanceof ConfigUnreadableError) {
          // Auto-import is skipped too: it would save over the file.
          setConfigProblem(error)
        } else {
          pushFeedback('error', t('loadFailedDetail', { error: String(error) }))
        }
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

    // Changes are saved as they are made, so picking up an outside edit never
    // discards anything here; an open editor draft is separate state and survives.
    const handleFocus = async () => {
      // The CLI may have recorded (or undone) a change meanwhile.
      void refreshRollbackDepth()
      try {
        if (await hasExternalConfigChange()) {
          await reloadConfigFromDisk('info', t('configChangedExternally'))
        }
      } catch {
        // The next save still checks for conflicts.
      }
    }

    window.addEventListener('focus', handleFocus)
    return () => window.removeEventListener('focus', handleFocus)
  }, [t])

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

  useEffect(() => {
    if (!isDesktopRuntime()) {
      return
    }

    let disposed = false
    let syncCleanup: (() => void) | undefined
    let installCleanup: (() => void) | undefined

    void getCurrentWindow()
      .listen(UPDATE_SYNC_REQUEST_EVENT, () => {
        void syncUpdateStateToWindow()
      })
      .then((cleanup) => {
        if (disposed) {
          cleanup()
          return
        }
        syncCleanup = cleanup
      })

    void getCurrentWindow()
      .listen(UPDATE_INSTALL_REQUEST_EVENT, () => {
        void installDownloadedUpdate()
      })
      .then((cleanup) => {
        if (disposed) {
          cleanup()
          return
        }
        installCleanup = cleanup
      })

    return () => {
      disposed = true
      syncCleanup?.()
      installCleanup?.()
    }
  }, [])

  const projectConfig = useMemo(() => configForProject(config, currentWorkspace), [config, currentWorkspace])
  const workspace = useMemo(() => mapConfigToWorkspaceView(projectConfig, visibleApps, t), [projectConfig, t, visibleApps])

  const selectProject = async (root?: string) => {
    if (configCommitInFlightRef.current || actionState !== 'idle') return
    configCommitInFlightRef.current = true
    setActionState('loading')
    try {
      const selected = root ?? await chooseProjectDirectory(t('projectChoose'))
      if (selected === null) return
      const next = await getCurrentWorkspace(selected)
      saveProjectSelection(next.root)
      setCurrentWorkspace(next)
      setWorkspaceProblem(null)
      setRecentProjects(readRecentProjects())
      try { setVisibleApps(deriveVisibleApps(await detectInstalledApps(next))) }
      catch { setVisibleApps([...SUPPORTED_APPS]) }
    } catch (error) {
      pushFeedback('error', t('projectSelectionFailed', { error: String(error) }))
    } finally {
      configCommitInFlightRef.current = false
      setActionState('idle')
    }
  }

  const commitConfigChange = async ({
    nextConfig,
    successKind = 'success',
    successMessage,
  }: {
    nextConfig: MCPConfig
    successKind?: FeedbackItem['kind']
    successMessage?: string
    editedServerIds?: string[]
  }): Promise<boolean> => {
    if (configCommitInFlightRef.current) return false
    if (workspaceProblem) { pushFeedback('error', t('projectSelectionFailed', { error: workspaceProblem })); return false }
    configCommitInFlightRef.current = true
    try {
      let workspaceForSave = currentWorkspace
      try {
        workspaceForSave = await getCurrentWorkspace(currentWorkspace.root)
        if (projectContextChanged(currentWorkspace, workspaceForSave)) {
          setCurrentWorkspace(workspaceForSave)
          pushFeedback('warning', t('configChangedExternally'))
          return false
        }
        nextConfig = refreshMiMoProjectPaths(nextConfig, currentWorkspace, workspaceForSave, nextConfig.servers.map((server) => server.id))
        nextConfig = mergeProjectChange(config, nextConfig, workspaceForSave)
      } catch (error) {
        pushFeedback('error', String(error))
        return false
      }
      const inheritedPath = inheritedAgentsRemovalPath(config, nextConfig, workspaceForSave)
      if (inheritedPath) {
        pushFeedback('warning', t('inheritedAgentsReadOnly', { path: inheritedPath }))
        return false
      }
      const scopedNext = configForProject(nextConfig, workspaceForSave)
      const summary = evaluateApplyRisks(scopedNext.servers)
      if (summary.blockingErrors.length > 0) {
        for (const error of summary.blockingErrors) {
          pushFeedback('error', error)
        }
        return false
      }

      const secretWarnings = sharedProjectSecretWarnings(scopedNext.servers, workspaceForSave, t)
      try {
        for (const warning of await projectSecretWarnings(nextConfig, workspaceForSave)) {
          const message = t('sharedProjectSecretWarning', {
            server: nextConfig.servers.find((server) => server.id === warning.serverId)?.name || warning.serverId,
            client: CLIENTS.find((client) => client.id === warning.app)?.label ?? warning.app, path: warning.path, key: warning.key,
            advice: t(warning.app === 'agents' ? 'secretAdviceDollarInterpolation' : warning.app === 'windsurf' ? 'secretAdviceDevin' : ['grok', 'claudeCode', 'factoryDroid'].includes(warning.app) ? 'secretAdviceInterpolation' : 'secretAdvicePrivate'),
          })
          if (!secretWarnings.includes(message)) secretWarnings.push(message)
        }
      } catch (error) {
        pushFeedback('error', String(error))
        return false
      }
      if (secretWarnings.length > 0) {
        const confirmed = await confirmDialog({
          cancelLabel: t('cancel'),
          kind: 'warning',
          message: secretWarnings.join('\n\n'),
          title: t('sharedProjectSecretTitle'),
        })
        if (!confirmed) return false
      }

      setActionState('saving')
      try {
        const result = await saveAndSyncConfig({
          applyConfig: (next, previous) => applyConfig(next, previous, workspaceForSave, currentWorkspace),
          nextConfig,
          previousConfig: config,
          saveConfig: (next) => saveConfig(next, workspaceForSave),
          workspace: workspaceForSave,
        })
        setConfig(nextConfig)
        setCurrentWorkspace(workspaceForSave)
        void refreshRollbackDepth()
        for (const message of applyWarningMessages(result, nextConfig.servers, t)) {
          pushFeedback('warning', message)
        }
        if (successMessage) {
          pushFeedback(successKind, successMessage)
        }
        return true
      } catch (error) {
        if (error instanceof ConfigUnreadableError) {
          setConfigProblem(error)
          pushFeedback('error', t('configUnreadableNotSaved'))
        } else if (error instanceof ConfigConflictError) {
          await reloadConfigFromDisk('warning', t('configConflictReloaded'))
        } else if (error instanceof ConfigTooNewError) {
          pushFeedback('error', t('configTooNew'))
        } else {
          pushFeedback('error', t('syncFailedDetail', { error: String(error) }))
        }
        return false
      } finally {
        setActionState('idle')
      }
    } finally {
      configCommitInFlightRef.current = false
    }
  }

  const openCreate = () => {
    clearEditorState()
    setView('editor')
  }

  const openFromMarketplace = (server: MCPServer) => {
    clearEditorState()
    setPrefillServer(server)
    setEditorOrigin('marketplace')
    setView('editor')
  }

  const openEdit = (serverId: string) => {
    const server = projectConfig.servers.find((item) => item.id === serverId)
    if (!server) {
      return
    }
    clearEditorState()
    setEditingServer(server)
    setView('editor')
  }

  const handleDelete = async (serverId: string) => {
    const row = workspace.rows.find((item) => item.id === serverId)
    if (!row) {
      return
    }

    if (configCommitInFlightRef.current) return
    configCommitInFlightRef.current = true
    let confirmed = false
    try {
      confirmed = await confirmDialog({
      cancelLabel: t('cancel'),
      kind: 'warning',
      message: t('deleteConfirm', { name: row.name }),
      okLabel: t('delete'),
      title: t('delete'),
      })
    } finally { configCommitInFlightRef.current = false }
    if (!confirmed) {
      return
    }

    const saved = await commitConfigChange({
      nextConfig: deleteServerFromConfig(projectConfig, serverId),
      successMessage: t('serverDeleted', { name: row.name }),
    })
    if (saved) {
      clearEditorState()
    }
  }

  const handleToggleApp = async (serverId: string, app: SupportedApp) => {
    await commitConfigChange({
      nextConfig: toggleServerAppInConfig(projectConfig, serverId, app),
    })
  }

  const handleBatchSetApp = async (serverIds: string[], app: SupportedApp, enabled: boolean) => {
    const client = CLIENTS.find((item) => item.id === app)?.label ?? app
    await commitConfigChange({
      nextConfig: setServersAppInConfig(projectConfig, serverIds, app, enabled),
      successMessage: t(enabled ? 'batchEnabledSummary' : 'batchDisabledSummary', {
        client,
        count: serverIds.length,
      }),
    })
  }

  const handleImport = async () => {
    if (configCommitInFlightRef.current || workspaceProblem) return
    configCommitInFlightRef.current = true
    setActionState('importing')
    try {
      try {
        const detected = await detectInstalledApps(currentWorkspace)
        setVisibleApps(deriveVisibleApps(detected))
      } catch (error) {
        setVisibleApps([...SUPPORTED_APPS])
        pushFeedback('warning', t('detectInstalledAppsFailedDetail', { error: String(error) }))
      }

      const result = await importDetectedConfigs(currentWorkspace)
      await applyImportResult(result, config, 'manual')
    } catch (error) {
      pushFeedback('error', t('importFailedDetail', { error: String(error) }))
    } finally {
      configCommitInFlightRef.current = false
      setActionState('idle')
    }
  }

  /** Backs up the unreadable servers.yaml, then replaces it with an empty list or, with `restore`, the one shown. */
  const handleStartOver = async (restore: boolean) => {
    if (!configProblem) {
      return
    }
    const count = config.servers.length
    const confirmed = await confirmDialog({
      cancelLabel: t('cancel'),
      kind: 'warning',
      message: t(restore ? 'configRestoreConfirm' : 'configStartOverConfirm', { path: configProblem.path, count }),
      okLabel: t(restore ? 'configRestoreOk' : 'configStartOverOk'),
      title: t(restore ? 'configRestoreTitle' : 'configStartOverTitle'),
    })
    if (!confirmed) {
      return
    }

    setActionState('saving')
    try {
      const { config: fresh, backupPath } = await startOverConfig(restore ? config : undefined)
      setConfig(fresh)
      setConfigProblem(null)
      pushFeedback('success', t(restore ? 'configRestored' : 'configStartedOver', { path: backupPath, count }))
    } catch (error) {
      if (error instanceof ConfigConflictError) {
        await reloadConfigFromDisk('info', t('configChangedExternally'))
      } else {
        pushFeedback('error', t('configStartOverFailed', { error: String(error) }))
      }
    } finally {
      setActionState('idle')
    }
  }

  const handleShowConfigFile = async () => {
    if (!configProblem) {
      return
    }
    try {
      await openPath(configProblem.path)
    } catch (error) {
      pushFeedback('error', String(error))
    }
  }

  const handleRollback = async () => {
    if (rollbackDepth === 0) {
      pushFeedback('info', t('rollbackUnavailable'))
      return
    }

    if (configCommitInFlightRef.current) return
    configCommitInFlightRef.current = true
    setActionState('rolling-back')
    try {
      await rollback()
      await reloadConfigFromDisk('success', t('rollbackSuccess'))
    } catch (error) {
      pushFeedback('error', t('rollbackFailedDetail', { error: String(error) }))
    } finally {
      await refreshRollbackDepth()
      configCommitInFlightRef.current = false
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
    const nextConfig = mergeServerIntoConfig(projectConfig, editingServer?.id, server)
    const saved = await commitConfigChange({
      nextConfig,
      successMessage: t(editingServer ? 'serverUpdated' : 'serverCreated', { name: server.name }),
      editedServerIds: [server.id],
    })

    if (!saved) {
      return
    }

    clearEditorState()
    setView('dashboard')
  }

  const handleSaveServers = async (servers: MCPServer[]) => {
    const { config: nextConfig, added, updated } = importServersIntoConfig(projectConfig, servers)
    const saved = await commitConfigChange({
      nextConfig,
      successMessage: t('batchImportSummary', { added: added.length, updated: updated.length }),
      editedServerIds: servers.map((server) => server.id),
    })

    if (!saved) {
      return
    }

    clearEditorState()
    setView('dashboard')
  }

  const handleEditorCancel = async () => {
    const destination = editorOrigin
    if (
      !shouldPromptForPendingChanges({
        currentView: view,
        nextAction: { kind: 'view', view: destination },
        workspaceDirty: false,
        editorDirty,
      })
    ) {
      navigateToView(destination)
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

    navigateToView(destination)
  }

  const handleMarketplaceEnabledChange = (enabled: boolean) => {
    setMarketplaceEnabled(enabled)
    saveMarketplaceEnabledPreference(enabled)
  }

  const handleAutoImportOnLaunchChange = (enabled: boolean) => {
    setAutoImportOnLaunch(enabled)
    saveAutoImportOnLaunchPreference(enabled)
  }

  const isBusy = actionState !== 'idle'

  return (
    <>
      <ToastViewport items={feedbacks} onDismiss={dismissFeedback} />
      {configProblem ? (
        <ConfigProblemDialog
          path={configProblem.path}
          detail={configProblem.detail}
          busy={isBusy}
          restorableCount={config.servers.length}
          onRestore={() => void handleStartOver(true)}
          onShowFile={() => void handleShowConfigFile()}
          onReload={() => void reloadConfigFromDisk('success', t('configReadableAgain'))}
          onStartOver={() => void handleStartOver(false)}
        />
      ) : null}
      {view === 'editor' ? (
        <ServerEditor
          server={editingServer}
          initialServer={prefillServer}
          busy={isBusy}
          workspace={currentWorkspace}
          visibleApps={visibleApps}
          onDraftChange={(_draft, dirty) => setEditorDirty(dirty)}
          onSave={(server) => void handleSaveServer(server)}
          onSaveMany={(servers) => void handleSaveServers(servers)}
          onCancel={() => void handleEditorCancel()}
        />
      ) : view === 'settings' ? (
        <SettingsPage
          appVersion={appVersion}
          autoImportOnLaunch={autoImportOnLaunch}
          busy={isBusy}
          checkingUpdates={actionState === 'checking-updates'}
          language={languagePreference}
          marketplaceEnabled={marketplaceEnabled}
          onMarketplaceEnabledChange={handleMarketplaceEnabledChange}
          onOpenRepository={() => void handleOpenRepository()}
          onAutoImportOnLaunchChange={handleAutoImportOnLaunchChange}
          theme={theme}
          onBack={() => navigateToView('dashboard')}
          onCheckUpdates={() => void handleCheckUpdates()}
          onLanguageChange={(language) => {
            setLanguagePreferenceState(language)
            setLanguagePreference(language)
          }}
          onThemeChange={setTheme}
        />
      ) : view === 'marketplace' && marketplaceEnabled ? (
        <MarketplacePage
          servers={config.servers}
          onBack={() => navigateToView('dashboard')}
          onInstall={openFromMarketplace}
        />
      ) : (
        <Dashboard
          workspace={workspace}
          projectSelector={<ProjectSelector workspace={currentWorkspace} recent={recentProjects} disabled={isBusy} problem={workspaceProblem} onSelect={(root) => void selectProject(root)} />}
          busy={actionState}
          canRollback={rollbackDepth > 0}
          visibleApps={visibleApps}
          onOpenRepository={() => void handleOpenRepository()}
          onSyncLocalConfig={() => void handleImport()}
          onOpenSettings={() => navigateToView('settings')}
          onCopyCommand={(serverId) => void handleCopyCommand(serverId)}
          onAdd={openCreate}
          onOpenMarketplace={marketplaceEnabled ? () => navigateToView('marketplace') : undefined}
          onEdit={openEdit}
          onDelete={(serverId) => void handleDelete(serverId)}
          onToggleApp={(serverId, app) => void handleToggleApp(serverId, app)}
          onBatchSetApp={(serverIds, app, enabled) => void handleBatchSetApp(serverIds, app, enabled)}
          onRollback={() => void handleRollback()}
          updateIndicator={
            <UpdateIndicator
              state={updateState}
              onInstall={() => void installDownloadedUpdate()}
              onShowProgress={() => void showUpdateProgressWindow().then(() => syncUpdateStateToWindow())}
            />
          }
        />
      )}
    </>
  )
}

function UpdateProgressWindowApp() {
  const [theme] = useState<ThemeMode>(readThemePreference)
  const [systemTheme, setSystemTheme] = useState<'light' | 'dark'>(readSystemTheme)
  const [platform] = useState(detectPlatform)

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
  }, [platform, systemTheme, theme])

  return <UpdateProgressWindow />
}

export default function App() {
  return resolveWindowSurface() === 'updater-progress' ? <UpdateProgressWindowApp /> : <MainApp />
}

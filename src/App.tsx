import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Dashboard } from './components/Dashboard'
import { SettingsPage } from './components/SettingsPage'
import { ServerEditor } from './components/ServerEditor'
import { ToastViewport } from './components/ToastViewport'
import { mapConfigToWorkspaceView, type FeedbackItem } from './view-models/workspace'
import './i18n'
import type { MCPConfig, MCPServer, SupportedApp } from './types/config'
import { applyConfig, importDetectedConfigs, loadConfig, rollback, saveConfig } from './services/configService'
import { evaluateApplyRisks } from './services/risk'
import { checkForUpdatesAndPrompt } from './services/updater'

type View = 'dashboard' | 'editor' | 'settings'
type ThemeMode = 'light' | 'dark' | 'system'
type ActionState =
  | 'idle'
  | 'loading'
  | 'saving'
  | 'importing'
  | 'applying'
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
  const [theme, setTheme] = useState<ThemeMode>(readThemePreference)
  const [systemTheme, setSystemTheme] = useState<'light' | 'dark'>(readSystemTheme)
  const [platform] = useState(detectPlatform)
  const [view, setView] = useState<View>('dashboard')
  const [actionState, setActionState] = useState<ActionState>('loading')
  const [config, setConfig] = useState<MCPConfig>({ version: 1, servers: [] })
  const [savedConfigSnapshot, setSavedConfigSnapshot] = useState<MCPConfig>({ version: 1, servers: [] })
  const [editingServer, setEditingServer] = useState<MCPServer | null>(null)
  const [feedbacks, setFeedbacks] = useState<FeedbackItem[]>([])
  const [unsaved, setUnsaved] = useState(false)
  const [backups, setBackups] = useState<string[]>([])
  const [lastRiskSummary, setLastRiskSummary] = useState<string[]>([])

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

    const bootstrap = async () => {
      setActionState('loading')
      try {
        const loaded = await loadConfig()
        if (!alive) {
          return
        }
        setConfig(loaded)
        setSavedConfigSnapshot(loaded)
        setFeedbacks([])
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
  }, [t])

  const pushFeedback = (kind: FeedbackItem['kind'], message: string) => {
    setFeedbacks((current) => [
      { id: `${Date.now()}-${Math.random().toString(36).slice(2)}`, kind, message },
      ...current,
    ].slice(0, 4))
  }

  const dismissFeedback = (id: string) => {
    setFeedbacks((current) => current.filter((item) => item.id !== id))
  }

  const workspace = useMemo(() => mapConfigToWorkspaceView(config, t), [config, t])

  const openCreate = () => {
    setEditingServer(null)
    setView('editor')
  }

  const openEdit = (serverId: string) => {
    const server = config.servers.find((item) => item.id === serverId)
    if (!server) {
      return
    }
    setEditingServer(server)
    setView('editor')
  }

  const handleDelete = (serverId: string) => {
    const row = workspace.rows.find((item) => item.id === serverId)
    if (!row) {
      return
    }

    const confirmed = window.confirm(t('deleteConfirm', { name: row.name }))
    if (!confirmed) {
      return
    }

    setConfig((current) => ({
      ...current,
      servers: current.servers.filter((server) => server.id !== serverId),
    }))
    setUnsaved(true)
    pushFeedback('warning', t('serverDeleted', { name: row.name }))
  }

  const handleToggleApp = (serverId: string, app: SupportedApp) => {
    setConfig((current) => ({
      ...current,
      servers: current.servers.map((server) =>
        server.id === serverId
          ? { ...server, apps: { ...server.apps, [app]: !server.apps[app] } }
          : server,
      ),
    }))
    setUnsaved(true)
  }

  const handlePersist = async () => {
    setActionState('saving')
    try {
      await saveConfig(config)
      setUnsaved(false)
      setSavedConfigSnapshot(config)
      pushFeedback('success', t('saveSuccess'))
    } catch (error) {
      pushFeedback('error', t('saveFailedDetail', { error: String(error) }))
    } finally {
      setActionState('idle')
    }
  }

  const handleImport = async () => {
    setActionState('importing')
    try {
      const result = await importDetectedConfigs()
      const detectedCount = result.sources.filter((source) => source.exists).length

      if (result.config.servers.length === 0) {
        pushFeedback(
          result.errors.length > 0 ? 'error' : 'info',
          result.errors.length > 0
            ? `${t('importFailed')}: ${result.errors.join(' | ')}`
            : t('importEmpty'),
        )
        return
      }

      setConfig(result.config)
      setUnsaved(true)
      setLastRiskSummary([])

      pushFeedback(
        'success',
        t('importSummary', {
          count: detectedCount,
          servers: result.config.servers.length,
        }),
      )

      for (const warning of result.warnings) {
        pushFeedback('warning', warning)
      }
      for (const error of result.errors) {
        pushFeedback('error', error)
      }
    } catch (error) {
      pushFeedback('error', t('importFailedDetail', { error: String(error) }))
    } finally {
      setActionState('idle')
    }
  }

  const handleApply = async () => {
    const summary = evaluateApplyRisks(config.servers)
    const riskLines = [...summary.warnings, ...summary.blockingErrors]
    setLastRiskSummary(riskLines)

    if (summary.blockingErrors.length > 0) {
      for (const error of summary.blockingErrors) {
        pushFeedback('error', error)
      }
      return
    }

    if (summary.warnings.length > 0) {
      const confirmed = window.confirm(summary.warnings.join('\n'))
      if (!confirmed) {
        return
      }
    }

    setActionState('applying')
    try {
      const result = await applyConfig(config)
      await saveConfig(config)
      setBackups(result.backups)
      setUnsaved(false)
      setSavedConfigSnapshot(config)
      pushFeedback('success', t('applySuccess'))
    } catch (error) {
      pushFeedback('error', t('applyFailedDetail', { error: String(error) }))
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

  const handleSaveServer = (server: MCPServer) => {
    setConfig((current) => {
      const editingId = editingServer?.id
      const exists = current.servers.some((item) => item.id === (editingId ?? server.id))
      return {
        ...current,
        servers: exists
          ? current.servers.map((item) => (item.id === (editingId ?? server.id) ? server : item))
          : [...current.servers, server],
      }
    })
    setUnsaved(true)
    setView('dashboard')
    setEditingServer(null)
    pushFeedback('success', t(editingServer ? 'serverUpdated' : 'serverCreated', { name: server.name }))
  }

  const handleReset = () => {
    setConfig(savedConfigSnapshot)
    setUnsaved(false)
    setLastRiskSummary([])
  }

  const isBusy = actionState !== 'idle'

  return (
    <>
      <ToastViewport items={feedbacks} onDismiss={dismissFeedback} />
      {view === 'editor' ? (
        <ServerEditor
          server={editingServer}
          busy={isBusy}
          onSave={handleSaveServer}
          onCancel={() => {
            setView('dashboard')
            setEditingServer(null)
          }}
        />
      ) : view === 'settings' ? (
        <SettingsPage
          busy={isBusy}
          language={i18n.language}
          theme={theme}
          onBack={() => setView('dashboard')}
          onCheckUpdates={() => void handleCheckUpdates()}
          onLanguageChange={(language) => void i18n.changeLanguage(language)}
          onThemeChange={setTheme}
        />
      ) : (
        <Dashboard
          workspace={workspace}
          unsaved={unsaved}
          lastRiskSummary={lastRiskSummary}
          busy={actionState}
          canRollback={backups.length > 0}
          onSyncLocalConfig={() => void handleImport()}
          onOpenSettings={() => setView('settings')}
          onReset={handleReset}
          onCopyCommand={(serverId) => void handleCopyCommand(serverId)}
          onAdd={openCreate}
          onEdit={openEdit}
          onDelete={handleDelete}
          onToggleApp={handleToggleApp}
          onSave={() => void handlePersist()}
          onApply={() => void handleApply()}
          onRollback={() => void handleRollback()}
        />
      )}
    </>
  )
}

import { Copy, LoaderCircle, PenSquare, Plus, RefreshCw, RotateCcw, SendHorizontal, Settings, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { CLIENTS } from './clientMeta'
import { AppLogo } from './AppLogo'
import { Tooltip } from './Tooltip'
import type { WorkspaceViewModel } from '../view-models/workspace'
import type { SupportedApp } from '../types/config'

interface DashboardProps {
  busy: 'idle' | 'loading' | 'saving' | 'importing' | 'applying' | 'rolling-back' | 'checking-updates'
  canRollback: boolean
  lastRiskSummary: string[]
  unsaved: boolean
  workspace: WorkspaceViewModel
  onAdd: () => void
  onApply: () => void
  onSyncLocalConfig: () => void
  onOpenSettings: () => void
  onReset: () => void
  onDelete: (serverId: string) => void
  onEdit: (serverId: string) => void
  onRollback: () => void
  onSave: () => void
  onToggleApp: (serverId: string, app: SupportedApp) => void
  onCopyCommand: (serverId: string) => void
}

function BusyIcon({ spinning }: { spinning: boolean }) {
  return <LoaderCircle size={14} className={spinning ? 'spin' : ''} />
}

export function Dashboard({
  busy,
  canRollback,
  lastRiskSummary,
  unsaved,
  workspace,
  onAdd,
  onApply,
  onSyncLocalConfig,
  onOpenSettings,
  onReset,
  onDelete,
  onEdit,
  onRollback,
  onSave,
  onToggleApp,
  onCopyCommand,
}: DashboardProps) {
  const { t } = useTranslation()
  const loading = busy === 'loading'

  return (
    <div className="app-shell shell-flat dashboard-shell">
      <div className="mac-window-drag-region" data-tauri-drag-region aria-hidden="true" />
      <div className="top-chrome">
        <div className="page-header">
          <div className="brand-block brand-block-compact">
            <AppLogo className="brand-logo" alt={t('title')} />
            <div>
              <h1 className="shell-title">{t('title')}</h1>
            </div>
          </div>

          <div className="header-controls" data-tauri-no-drag>
            <Tooltip content={t('settings')}>
              <button type="button" className="icon-button toolbar-icon" onClick={onOpenSettings} aria-label={t('settings')}>
                <Settings size={14} />
              </button>
            </Tooltip>

            <Tooltip content={t('syncLocalConfig')}>
              <button
                type="button"
                className="icon-button toolbar-icon"
                onClick={onSyncLocalConfig}
                disabled={busy !== 'idle' && busy !== 'importing'}
                aria-label={t('syncLocalConfig')}
              >
                {busy === 'importing' ? <BusyIcon spinning /> : <RefreshCw size={14} />}
              </button>
            </Tooltip>

            <Tooltip content={t('rollback')}>
              <button
                type="button"
                className="icon-button toolbar-icon"
                onClick={onRollback}
                disabled={!canRollback || busy !== 'idle' && busy !== 'rolling-back'}
                aria-label={t('rollback')}
              >
                {busy === 'rolling-back' ? <BusyIcon spinning /> : <RotateCcw size={14} />}
              </button>
            </Tooltip>

            <Tooltip content={t('apply')}>
              <button
                type="button"
                className="icon-button toolbar-icon toolbar-accent"
                onClick={onApply}
                disabled={busy !== 'idle' && busy !== 'applying'}
                aria-label={t('apply')}
              >
                {busy === 'applying' ? <BusyIcon spinning /> : <SendHorizontal size={14} />}
              </button>
            </Tooltip>

            <Tooltip content={t('add')}>
              <button
                type="button"
                className="icon-button toolbar-icon toolbar-accent"
                onClick={onAdd}
                disabled={busy !== 'idle'}
                aria-label={t('add')}
              >
                <Plus size={14} />
              </button>
            </Tooltip>
          </div>
        </div>

        <section className="stats-strip">
          {workspace.stats.map((stat) => (
            <article key={stat.id} className={`stat-card ${stat.accent}`}>
              <div className="stat-topline">
                <span className="stat-icon-wrap">{stat.icon}</span>
              </div>
              <strong className="stat-value">{stat.count}</strong>
              <span className="stat-caption">{stat.label}</span>
            </article>
          ))}
        </section>
      </div>

      {unsaved ? <div className="callout callout-warning">{t('unsaved')}</div> : null}
      {lastRiskSummary.length > 0 ? <div className="callout callout-info">{`${t('riskSummary')}: ${lastRiskSummary.join(' | ')}`}</div> : null}

      <section className="list-panel">
        {loading ? (
          <div className="empty-state">
            <LoaderCircle size={24} className="spin" />
            <h3>{t('loading')}</h3>
            <p>{t('loadingWorkspace')}</p>
          </div>
        ) : workspace.rows.length === 0 ? (
          <div className="empty-state">
            <Plus size={24} />
            <h3>{t('emptyTitle')}</h3>
            <p>{t('emptyDescription')}</p>
            <button type="button" className="ghost-button" onClick={onAdd}>
              {t('add')}
            </button>
          </div>
        ) : (
          <div className="server-list-scroll">
            {workspace.rows.map((row) => (
              <article key={row.id} className="server-list-row">
                <div className="server-cell server-cell-name">
                  <strong>{row.name}</strong>
                </div>
                <div className="server-cell">
                  <span className="transport-pill">{row.transportLabel}</span>
                </div>
                <div className="server-cell">
                  <div className="client-pills client-pills-reference">
                    {CLIENTS.map((client) => {
                      const enabled = row.enabledApps.includes(client.id)
                      return (
                        <Tooltip key={client.id} content={client.label}>
                          <button
                            type="button"
                            className={`client-pill client-pill-reference ${client.accent} ${enabled ? 'is-enabled' : 'is-muted'}`}
                            onClick={() => onToggleApp(row.id, client.id)}
                          >
                            {client.icon}
                            <span className="sr-only">{client.label}</span>
                          </button>
                        </Tooltip>
                      )
                    })}
                  </div>
                </div>
                <div className="row-actions">
                  <Tooltip content={t('copyCommand')}>
                    <button type="button" className="icon-button compact-icon" onClick={() => onCopyCommand(row.id)} aria-label={t('copyCommand')}>
                      <Copy size={14} />
                    </button>
                  </Tooltip>
                  <Tooltip content={t('edit')}>
                    <button type="button" className="icon-button compact-icon" onClick={() => onEdit(row.id)} aria-label={t('edit')}>
                      <PenSquare size={14} />
                    </button>
                  </Tooltip>
                  <Tooltip content={t('delete')}>
                    <button type="button" className="icon-button compact-icon danger" onClick={() => onDelete(row.id)} aria-label={t('delete')}>
                      <Trash2 size={14} />
                    </button>
                  </Tooltip>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

      {unsaved ? (
        <div className="dashboard-floating-save-wrap">
          <button
            type="button"
            className="ghost-button dashboard-floating-reset"
            onClick={onReset}
            disabled={busy !== 'idle'}
          >
            {t('reset')}
          </button>
          <button
            type="button"
            className="dashboard-floating-save-button"
            onClick={onSave}
            disabled={busy !== 'idle' && busy !== 'saving'}
          >
            {busy === 'saving' ? <BusyIcon spinning /> : null}
            {t('saveChanges')}
          </button>
        </div>
      ) : null}
    </div>
  )
}

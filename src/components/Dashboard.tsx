import { useEffect, useRef, useState } from 'react'
import { Copy, LoaderCircle, PenSquare, Plus, RefreshCw, RotateCcw, Settings, Store, Trash2, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { getVisibleClients } from './clientMeta'
import { AppLogo } from './AppLogo'
import { Tooltip } from './Tooltip'
import type { WorkspaceViewModel } from '../view-models/workspace'
import type { SupportedApp } from '../types/config'

interface DashboardProps {
  busy: 'idle' | 'loading' | 'saving' | 'importing' | 'rolling-back' | 'checking-updates'
  canRollback: boolean
  visibleApps: SupportedApp[]
  workspace: WorkspaceViewModel
  onAdd: () => void
  /** Omitted when the online marketplace is turned off. */
  onOpenMarketplace?: () => void
  onOpenRepository: () => void
  onSyncLocalConfig: () => void
  onOpenSettings: () => void
  onDelete: (serverId: string) => void
  onEdit: (serverId: string) => void
  onRollback: () => void
  onToggleApp: (serverId: string, app: SupportedApp) => void
  onBatchSetApp: (serverIds: string[], app: SupportedApp, enabled: boolean) => void
  onCopyCommand: (serverId: string) => void
}

function SelectAllCheckbox({
  checked,
  indeterminate,
  label,
  disabled,
  onChange,
}: {
  checked: boolean
  disabled: boolean
  indeterminate: boolean
  label: string
  onChange: () => void
}) {
  const ref = useRef<HTMLInputElement | null>(null)

  useEffect(() => {
    if (ref.current) {
      ref.current.indeterminate = indeterminate
    }
  }, [indeterminate])

  return (
    <input
      ref={ref}
      type="checkbox"
      className="row-checkbox"
      checked={checked}
      disabled={disabled}
      aria-label={label}
      onChange={onChange}
    />
  )
}

function BusyIcon({ spinning }: { spinning: boolean }) {
  return <LoaderCircle size={14} className={spinning ? 'spin' : ''} />
}

export function Dashboard({
  busy,
  canRollback,
  visibleApps,
  workspace,
  onAdd,
  onOpenMarketplace,
  onOpenRepository,
  onSyncLocalConfig,
  onOpenSettings,
  onDelete,
  onEdit,
  onRollback,
  onToggleApp,
  onBatchSetApp,
  onCopyCommand,
}: DashboardProps) {
  const { t } = useTranslation()
  const loading = busy === 'loading'
  const visibleClients = getVisibleClients(visibleApps)
  const [selectedIds, setSelectedIds] = useState<string[]>([])
  const rowIds = workspace.rows.map((row) => row.id)
  // Drop selections for servers that no longer exist (deleted or replaced).
  const selected = selectedIds.filter((id) => rowIds.includes(id))
  const selectedRows = workspace.rows.filter((row) => selected.includes(row.id))
  const allSelected = rowIds.length > 0 && selected.length === rowIds.length

  const toggleSelected = (serverId: string) => {
    setSelectedIds((current) =>
      current.includes(serverId) ? current.filter((id) => id !== serverId) : [...current, serverId],
    )
  }

  return (
    <div className="app-shell shell-flat dashboard-shell">
      <div className="mac-window-drag-region" data-tauri-drag-region aria-hidden="true" />
      <div className="top-chrome">
        <div className="page-header page-header-dashboard">
          <div className="brand-block brand-block-dashboard">
            <button
              type="button"
              className="brand-logo-button"
              onClick={onOpenRepository}
              aria-label={t('settingsRepositoryAction')}
              title={t('settingsRepositoryAction')}
            >
              <AppLogo className="brand-logo" alt={t('title')} />
            </button>
            <div className="brand-copy">
              <h1 className="shell-title">{t('title')}</h1>
              <p className="brand-subtitle">{t('surfaceSubtitle')}</p>
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

            {onOpenMarketplace ? (
              <Tooltip content={t('marketplace')}>
                <button
                  type="button"
                  className="icon-button toolbar-icon"
                  onClick={onOpenMarketplace}
                  aria-label={t('marketplace')}
                >
                  <Store size={14} />
                </button>
              </Tooltip>
            ) : null}

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
            <div className="empty-state-actions">
              <button type="button" className="ghost-button" onClick={onAdd}>
                {t('add')}
              </button>
              {onOpenMarketplace ? (
                <button type="button" className="ghost-button" onClick={onOpenMarketplace}>
                  <Store size={14} />
                  {t('marketplaceOpen')}
                </button>
              ) : null}
            </div>
          </div>
        ) : (
          <>
          <div className="batch-bar" data-tauri-no-drag>
            <label className="batch-select-all">
              <SelectAllCheckbox
                checked={allSelected}
                indeterminate={selected.length > 0 && !allSelected}
                disabled={busy !== 'idle'}
                label={t('batchSelectAll')}
                onChange={() => setSelectedIds(allSelected ? [] : rowIds)}
              />
              <span>{selected.length > 0 ? t('batchSelected', { count: selected.length }) : t('batchSelectAll')}</span>
            </label>
            {selected.length > 0 ? (
              <>
                <span className="batch-bar-label">{t('batchApplyTo')}</span>
                <div className="client-pills client-pills-reference">
                  {visibleClients.map((client) => {
                    const enabledCount = selectedRows.filter((row) => row.enabledApps.includes(client.id)).length
                    const allEnabled = enabledCount === selectedRows.length
                    const state = allEnabled ? 'is-enabled' : enabledCount > 0 ? 'is-partial' : 'is-muted'
                    const tooltip = t(allEnabled ? 'batchDisableFor' : 'batchEnableFor', {
                      client: client.label,
                      count: selected.length,
                    })
                    return (
                      <Tooltip key={client.id} content={tooltip}>
                        <button
                          type="button"
                          className={`client-pill client-pill-reference ${client.accent} ${state}`}
                          onClick={() => onBatchSetApp(selected, client.id, !allEnabled)}
                          disabled={busy !== 'idle'}
                        >
                          {client.icon}
                          <span className="sr-only">{tooltip}</span>
                        </button>
                      </Tooltip>
                    )
                  })}
                </div>
                <Tooltip content={t('batchClear')}>
                  <button
                    type="button"
                    className="icon-button compact-icon batch-clear"
                    onClick={() => setSelectedIds([])}
                    aria-label={t('batchClear')}
                  >
                    <X size={14} />
                  </button>
                </Tooltip>
              </>
            ) : null}
          </div>
          <div className="server-list-scroll">
            {workspace.rows.map((row) => (
              <article
                key={row.id}
                className={selected.includes(row.id) ? 'server-list-row is-selected' : 'server-list-row'}
                onClick={() => {
                  if (busy === 'idle') {
                    onEdit(row.id)
                  }
                }}
              >
                <div className="server-cell server-cell-select" onClick={(event) => event.stopPropagation()}>
                  <input
                    type="checkbox"
                    className="row-checkbox"
                    checked={selected.includes(row.id)}
                    disabled={busy !== 'idle'}
                    aria-label={t('batchSelectRow', { name: row.name })}
                    onChange={() => toggleSelected(row.id)}
                  />
                </div>
                <div className="server-cell server-cell-name">
                  <strong>{row.name}</strong>
                </div>
                <div className="server-cell">
                  <span className="transport-pill">{row.transportLabel}</span>
                </div>
                <div className="server-cell">
                  <div className="client-pills client-pills-reference">
                    {visibleClients.map((client) => {
                      const enabled = row.enabledApps.includes(client.id)
                      return (
                        <Tooltip key={client.id} content={client.label}>
                          <button
                            type="button"
                            className={`client-pill client-pill-reference ${client.accent} ${enabled ? 'is-enabled' : 'is-muted'}`}
                            onClick={(event) => {
                              event.stopPropagation()
                              onToggleApp(row.id, client.id)
                            }}
                            disabled={busy !== 'idle'}
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
                    <button
                      type="button"
                      className="icon-button compact-icon server-row-action"
                      onClick={(event) => {
                        event.stopPropagation()
                        onCopyCommand(row.id)
                      }}
                      aria-label={t('copyCommand')}
                      disabled={busy !== 'idle'}
                    >
                      <Copy size={14} />
                    </button>
                  </Tooltip>
                  <Tooltip content={t('edit')}>
                    <button
                      type="button"
                      className="icon-button compact-icon server-row-action"
                      onClick={(event) => {
                        event.stopPropagation()
                        onEdit(row.id)
                      }}
                      aria-label={t('edit')}
                      disabled={busy !== 'idle'}
                    >
                      <PenSquare size={14} />
                    </button>
                  </Tooltip>
                  <Tooltip content={t('delete')}>
                    <button
                      type="button"
                      className="icon-button compact-icon server-row-action server-row-action-danger"
                      onClick={(event) => {
                        event.stopPropagation()
                        onDelete(row.id)
                      }}
                      aria-label={t('delete')}
                      disabled={busy !== 'idle'}
                    >
                      <Trash2 size={14} />
                    </button>
                  </Tooltip>
                </div>
              </article>
            ))}
          </div>
          </>
        )}
      </section>
    </div>
  )
}

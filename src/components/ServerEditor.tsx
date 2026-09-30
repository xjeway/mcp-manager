import type { ReactNode } from 'react'
import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { ArrowLeft, Check, Code, Eye, EyeOff, FileUp, FolderOpen, LayoutTemplate, Plus, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { getVisibleClients } from './clientMeta'
import { JsonEditor } from './JsonEditor'
import { Segmented } from './Segmented'
import { Tabs, tabId, tabPanelId } from './Tabs'
import { Tooltip } from './Tooltip'
import { parseMcpJson } from '../services/jsonParser'
import { openPath } from '../services/externalLinks'
import type { MCPServer, SupportedApp, WorkspaceContext } from '../types/config'
import { getProjectConfigurableApps } from '../view-models/clientCapabilities'
import {
  createEmptyEditorDraft,
  editorDraftToServer,
  serverToEditorDraft,
  isUserScopeEnabled,
  serverToJsonText,
  setServerAppEnabled,
  setWorkspacePlacement,
  workspacePlacementPath,
  type EditorDraft,
  type EditorMode,
} from '../view-models/workspace'

interface ServerEditorProps {
  busy: boolean
  server: MCPServer | null
  /** Prefills a new server (e.g. from the marketplace); ignored when `server` is set. */
  initialServer?: MCPServer | null
  workspace: WorkspaceContext
  onCancel: () => void
  onDraftChange: (draft: EditorDraft, dirty: boolean) => void
  onSave: (server: MCPServer) => void
  onSaveMany: (servers: MCPServer[]) => void
  visibleApps: Array<keyof MCPServer['apps']>
}

function normalizeEntries(entries: EditorDraft['envEntries']): EditorDraft['envEntries'] {
  return entries.length > 0 ? entries : [{ key: '', value: '' }]
}

function Switch({
  checked,
  compact = false,
  label,
  onChange,
}: {
  checked: boolean
  compact?: boolean
  label?: string
  onChange: (value: boolean) => void
}) {
  const classes = ['switch-control', checked ? 'checked' : '', compact ? 'settings-switch-compact' : '']
  return (
    <button
      type="button"
      className={classes.filter(Boolean).join(' ')}
      onClick={() => onChange(!checked)}
      aria-pressed={checked}
      aria-label={label}
    >
      <span className="switch-thumb" />
    </button>
  )
}

function AutoGrowTextarea({
  className,
  value,
  onChange,
}: {
  className: string
  value: string
  onChange: (value: string) => void
}) {
  const ref = useRef<HTMLTextAreaElement | null>(null)

  useLayoutEffect(() => {
    const element = ref.current
    if (!element) {
      return
    }
    element.style.height = 'auto'
    element.style.height = `${element.scrollHeight}px`
  }, [value])

  return (
    <textarea ref={ref} className={className} rows={2} value={value} onChange={(event) => onChange(event.target.value)} />
  )
}

function ClientTag({
  client,
  enabled,
  title,
  onToggle,
}: {
  client: ReturnType<typeof getVisibleClients>[number]
  enabled: boolean
  title?: string
  onToggle: () => void
}) {
  return (
    <label
      title={title}
      className={enabled ? `editor-reference-client-tag ${client.accent} is-enabled` : `editor-reference-client-tag ${client.accent}`}
    >
      <input type="checkbox" className="sr-only" checked={enabled} onChange={onToggle} />
      <span className={`client-pill client-pill-reference ${client.accent} ${enabled ? 'is-enabled' : 'is-muted'}`}>
        {client.icon}
      </span>
      <span>{client.label}</span>
    </label>
  )
}

export function ServerEditor({
  busy,
  server,
  initialServer = null,
  workspace,
  onCancel,
  onDraftChange,
  onSave,
  onSaveMany,
  visibleApps,
}: ServerEditorProps) {
  const { t } = useTranslation()
  const [mode, setMode] = useState<EditorMode>('form')
  const seed = server ?? initialServer
  const [draft, setDraft] = useState<EditorDraft>(() => serverToEditorDraft(seed))
  const [jsonText, setJsonText] = useState(() => serverToJsonText(seed))
  const [warnings, setWarnings] = useState<string[]>([])
  const [errors, setErrors] = useState<string[]>([])
  const [openPathError, setOpenPathError] = useState<string | null>(null)
  // Servers parsed from a multi-server JSON while creating; saved together.
  const [batchServers, setBatchServers] = useState<MCPServer[]>([])
  const fileInputRef = useRef<HTMLInputElement | null>(null)
  // Row that was just added, so its input can take focus.
  const [pendingFocus, setPendingFocus] = useState<{ args?: number; env?: number; header?: number }>({})
  // Header values are secrets: masked unless the user reveals a row.
  const [revealedHeaders, setRevealedHeaders] = useState<ReadonlySet<number>>(new Set())
  const visibleClients = getVisibleClients(visibleApps)
  const projectConfigurableApps = getProjectConfigurableApps()
  const projectConfigurableClients = visibleClients.filter((client) => projectConfigurableApps.includes(client.id))
  const hasWorkspace = workspace.root.trim() !== ''
  const workspaceName = workspace.root.split(/[\\/]/).filter(Boolean).pop() ?? workspace.root
  const relativeToWorkspace = (path: string) =>
    path.startsWith(workspace.root) ? path.slice(workspace.root.length).replace(/^[\\/]/, '') : path

  const currentWorkspacePlacementForApp = (app: SupportedApp) => {
    const path = workspacePlacementPath(app, workspace)
    if (!path) {
      return null
    }
    return draft.placements.find(
      (placement) => placement.app === app && placement.scope === 'workspace' && placement.path === path,
    )
  }

  const setCurrentWorkspacePlacementEnabled = (app: SupportedApp, enabled: boolean) => {
    const path = workspacePlacementPath(app, workspace)
    if (!path) {
      return
    }

    setDraft((current) => ({
      ...current,
      placements: setWorkspacePlacement(current.placements, app, path, enabled),
    }))
  }

  const handleOpenPath = async (path: string) => {
    setOpenPathError(null)
    try {
      await openPath(path)
    } catch (error) {
      setOpenPathError(t('openPathFailed', { error: String(error) }))
    }
  }

  useEffect(() => {
    const nextDraft = seed ? serverToEditorDraft(seed) : createEmptyEditorDraft()
    setDraft(nextDraft)
    setJsonText(seed ? serverToJsonText(seed) : '')
    setWarnings([])
    setErrors([])
    setBatchServers([])
    setRevealedHeaders(new Set())
    setMode('form')
  }, [seed])

  const jsonPlaceholder = serverToJsonText(null)

  const isDraftEffectivelyEmpty =
    !draft.name.trim() &&
    !draft.id.trim() &&
    !draft.description.trim() &&
    !draft.homepage.trim() &&
    !draft.program.trim() &&
    !draft.url.trim() &&
    draft.args.every((arg) => !arg.trim()) &&
    draft.envEntries.every((entry) => !entry.key.trim() && !entry.value.trim()) &&
    draft.headerEntries.every((entry) => !entry.key.trim() && !entry.value.trim())

  const initialSerialized = server ? JSON.stringify(editorDraftToServer(serverToEditorDraft(server))) : ''
  const currentSerialized = !server && isDraftEffectivelyEmpty ? '' : JSON.stringify(editorDraftToServer(draft))
  const isDirty = currentSerialized !== initialSerialized || batchServers.length > 0

  useEffect(() => {
    onDraftChange(draft, isDirty)
  }, [draft, isDirty, onDraftChange])

  const syncJsonFromDraft = () => {
    try {
      if (!server && isDraftEffectivelyEmpty) {
        setJsonText('')
        return
      }
      setJsonText(serverToJsonText(editorDraftToServer(draft)))
    } catch {
      // Wait until the form becomes valid enough to serialize.
    }
  }

  const parseJsonText = (text: string) => {
    const result = parseMcpJson(text)
    const nextWarnings = [...result.warnings.map((item) => item.message)]
    const nextErrors = [...result.errors.map((item) => item.message)]

    // Creating from several servers imports them all; editing one keeps just the first.
    const isBatch = !server && result.servers.length > 1
    if (result.servers.length > 1 && server) {
      nextWarnings.unshift(t('jsonMultiServerHint'))
    }

    if (isBatch) {
      setBatchServers(result.servers)
    } else {
      setBatchServers([])
      if (result.servers.length > 0) {
        // Keep the client selection the user already made while pasting a server.
        const parsed = serverToEditorDraft(result.servers[0])
        setDraft((current) => (server ? parsed : { ...parsed, apps: current.apps, placements: current.placements }))
      }
    }

    setWarnings(nextWarnings)
    setErrors(nextErrors)
  }

  const handleParseJson = () => parseJsonText(jsonText)

  const handleJsonTextChange = (value: string) => {
    setJsonText(value)
    setBatchServers([])
  }

  const handleImportFile = async (file: File | undefined) => {
    if (!file) {
      return
    }
    try {
      const text = await file.text()
      setJsonText(text)
      parseJsonText(text)
    } catch (error) {
      setWarnings([])
      setErrors([t('jsonFileReadFailed', { error: String(error) })])
    }
  }

  const addArg = () => {
    setPendingFocus({ args: draft.args.length })
    setDraft((current) => ({ ...current, args: [...current.args, ''] }))
  }

  const updateArg = (index: number, value: string) => {
    setDraft((current) => {
      const next = [...current.args]
      next[index] = value
      return { ...current, args: next }
    })
  }

  const removeArg = (index: number) => {
    setDraft((current) => ({
      ...current,
      args: current.args.filter((_, position) => position !== index),
    }))
  }

  const addEnv = () => {
    setPendingFocus({ env: draft.envEntries.length })
    setDraft((current) => ({
      ...current,
      envEntries: [...current.envEntries, { key: '', value: '' }],
    }))
  }

  const updateEnv = (index: number, patch: Partial<EditorDraft['envEntries'][number]>) => {
    setDraft((current) => {
      const next = [...normalizeEntries(current.envEntries)]
      next[index] = { ...next[index], ...patch }
      return { ...current, envEntries: next }
    })
  }

  const removeEnv = (index: number) => {
    setDraft((current) => ({
      ...current,
      envEntries: current.envEntries.filter((_, position) => position !== index),
    }))
  }

  const addHeader = () => {
    setPendingFocus({ header: draft.headerEntries.length })
    setDraft((current) => ({
      ...current,
      headerEntries: [...current.headerEntries, { key: '', value: '' }],
    }))
  }

  const updateHeader = (index: number, patch: Partial<EditorDraft['headerEntries'][number]>) => {
    setDraft((current) => {
      const next = [...current.headerEntries]
      next[index] = { ...next[index], ...patch }
      return { ...current, headerEntries: next }
    })
  }

  const removeHeader = (index: number) => {
    setRevealedHeaders(new Set())
    setDraft((current) => ({
      ...current,
      headerEntries: current.headerEntries.filter((_, position) => position !== index),
    }))
  }

  const toggleHeaderReveal = (index: number) => {
    setRevealedHeaders((current) => {
      const next = new Set(current)
      if (!next.delete(index)) {
        next.add(index)
      }
      return next
    })
  }

  const submit = () => {
    if (batchServers.length > 0) {
      // Every imported server gets the clients chosen above.
      onSaveMany(
        batchServers.map((item) => ({
          ...item,
          enabled: item.enabled && draft.enabled,
          apps: { ...draft.apps },
          placements: draft.placements.filter((placement) => placement.enabled),
        })),
      )
      return
    }
    onSave(editorDraftToServer(draft))
  }

  return (
    <div className="app-shell shell-flat editor-shell editor-page-reference">
      <div className="mac-window-drag-region" data-tauri-drag-region aria-hidden="true" />
      <div className="editor-reference-header">
        <div className="editor-reference-title">
          <button type="button" className="editor-back-button" onClick={onCancel} aria-label={t('back')}>
            <ArrowLeft size={18} />
          </button>
          <div>
            <h1>{server ? t('edit') : t('add')}</h1>
            <p>{t('editorSubtitle')}</p>
          </div>
        </div>

        <div className="editor-reference-toolbar" data-tauri-no-drag>
          <button type="button" className="primary-button" onClick={submit} disabled={busy}>
            <Check size={14} />
            {batchServers.length > 0 ? t('batchImportConfirm', { count: batchServers.length }) : t('confirm')}
          </button>
        </div>
      </div>

      <div className="editor-reference-scroll">
        <div className="editor-reference-content">
          <section className="editor-reference-card">
            <div className="editor-card-header">
              <h2>{t('basicInformation')}</h2>
              <label className="editor-card-header-switch">
                <span>{t('enabled')}</span>
                <Switch
                  compact
                  label={t('enabled')}
                  checked={draft.enabled}
                  onChange={(value) => setDraft((current) => ({ ...current, enabled: value }))}
                />
              </label>
            </div>

            <div className="editor-card-body">
              <div className="editor-field-grid">
                <label className="editor-field">
                  <span className="editor-field-label">{t('name')}</span>
                  <input
                    className="editor-input"
                    value={draft.name}
                    onChange={(event) => setDraft((current) => ({ ...current, name: event.target.value }))}
                  />
                </label>

                <label className="editor-field">
                  <span className="editor-field-label">{t('id')}</span>
                  <input
                    className="editor-input editor-input-mono"
                    spellCheck={false}
                    value={draft.id}
                    onChange={(event) => setDraft((current) => ({ ...current, id: event.target.value }))}
                  />
                </label>
              </div>

              <label className="editor-field">
                <span className="editor-field-label">{t('description')}</span>
                <AutoGrowTextarea
                  className="editor-input editor-textarea"
                  value={draft.description}
                  onChange={(value) => setDraft((current) => ({ ...current, description: value }))}
                />
              </label>

              <label className="editor-field">
                <span className="editor-field-label">{t('homepage')}</span>
                <input
                  className="editor-input"
                  placeholder="https://example.com"
                  spellCheck={false}
                  value={draft.homepage}
                  onChange={(event) => setDraft((current) => ({ ...current, homepage: event.target.value }))}
                />
              </label>

              <div className="editor-field">
                <span className="editor-field-label">{hasWorkspace ? t('appScopeGlobal') : t('appScope')}</span>
                <div className="editor-reference-client-tags">
                  {visibleClients.map((client) => (
                    <ClientTag
                      key={client.id}
                      client={client}
                      enabled={isUserScopeEnabled(draft, client.id)}
                      onToggle={() =>
                        setDraft((current) =>
                          setServerAppEnabled(current, client.id, !isUserScopeEnabled(current, client.id)),
                        )
                      }
                    />
                  ))}
                </div>
              </div>

              {hasWorkspace ? (
                <div className="editor-field">
                  <div className="editor-field-label-row">
                    <span className="editor-field-label">{t('appScopeProject')}</span>
                    <Tooltip content={workspace.root}>
                      <button
                        type="button"
                        className="project-scope-name"
                        onClick={() => void handleOpenPath(workspace.root)}
                        aria-label={t('openWorkspace')}
                      >
                        <FolderOpen size={12} />
                        <span>{workspaceName}</span>
                      </button>
                    </Tooltip>
                  </div>
                  <div className="editor-reference-client-tags">
                    {projectConfigurableClients.flatMap((client) => {
                      const path = workspacePlacementPath(client.id, workspace)
                      if (!path) {
                        return []
                      }
                      const enabled = currentWorkspacePlacementForApp(client.id)?.enabled ?? false
                      return (
                        <ClientTag
                          key={client.id}
                          client={client}
                          enabled={enabled}
                          title={relativeToWorkspace(path)}
                          onToggle={() => setCurrentWorkspacePlacementEnabled(client.id, !enabled)}
                        />
                      )
                    })}
                  </div>
                  {openPathError ? <div className="feedback-card feedback-error">{openPathError}</div> : null}
                </div>
              ) : null}
            </div>
          </section>

          <section className="editor-reference-card editor-reference-config-card">
            <div className="editor-card-header">
              <h2>{t('serverConfiguration')}</h2>
              <Tabs
                ariaLabel={t('serverConfiguration')}
                className="editor-mode-tabs"
                idPrefix="editor-mode"
                value={mode}
                onChange={(value) => {
                  if (value === 'json') {
                    syncJsonFromDraft()
                  } else {
                    setBatchServers([])
                  }
                  setMode(value)
                }}
                options={[
                  { value: 'form', label: t('formMode'), icon: <LayoutTemplate size={12} /> },
                  { value: 'json', label: 'JSON', icon: <Code size={12} /> },
                ]}
              />
            </div>

            <div
              className="editor-card-body"
              role="tabpanel"
              id={tabPanelId('editor-mode')}
              aria-labelledby={tabId('editor-mode', mode)}
            >
              {mode === 'form' ? (
                <div className="editor-field editor-field-inline">
                  <span className="editor-field-label">{t('transport')}</span>
                  <Segmented
                    ariaLabel={t('transport')}
                    value={draft.transportType}
                    onChange={(value) => setDraft((current) => ({ ...current, transportType: value }))}
                    options={[
                      { value: 'stdio', label: 'STDIO' },
                      { value: 'http', label: 'HTTP / SSE' },
                    ]}
                  />
                </div>
              ) : null}

              {mode === 'form' ? (
                draft.transportType === 'stdio' ? (
                  <>
                    <label className="editor-field">
                      <span className="editor-field-label">{t('command')}</span>
                      <input
                        className="editor-input editor-input-mono"
                        placeholder="npx, node, uvx…"
                        spellCheck={false}
                        value={draft.program}
                        onChange={(event) => setDraft((current) => ({ ...current, program: event.target.value }))}
                      />
                    </label>

                    <div className="editor-field">
                      <span className="editor-field-label">{t('args')}</span>
                      <div className="config-list">
                        {draft.args.map((value, index) => (
                          <div key={index} className="config-list-row">
                            <input
                              className="config-list-input"
                              aria-label={`${t('args')} ${index + 1}`}
                              autoFocus={index === pendingFocus.args}
                              spellCheck={false}
                              value={value}
                              onChange={(event) => updateArg(index, event.target.value)}
                            />
                            <button type="button" className="config-list-remove" onClick={() => removeArg(index)} aria-label={t('delete')}>
                              <X size={13} />
                            </button>
                          </div>
                        ))}
                        <button type="button" className="config-list-add" onClick={addArg}>
                          <Plus size={12} />
                          {t('addArg')}
                        </button>
                      </div>
                    </div>

                    <div className="editor-field">
                      <span className="editor-field-label">{t('environmentVariables')}</span>
                      <div className="config-list">
                        {draft.envEntries.map((entry, index) => (
                          <div key={index} className="config-list-row config-list-row-env">
                            <input
                              className="config-list-input config-list-input-key"
                              placeholder={t('envKey')}
                              aria-label={t('envKey')}
                              autoFocus={index === pendingFocus.env}
                              spellCheck={false}
                              value={entry.key}
                              onChange={(event) => updateEnv(index, { key: event.target.value })}
                            />
                            <input
                              className="config-list-input"
                              placeholder={t('envValue')}
                              aria-label={t('envValue')}
                              spellCheck={false}
                              value={entry.value}
                              onChange={(event) => updateEnv(index, { value: event.target.value })}
                            />
                            <button type="button" className="config-list-remove" onClick={() => removeEnv(index)} aria-label={t('delete')}>
                              <X size={13} />
                            </button>
                          </div>
                        ))}
                        <button type="button" className="config-list-add" onClick={addEnv}>
                          <Plus size={12} />
                          {t('addEnv')}
                        </button>
                      </div>
                    </div>
                  </>
                ) : (
                  <>
                    <label className="editor-field">
                      <span className="editor-field-label">{t('url')}</span>
                      <input
                        className="editor-input editor-input-mono"
                        placeholder="https://example.com/mcp"
                        spellCheck={false}
                        value={draft.url}
                        onChange={(event) => setDraft((current) => ({ ...current, url: event.target.value }))}
                      />
                    </label>

                    <div className="editor-field">
                      <span className="editor-field-label">{t('requestHeaders')}</span>
                      <div className="config-list">
                        {draft.headerEntries.map((entry, index) => {
                          const revealed = revealedHeaders.has(index)
                          return (
                            <div key={index} className="config-list-row config-list-row-header">
                              <input
                                className="config-list-input config-list-input-key"
                                placeholder="Authorization"
                                aria-label={t('headerKey')}
                                autoFocus={index === pendingFocus.header}
                                spellCheck={false}
                                value={entry.key}
                                onChange={(event) => updateHeader(index, { key: event.target.value })}
                              />
                              <input
                                className="config-list-input"
                                type={revealed ? 'text' : 'password'}
                                placeholder={t('headerValue')}
                                aria-label={t('headerValue')}
                                autoComplete="off"
                                spellCheck={false}
                                value={entry.value}
                                onChange={(event) => updateHeader(index, { value: event.target.value })}
                              />
                              <button
                                type="button"
                                className="config-list-remove config-list-reveal"
                                onClick={() => toggleHeaderReveal(index)}
                                aria-label={revealed ? t('hideHeaderValue') : t('showHeaderValue')}
                                aria-pressed={revealed}
                              >
                                {revealed ? <EyeOff size={13} /> : <Eye size={13} />}
                              </button>
                              <button type="button" className="config-list-remove" onClick={() => removeHeader(index)} aria-label={t('delete')}>
                                <X size={13} />
                              </button>
                            </div>
                          )
                        })}
                        <button type="button" className="config-list-add" onClick={addHeader}>
                          <Plus size={12} />
                          {t('addHeader')}
                        </button>
                      </div>
                    </div>
                  </>
                )
              ) : (
                <>
                  <JsonEditor ariaLabel="JSON" value={jsonText} placeholder={jsonPlaceholder} onChange={handleJsonTextChange} />
                  <div className="config-json-footer">
                    <span>{server ? t('jsonPasteHintEdit') : t('jsonPasteHint')}</span>
                    <div className="config-json-actions">
                      <input
                        ref={fileInputRef}
                        type="file"
                        accept=".json,application/json"
                        className="sr-only"
                        tabIndex={-1}
                        onChange={(event) => {
                          void handleImportFile(event.target.files?.[0])
                          event.target.value = ''
                        }}
                      />
                      <button type="button" className="ghost-button config-json-parse" onClick={() => fileInputRef.current?.click()}>
                        <FileUp size={12} />
                        {t('jsonImportFile')}
                      </button>
                      <button type="button" className="ghost-button config-json-parse" onClick={handleParseJson}>
                        {t('parse')}
                      </button>
                    </div>
                  </div>
                  {batchServers.length > 0 ? (
                    <div className="batch-import-preview">
                      <p>{t('batchImportHint', { count: batchServers.length })}</p>
                      <ul>
                        {batchServers.map((item) => (
                          <li key={item.id}>
                            <strong>{item.name}</strong>
                            <span>{item.transport.type === 'http' ? item.transport.url : [item.command?.program, ...(item.command?.args ?? [])].join(' ')}</span>
                          </li>
                        ))}
                      </ul>
                    </div>
                  ) : null}
                </>
              )}

              {warnings.length > 0 || errors.length > 0 ? (
                <div className="feedback-stack">
                  {warnings.map((warning) => (
                    <div key={warning} className="feedback-card feedback-warning">
                      {warning}
                    </div>
                  ))}
                  {errors.map((error) => (
                    <div key={error} className="feedback-card feedback-error">
                      {error}
                    </div>
                  ))}
                </div>
              ) : null}
            </div>
          </section>
        </div>
      </div>
    </div>
  )
}

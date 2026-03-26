import type { ReactNode } from 'react'
import { useEffect, useState } from 'react'
import { ArrowLeft, Check, Code, LayoutTemplate, Plus, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { CLIENTS } from './clientMeta'
import { parseMcpJson } from '../services/jsonParser'
import type { MCPServer } from '../types/config'
import {
  createEmptyEditorDraft,
  editorDraftToServer,
  serverToEditorDraft,
  serverToJsonText,
  type EditorDraft,
  type EditorMode,
} from '../view-models/workspace'

interface ServerEditorProps {
  busy: boolean
  server: MCPServer | null
  onCancel: () => void
  onDraftChange: (draft: EditorDraft, dirty: boolean) => void
  onSave: (server: MCPServer) => void
  visibleApps: Array<keyof MCPServer['apps']>
}

function normalizeEntries(entries: EditorDraft['envEntries']): EditorDraft['envEntries'] {
  return entries.length > 0 ? entries : [{ key: '', value: '' }]
}

function Segmented({
  options,
  value,
  onChange,
}: {
  options: Array<{ icon?: ReactNode; label: string; value: string }>
  value: string
  onChange: (value: string) => void
}) {
  return (
    <div className="segment-control">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          className={value === option.value ? 'segment-button active' : 'segment-button'}
          onClick={() => onChange(option.value)}
        >
          {option.icon}
          <span>{option.label}</span>
        </button>
      ))}
    </div>
  )
}

function Switch({
  checked,
  onChange,
}: {
  checked: boolean
  onChange: (value: boolean) => void
}) {
  return (
    <button
      type="button"
      className={checked ? 'switch-control checked' : 'switch-control'}
      onClick={() => onChange(!checked)}
      aria-pressed={checked}
    >
      <span className="switch-thumb" />
    </button>
  )
}

export function ServerEditor({
  busy,
  server,
  onCancel,
  onDraftChange,
  onSave,
  visibleApps,
}: ServerEditorProps) {
  const { t } = useTranslation()
  const [mode, setMode] = useState<EditorMode>('form')
  const [draft, setDraft] = useState<EditorDraft>(() => serverToEditorDraft(server))
  const [jsonText, setJsonText] = useState(() => serverToJsonText(server))
  const [warnings, setWarnings] = useState<string[]>([])
  const [errors, setErrors] = useState<string[]>([])
  const visibleClients = CLIENTS.filter((client) => visibleApps.includes(client.id))

  useEffect(() => {
    const nextDraft = server ? serverToEditorDraft(server) : createEmptyEditorDraft()
    setDraft(nextDraft)
    setJsonText(server ? serverToJsonText(server) : '')
    setWarnings([])
    setErrors([])
    setMode('form')
  }, [server])

  const jsonPlaceholder = serverToJsonText(null)

  const isDraftEffectivelyEmpty =
    !draft.name.trim() &&
    !draft.id.trim() &&
    !draft.description.trim() &&
    !draft.homepage.trim() &&
    !draft.program.trim() &&
    !draft.url.trim() &&
    draft.args.every((arg) => !arg.trim()) &&
    draft.envEntries.every((entry) => !entry.key.trim() && !entry.value.trim())

  const initialSerialized = server ? JSON.stringify(editorDraftToServer(serverToEditorDraft(server))) : ''
  const currentSerialized = !server && isDraftEffectivelyEmpty ? '' : JSON.stringify(editorDraftToServer(draft))
  const isDirty = currentSerialized !== initialSerialized

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

  const handleParseJson = () => {
    const result = parseMcpJson(jsonText)
    const nextWarnings = [...result.warnings.map((item) => item.message)]
    const nextErrors = [...result.errors.map((item) => item.message)]

    if (result.servers.length > 1) {
      nextWarnings.unshift(t('jsonMultiServerHint'))
    }

    if (result.servers.length > 0) {
      setDraft(serverToEditorDraft(result.servers[0]))
    }

    setWarnings(nextWarnings)
    setErrors(nextErrors)
  }

  const addArg = () => {
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

  const submit = () => {
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
            {t('confirm')}
          </button>
        </div>
      </div>

      <div className="editor-reference-scroll">
        <div className="editor-reference-content">
          <section className="editor-reference-card editor-reference-card-basic">
            <h2>{t('basicInformation')}</h2>

            <div className="editor-reference-stack">
              <div className="editor-reference-grid">
                <label className="editor-reference-field">
                  <span>{t('name')}</span>
                  <input
                    value={draft.name}
                    onChange={(event) => setDraft((current) => ({ ...current, name: event.target.value }))}
                  />
                </label>

                <label className="editor-reference-field">
                  <span>{t('id')}</span>
                  <input
                    value={draft.id}
                    onChange={(event) => setDraft((current) => ({ ...current, id: event.target.value }))}
                  />
                </label>

                <div className="editor-reference-field">
                  <span>{t('enabled')}</span>
                  <div className="editor-reference-inline">
                    <Switch checked={draft.enabled} onChange={(value) => setDraft((current) => ({ ...current, enabled: value }))} />
                    <strong>{draft.enabled ? t('yes') : t('no')}</strong>
                  </div>
                </div>

                <div className="editor-reference-field">
                  <span>{t('transport')}</span>
                  <Segmented
                    value={draft.transportType}
                    onChange={(value) => setDraft((current) => ({ ...current, transportType: value as 'stdio' | 'http' }))}
                    options={[
                      { value: 'stdio', label: 'STDIO' },
                      { value: 'http', label: 'HTTP / SSE' },
                    ]}
                  />
                </div>
              </div>

              <div>
                <label className="editor-reference-label">{t('appScope')}</label>
                <div className="editor-reference-client-tags">
                  {visibleClients.map((client) => {
                    const enabled = draft.apps[client.id]
                    return (
                      <label
                        key={client.id}
                        className={enabled ? `editor-reference-client-tag ${client.accent} is-enabled` : `editor-reference-client-tag ${client.accent}`}
                      >
                        <input
                          type="checkbox"
                          className="sr-only"
                          checked={enabled}
                          onChange={() =>
                            setDraft((current) => ({
                              ...current,
                              apps: { ...current.apps, [client.id]: !current.apps[client.id] },
                            }))
                          }
                        />
                        <span className={`client-pill client-pill-reference ${client.accent} ${enabled ? 'is-enabled' : 'is-muted'}`}>
                          {client.icon}
                        </span>
                        <span>{client.label}</span>
                      </label>
                    )
                  })}
                </div>
              </div>

              <div className="editor-reference-grid">
                <label className="editor-reference-field editor-reference-field-wide">
                  <span>{t('description')}</span>
                  <textarea
                    rows={1}
                    value={draft.description}
                    onChange={(event) => setDraft((current) => ({ ...current, description: event.target.value }))}
                  />
                </label>

                <label className="editor-reference-field editor-reference-field-wide">
                  <span>{t('homepage')}</span>
                  <input
                    placeholder="https://example.com"
                    value={draft.homepage}
                    onChange={(event) => setDraft((current) => ({ ...current, homepage: event.target.value }))}
                  />
                </label>
              </div>
            </div>
          </section>

          <section className="editor-reference-card editor-reference-config-card">
            <div className="editor-reference-card-header">
              <h2>{t('serverConfiguration')}</h2>
              <Segmented
                value={mode}
                onChange={(value) => {
                  if (value === 'json') {
                    syncJsonFromDraft()
                  }
                  setMode(value as EditorMode)
                }}
                options={[
                  { value: 'form', label: t('formMode'), icon: <LayoutTemplate size={14} /> },
                  { value: 'json', label: t('jsonMode'), icon: <Code size={14} /> },
                ]}
              />
            </div>

            <div className="editor-reference-card-body">
              <div className="editor-reference-stack">
                {mode === 'form' ? (
                  draft.transportType === 'stdio' ? (
                    <>
                      <label className="editor-reference-field">
                        <span>{t('command')}</span>
                        <input
                          placeholder="e.g., npx, node, python"
                          value={draft.program}
                          onChange={(event) => setDraft((current) => ({ ...current, program: event.target.value }))}
                        />
                      </label>

                      <div>
                        <div className="editor-reference-list-header">
                          <label className="editor-reference-label">{t('args')}</label>
                          <button type="button" className="editor-reference-mini-action" onClick={addArg}>
                            <Plus size={12} />
                            {t('addArg')}
                          </button>
                        </div>
                        <div className="editor-reference-list">
                          {draft.args.length === 0 ? (
                            <div className="editor-reference-empty-row">{t('noArguments')}</div>
                          ) : (
                            draft.args.map((value, index) => (
                              <div key={`${index}-${value}`} className="editor-reference-row">
                                <input value={value} onChange={(event) => updateArg(index, event.target.value)} />
                                <button type="button" className="editor-reference-danger-button" onClick={() => removeArg(index)} aria-label={t('delete')}>
                                  <Trash2 size={16} />
                                </button>
                              </div>
                            ))
                          )}
                        </div>
                      </div>

                      <div>
                        <div className="editor-reference-list-header">
                          <label className="editor-reference-label">{t('environmentVariables')}</label>
                          <button type="button" className="editor-reference-mini-action" onClick={addEnv}>
                            <Plus size={12} />
                            {t('addEnv')}
                          </button>
                        </div>
                        <div className="editor-reference-list">
                          {draft.envEntries.length === 0 ? (
                            <div className="editor-reference-empty-row">{t('noEnvironmentVariables')}</div>
                          ) : (
                            draft.envEntries.map((entry, index) => (
                              <div key={`${entry.key}-${index}`} className="editor-reference-row editor-reference-row-split">
                                <input
                                  placeholder={t('envKey')}
                                  value={entry.key}
                                  onChange={(event) => updateEnv(index, { key: event.target.value })}
                                />
                                <input
                                  placeholder={t('envValue')}
                                  value={entry.value}
                                  onChange={(event) => updateEnv(index, { value: event.target.value })}
                                />
                                <button type="button" className="editor-reference-danger-button" onClick={() => removeEnv(index)} aria-label={t('delete')}>
                                  <Trash2 size={16} />
                                </button>
                              </div>
                            ))
                          )}
                        </div>
                      </div>
                    </>
                  ) : (
                    <label className="editor-reference-field">
                      <span>{t('url')}</span>
                      <input value={draft.url} onChange={(event) => setDraft((current) => ({ ...current, url: event.target.value }))} />
                    </label>
                  )
                ) : (
                  <>
                    <div className="editor-reference-info">{t('jsonPasteHint')}</div>
                    <label className="editor-reference-field">
                      <span>{t('jsonMode')}</span>
                      <textarea
                        rows={16}
                        value={jsonText}
                        placeholder={jsonPlaceholder}
                        onChange={(event) => setJsonText(event.target.value)}
                        spellCheck={false}
                      />
                    </label>
                    <div className="editor-reference-footer">
                      <button type="button" className="ghost-button" onClick={handleParseJson}>
                        {t('parse')}
                      </button>
                    </div>
                  </>
                )}

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
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  )
}

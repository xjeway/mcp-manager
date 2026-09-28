import { useEffect, useMemo, useState } from 'react'
import {
  AlertTriangle,
  ArrowLeft,
  CheckCircle2,
  ExternalLink,
  Globe,
  LoaderCircle,
  Plus,
  RefreshCw,
  Search,
  SearchX,
  Star,
  X,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Tooltip } from './Tooltip'
import type { MCPServer } from '../types/config'
import type {
  InstallOption,
  MarketplaceEntry,
  MarketplaceError,
  MarketplaceSource,
  SearchPage,
} from '../types/marketplace'
import {
  listMarketplaceSources,
  openMarketplaceUrl,
  searchMarketplace,
} from '../services/marketplaceService'
import {
  buildServerFromOption,
  commandPreview,
  defaultOptionIndex,
  editableInputs,
  isEntryAdded,
  headerPreview,
  inputIsSecret,
  missingRequiredInputs,
  type InputValues,
} from '../view-models/marketplace'

interface MarketplacePageProps {
  servers: MCPServer[]
  onBack: () => void
  onInstall: (server: MCPServer) => void
}

type LoadState =
  | { kind: 'loading' }
  | { kind: 'ready'; page: SearchPage }
  | { kind: 'error'; error: MarketplaceError }

const SEARCH_DEBOUNCE_MS = 300

function optionLabel(option: InstallOption, t: (key: string) => string): string {
  switch (option.kind) {
    case 'stdio':
      return option.packageType === 'oci'
        ? 'Docker'
        : option.packageType === 'other'
          ? option.program
          : option.packageType
    case 'http':
      return t('marketplaceOptionRemote')
    case 'unsupported':
      return option.packageType
  }
}

function formatStars(stars: number): string {
  return stars >= 1000 ? `${(stars / 1000).toFixed(stars >= 10_000 ? 0 : 1)}k` : String(stars)
}

function Monogram({ title }: { title: string }) {
  const letter = (title.match(/[\p{L}\p{N}]/u)?.[0] ?? '?').toUpperCase()
  return (
    <span className="marketplace-monogram" aria-hidden="true">
      {letter}
    </span>
  )
}

export function MarketplacePage({ servers, onBack, onInstall }: MarketplacePageProps) {
  const { t, i18n } = useTranslation()
  const [sources, setSources] = useState<MarketplaceSource[]>([])
  const [sourceId, setSourceId] = useState('github')
  const [query, setQuery] = useState('')
  const [debouncedQuery, setDebouncedQuery] = useState('')
  const [state, setState] = useState<LoadState>({ kind: 'loading' })
  const [loadingMore, setLoadingMore] = useState(false)
  const [reloadKey, setReloadKey] = useState(0)
  const [selectedId, setSelectedId] = useState<string | null>(null)

  useEffect(() => {
    listMarketplaceSources()
      .then(setSources)
      .catch((error: MarketplaceError) => setState({ kind: 'error', error }))
  }, [])

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedQuery(query), SEARCH_DEBOUNCE_MS)
    return () => window.clearTimeout(timer)
  }, [query])

  useEffect(() => {
    if (sources.length === 0) {
      return
    }
    let alive = true
    setState({ kind: 'loading' })
    searchMarketplace(sourceId, debouncedQuery)
      .then((page) => alive && setState({ kind: 'ready', page }))
      .catch((error: MarketplaceError) => alive && setState({ kind: 'error', error }))
    return () => {
      alive = false
    }
  }, [sources, sourceId, debouncedQuery, reloadKey])

  const source = sources.find((item) => item.id === sourceId)
  const entries = state.kind === 'ready' ? state.page.entries : []
  const selected = entries.find((entry) => entry.id === selectedId) ?? null

  const loadMore = async () => {
    if (state.kind !== 'ready' || !state.page.nextCursor) {
      return
    }
    setLoadingMore(true)
    try {
      const next = await searchMarketplace(sourceId, debouncedQuery, state.page.nextCursor)
      setState({
        kind: 'ready',
        page: {
          ...next,
          entries: [
            ...state.page.entries,
            ...next.entries.filter((entry) => !state.page.entries.some((existing) => existing.id === entry.id)),
          ],
        },
      })
    } catch (error) {
      setState({ kind: 'error', error: error as MarketplaceError })
    } finally {
      setLoadingMore(false)
    }
  }

  const statusText = (() => {
    if (state.kind !== 'ready') {
      return null
    }
    const fetchedAt = new Date(state.page.fetchedAt * 1000).toLocaleString(i18n.language)
    return state.page.stale ? t('marketplaceStale', { time: fetchedAt }) : t('marketplaceCount', { count: entries.length })
  })()

  return (
    <div className="app-shell shell-flat marketplace-shell">
      <div className="mac-window-drag-region" data-tauri-drag-region aria-hidden="true" />
      <div className="page-header">
        <div className="brand-block">
          <Tooltip content={t('back')}>
            <button type="button" className="icon-button toolbar-icon" onClick={onBack} aria-label={t('back')}>
              <ArrowLeft size={16} />
            </button>
          </Tooltip>
          <div>
            <p className="eyebrow">{t('marketplace')}</p>
            <h1 className="shell-title">{t('marketplaceTitle')}</h1>
          </div>
        </div>

        {sources.length > 0 ? (
          <div className="header-controls" data-tauri-no-drag>
            <div className="segment-control marketplace-sources" role="tablist" aria-label={t('marketplaceSource')}>
              {sources.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  role="tab"
                  aria-selected={item.id === sourceId}
                  className={item.id === sourceId ? 'segment-button active' : 'segment-button'}
                  onClick={() => {
                    setSourceId(item.id)
                    setSelectedId(null)
                  }}
                >
                  {item.label}
                </button>
              ))}
            </div>
          </div>
        ) : null}
      </div>

      <div className="marketplace-toolbar" data-tauri-no-drag>
        <div className="server-search marketplace-search">
          <Search size={13} />
          <input
            autoFocus
            type="search"
            value={query}
            placeholder={t('marketplaceSearchPlaceholder')}
            aria-label={t('marketplaceSearchPlaceholder')}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Escape') {
                setQuery('')
              }
            }}
          />
        </div>
        {source ? (
          <span className={`marketplace-trust trust-${source.trust}`}>{t(`marketplaceTrust.${source.trust}`)}</span>
        ) : null}
        {statusText ? (
          <span className={state.kind === 'ready' && state.page.stale ? 'marketplace-status is-stale' : 'marketplace-status'}>
            {state.kind === 'ready' && state.page.stale ? <AlertTriangle size={12} /> : null}
            {statusText}
          </span>
        ) : null}
      </div>

      <div className={selected ? 'marketplace-body has-selection' : 'marketplace-body'}>
        <section className="list-panel marketplace-list" aria-busy={state.kind === 'loading'}>
          {state.kind === 'loading' ? (
            <div className="empty-state empty-state-compact">
              <LoaderCircle size={22} className="spin" />
              <h3>{t('marketplaceLoading', { source: source?.label ?? '' })}</h3>
              {source && !source.serverSearch ? <p>{t('marketplaceLoadingFullList')}</p> : null}
            </div>
          ) : state.kind === 'error' ? (
            <div className="empty-state empty-state-compact">
              <AlertTriangle size={22} />
              <h3>{t(`marketplaceError.${state.error.code}`)}</h3>
              {state.error.message ? <p className="marketplace-error-detail">{state.error.message}</p> : null}
              {state.error.code === 'network' || state.error.code === 'parse' ? (
                <button type="button" className="ghost-button" onClick={() => setReloadKey((key) => key + 1)}>
                  <RefreshCw size={14} />
                  {t('marketplaceRetry')}
                </button>
              ) : null}
            </div>
          ) : entries.length === 0 ? (
            <div className="empty-state empty-state-compact">
              <SearchX size={22} />
              <h3>{t('marketplaceEmpty')}</h3>
              {query ? (
                <button type="button" className="ghost-button" onClick={() => setQuery('')}>
                  {t('marketplaceClearSearch')}
                </button>
              ) : null}
            </div>
          ) : (
            <div className="server-list-scroll" role="listbox" aria-label={t('marketplace')}>
              {entries.map((entry) => {
                const added = isEntryAdded(entry, servers)
                return (
                  <button
                    key={entry.id}
                    type="button"
                    role="option"
                    aria-selected={entry.id === selectedId}
                    className={entry.id === selectedId ? 'marketplace-row is-selected' : 'marketplace-row'}
                    onClick={() => setSelectedId(entry.id)}
                  >
                    <Monogram title={entry.title} />
                    <span className="marketplace-row-main">
                      <span className="marketplace-row-title">
                        <strong>{entry.title}</strong>
                        {added ? (
                          <span className="marketplace-added">
                            <CheckCircle2 size={11} />
                            {t('marketplaceAdded')}
                          </span>
                        ) : null}
                      </span>
                      <span className="marketplace-row-description">{entry.description || entry.id}</span>
                    </span>
                    {entry.stars !== null ? (
                      <span className="marketplace-stars" aria-label={t('marketplaceStars', { count: entry.stars })}>
                        <Star size={11} />
                        {formatStars(entry.stars)}
                      </span>
                    ) : null}
                  </button>
                )
              })}
              {state.kind === 'ready' && state.page.nextCursor ? (
                <div className="marketplace-more">
                  <button type="button" className="ghost-button" onClick={() => void loadMore()} disabled={loadingMore}>
                    {loadingMore ? <LoaderCircle size={14} className="spin" /> : null}
                    {t('marketplaceLoadMore')}
                  </button>
                </div>
              ) : null}
            </div>
          )}
        </section>

        {selected ? (
          <EntryDetail
            key={`${selected.sourceId}:${selected.id}`}
            entry={selected}
            servers={servers}
            onClose={() => setSelectedId(null)}
            onInstall={onInstall}
          />
        ) : (
          <aside className="marketplace-detail marketplace-detail-empty">
            <p>{t('marketplaceSelectHint')}</p>
          </aside>
        )}
      </div>
    </div>
  )
}

function EntryDetail({
  entry,
  servers,
  onClose,
  onInstall,
}: {
  entry: MarketplaceEntry
  servers: MCPServer[]
  onClose: () => void
  onInstall: (server: MCPServer) => void
}) {
  const { t } = useTranslation()
  const [optionIndex, setOptionIndex] = useState(() => defaultOptionIndex(entry.installOptions))
  const [values, setValues] = useState<InputValues>({})
  const [showMissing, setShowMissing] = useState(false)
  const option = entry.installOptions[optionIndex] as InstallOption | undefined
  const inputs = useMemo(() => (option ? editableInputs(option) : []), [option])
  const missing = option ? missingRequiredInputs(option, values) : []
  const headers = option ? headerPreview(option, values) : []
  const added = isEntryAdded(entry, servers)

  const install = () => {
    if (!option || option.kind === 'unsupported') {
      return
    }
    if (missing.length > 0) {
      setShowMissing(true)
      return
    }
    onInstall(
      buildServerFromOption(
        entry,
        option,
        values,
        servers.map((server) => server.id),
      ),
    )
  }

  const openLink = (url: string) => void openMarketplaceUrl(url)

  return (
    <aside className="marketplace-detail" aria-label={entry.title}>
      <div className="marketplace-detail-scroll">
        <header className="marketplace-detail-header">
          <Monogram title={entry.title} />
          <div className="marketplace-detail-heading">
            <h2>{entry.title}</h2>
            <code className="marketplace-detail-id">
              {entry.id}
              {entry.version ? ` · ${entry.version}` : ''}
            </code>
          </div>
          <Tooltip content={t('close')}>
            <button type="button" className="icon-button compact-icon marketplace-detail-close" onClick={onClose} aria-label={t('close')}>
              <X size={14} />
            </button>
          </Tooltip>
        </header>

        <div className="marketplace-links">
          {entry.repositoryUrl ? (
            <button type="button" className="ghost-button compact" onClick={() => openLink(entry.repositoryUrl!)}>
              <ExternalLink size={13} />
              {t('marketplaceRepository')}
            </button>
          ) : null}
          {entry.websiteUrl && entry.websiteUrl !== entry.repositoryUrl ? (
            <button type="button" className="ghost-button compact" onClick={() => openLink(entry.websiteUrl!)}>
              <Globe size={13} />
              {t('marketplaceWebsite')}
            </button>
          ) : null}
          {added ? (
            <span className="marketplace-added">
              <CheckCircle2 size={11} />
              {t('marketplaceAdded')}
            </span>
          ) : null}
        </div>

        {entry.description ? <p className="marketplace-description">{entry.description}</p> : null}
        {entry.readmeExcerpt ? <p className="marketplace-readme">{entry.readmeExcerpt}</p> : null}

        <section className="marketplace-install">
          <p className="settings-label">{t('marketplaceInstallMethod')}</p>
          {entry.installOptions.length === 0 ? (
            <p className="marketplace-note">{t('marketplaceNoInstallOptions')}</p>
          ) : (
            <div className="marketplace-options" role="radiogroup" aria-label={t('marketplaceInstallMethod')}>
              {entry.installOptions.map((item, index) => (
                <button
                  key={index}
                  type="button"
                  role="radio"
                  aria-checked={index === optionIndex}
                  className={index === optionIndex ? 'marketplace-option is-active' : 'marketplace-option'}
                  onClick={() => {
                    setOptionIndex(index)
                    setShowMissing(false)
                  }}
                >
                  <span className="marketplace-option-kind">{optionLabel(item, t)}</span>
                  {item.kind === 'unsupported' ? (
                    <span className="marketplace-option-note">{t('marketplaceManualOnly')}</span>
                  ) : null}
                </button>
              ))}
            </div>
          )}

          {option?.kind === 'stdio' && option.inferred ? (
            <p className="marketplace-note is-warning">
              <AlertTriangle size={12} />
              {t('marketplaceInferred')}
            </p>
          ) : null}

          {option?.kind === 'unsupported' ? (
            <p className="marketplace-note">{t('marketplaceUnsupported', { type: option.packageType })}</p>
          ) : null}

          {inputs.length > 0 ? (
            <div className="marketplace-inputs">
              {inputs.map((item) => {
                const isMissing = showMissing && missing.some((m) => m.key === item.key)
                return (
                  <label key={item.key} className={isMissing ? 'marketplace-input is-missing' : 'marketplace-input'}>
                    <span className="marketplace-input-label">
                      <code>{item.key}</code>
                      {item.required ? <span className="marketplace-required">{t('marketplaceRequired')}</span> : null}
                    </span>
                    {item.description ? <span className="marketplace-input-help">{item.description}</span> : null}
                    <input
                      type={option && inputIsSecret(option, item) ? 'password' : 'text'}
                      autoComplete="off"
                      spellCheck={false}
                      value={values[item.key] ?? ''}
                      placeholder={item.defaultValue ?? ''}
                      onChange={(event) => setValues((current) => ({ ...current, [item.key]: event.target.value }))}
                    />
                  </label>
                )
              })}
            </div>
          ) : null}

          {headers.length > 0 ? (
            <div className="marketplace-note">
              <p>{t('requestHeaders')}</p>
              <ul className="marketplace-headers">
                {headers.map((header) => (
                  <li key={header.name}>
                    <code>
                      {header.name}: {header.value}
                    </code>
                  </li>
                ))}
              </ul>
            </div>
          ) : null}

          {option && option.kind !== 'unsupported' ? (
            <div className="marketplace-run-preview">
              <span className="marketplace-run-label">
                {option.kind === 'http' ? t('marketplaceWillConnect') : t('marketplaceWillRun')}
              </span>
              <code>
                <span className="marketplace-run-prompt" aria-hidden="true">
                  {option.kind === 'http' ? '→' : '$'}
                </span>
                {commandPreview(option, values)}
              </code>
            </div>
          ) : null}
        </section>
      </div>

      <footer className="marketplace-detail-footer">
        {showMissing && missing.length > 0 ? (
          <p className="marketplace-missing" role="alert">
            {t('marketplaceMissingInputs', { keys: missing.map((item) => item.key).join(', ') })}
          </p>
        ) : (
          <p className="marketplace-footer-hint">{t('marketplaceInstallHint')}</p>
        )}
        <button
          type="button"
          className="primary-button"
          onClick={install}
          disabled={!option || option.kind === 'unsupported'}
        >
          <Plus size={14} />
          {t('marketplaceAddToWorkspace')}
        </button>
      </footer>
    </aside>
  )
}

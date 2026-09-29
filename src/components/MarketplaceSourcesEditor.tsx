import { type FormEvent, useEffect, useState } from 'react'
import { LoaderCircle, Plus, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Tooltip } from './Tooltip'
import type { MarketplaceError, MarketplaceSource } from '../types/marketplace'
import {
  addMarketplaceSource,
  listMarketplaceSources,
  removeMarketplaceSource,
} from '../services/marketplaceService'

/** Lists marketplace sources and lets the user add or remove their own registries. */
export function MarketplaceSourcesEditor() {
  const { t } = useTranslation()
  const [sources, setSources] = useState<MarketplaceSource[]>([])
  const [label, setLabel] = useState('')
  const [baseUrl, setBaseUrl] = useState('')
  const [adding, setAdding] = useState(false)
  const [removingId, setRemovingId] = useState<string | null>(null)
  const [error, setError] = useState<MarketplaceError | null>(null)

  useEffect(() => {
    let alive = true
    listMarketplaceSources()
      .then((list) => alive && setSources(list))
      .catch((failure: MarketplaceError) => alive && setError(failure))
    return () => {
      alive = false
    }
  }, [])

  const add = async (event: FormEvent) => {
    event.preventDefault()
    if (!baseUrl.trim() || adding) {
      return
    }
    setAdding(true)
    setError(null)
    try {
      const source = await addMarketplaceSource(label, baseUrl)
      setSources((current) => [...current, source])
      setLabel('')
      setBaseUrl('')
    } catch (failure) {
      setError(failure as MarketplaceError)
    } finally {
      setAdding(false)
    }
  }

  const remove = async (source: MarketplaceSource) => {
    setRemovingId(source.id)
    setError(null)
    try {
      await removeMarketplaceSource(source.id)
      setSources((current) => current.filter((item) => item.id !== source.id))
    } catch (failure) {
      setError(failure as MarketplaceError)
    } finally {
      setRemovingId(null)
    }
  }

  return (
    <div className="marketplace-sources-editor">
      <ul className="marketplace-source-list">
        {sources.map((source) => (
          <li key={source.id} className="marketplace-source-item">
            <span className="marketplace-source-copy">
              <strong>{source.label}</strong>
              <span className="marketplace-source-url">{source.baseUrl}</span>
            </span>
            {source.builtin ? (
              <span className="marketplace-source-badge">{t('settingsMarketplaceSourceBuiltin')}</span>
            ) : (
              <Tooltip content={t('settingsMarketplaceSourceRemove')}>
                <button
                  type="button"
                  className="icon-button"
                  onClick={() => void remove(source)}
                  disabled={removingId === source.id}
                  aria-label={`${t('settingsMarketplaceSourceRemove')} ${source.label}`}
                >
                  <Trash2 size={14} />
                </button>
              </Tooltip>
            )}
          </li>
        ))}
      </ul>

      <form className="marketplace-source-form" onSubmit={(event) => void add(event)}>
        <input
          className="editor-input"
          value={label}
          maxLength={40}
          placeholder={t('settingsMarketplaceSourceName')}
          aria-label={t('settingsMarketplaceSourceName')}
          onChange={(event) => setLabel(event.target.value)}
        />
        <input
          className="editor-input editor-input-mono"
          type="url"
          value={baseUrl}
          placeholder="https://registry.example.com"
          aria-label={t('settingsMarketplaceSourceUrl')}
          onChange={(event) => setBaseUrl(event.target.value)}
        />
        <button type="submit" className="ghost-button" disabled={adding || !baseUrl.trim()}>
          {adding ? <LoaderCircle size={14} className="spin" /> : <Plus size={14} />}
          {adding ? t('settingsMarketplaceSourceChecking') : t('settingsMarketplaceSourceAdd')}
        </button>
      </form>

      {error ? (
        <p className="marketplace-source-error" role="alert">
          {t(`marketplaceError.${error.code}`)}
          {error.message ? <span className="marketplace-error-detail"> {error.message}</span> : null}
        </p>
      ) : null}
    </div>
  )
}

import { invoke } from '@tauri-apps/api/core'
import type { MarketplaceEntry, MarketplaceError, MarketplaceSource, SearchPage } from '../types/marketplace'
import { isDesktopRuntime } from './runtime'

const ERROR_CODES = new Set(['disabled', 'unknown-source', 'network', 'parse', 'unavailable'])

export function toMarketplaceError(error: unknown): MarketplaceError {
  if (typeof error === 'object' && error !== null && 'code' in error && ERROR_CODES.has(String(error.code))) {
    return error as MarketplaceError
  }
  return { code: 'network', message: String(error) }
}

const UNAVAILABLE: MarketplaceError = { code: 'unavailable' }

/**
 * Plain browser runs (`npm run dev`) cannot reach the registries (no CORS), so the dev
 * build serves normalized sample data captured from them; production browser builds
 * report the marketplace as unavailable.
 */
async function loadSample(): Promise<Record<string, MarketplaceEntry[]>> {
  if (!import.meta.env.DEV) {
    throw UNAVAILABLE
  }
  return (await import('../dev/marketplaceSample.json')).default as Record<string, MarketplaceEntry[]>
}

const SAMPLE_SOURCES: MarketplaceSource[] = [
  {
    id: 'github',
    kind: 'mcp-registry',
    label: 'GitHub MCP Registry',
    baseUrl: 'https://api.mcp.github.com/2025-09-15',
    trust: 'curated',
    serverSearch: false,
  },
  {
    id: 'official',
    kind: 'mcp-registry',
    label: 'MCP Registry',
    baseUrl: 'https://registry.modelcontextprotocol.io',
    trust: 'community',
    serverSearch: true,
  },
]

function matchesQuery(entry: MarketplaceEntry, query: string): boolean {
  const haystack = `${entry.id} ${entry.title} ${entry.description}`.toLowerCase()
  return query
    .toLowerCase()
    .split(/\s+/)
    .filter(Boolean)
    .every((term) => haystack.includes(term))
}

export async function setMarketplaceEnabled(enabled: boolean): Promise<void> {
  if (isDesktopRuntime()) {
    await invoke('marketplace_set_enabled', { enabled })
  }
}

export async function listMarketplaceSources(): Promise<MarketplaceSource[]> {
  if (!isDesktopRuntime()) {
    await loadSample()
    return SAMPLE_SOURCES
  }
  try {
    return await invoke<MarketplaceSource[]>('marketplace_sources')
  } catch (error) {
    throw toMarketplaceError(error)
  }
}

export async function searchMarketplace(sourceId: string, query: string, cursor?: string | null): Promise<SearchPage> {
  if (!isDesktopRuntime()) {
    const sample = await loadSample()
    return {
      entries: (sample[sourceId] ?? []).filter((entry) => matchesQuery(entry, query)),
      nextCursor: null,
      skipped: 0,
      stale: false,
      fetchedAt: Math.floor(Date.now() / 1000),
    }
  }
  try {
    return await invoke<SearchPage>('marketplace_search', { sourceId, query, cursor: cursor ?? null })
  } catch (error) {
    throw toMarketplaceError(error)
  }
}

export async function openMarketplaceUrl(url: string): Promise<void> {
  if (!isDesktopRuntime()) {
    window.open(url, '_blank', 'noopener,noreferrer')
    return
  }
  await invoke('marketplace_open_url', { url })
}

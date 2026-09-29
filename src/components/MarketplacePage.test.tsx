// @vitest-environment jsdom
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { MarketplaceEntry, SearchPage } from '../types/marketplace'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key, i18n: { language: 'en-US' } }),
}))

const entry = (id: string): MarketplaceEntry => ({
  sourceId: id.split(':')[0],
  id,
  title: id,
  description: '',
  version: null,
  repositoryUrl: null,
  websiteUrl: null,
  stars: null,
  readmeExcerpt: null,
  installOptions: [],
})
const page = (ids: string[], nextCursor: string | null = null): SearchPage => ({
  entries: ids.map(entry),
  nextCursor,
  skipped: 0,
  stale: false,
  fetchedAt: 0,
})

let resolveLoadMore: (page: SearchPage) => void = () => {}
let resolveRefresh: (page: SearchPage) => void = () => {}
let rejectRefresh: () => void = () => {}
let officialIsStale = false

vi.mock('../services/marketplaceService', () => ({
  listMarketplaceSources: () =>
    Promise.resolve([
      { id: 'github', kind: 'mcp-registry', label: 'GitHub', baseUrl: '', trust: 'curated', serverSearch: false, builtin: true },
      { id: 'official', kind: 'mcp-registry', label: 'Official', baseUrl: '', trust: 'community', serverSearch: true, builtin: true },
    ]),
  searchMarketplace: (sourceId: string, _query: string, cursor?: string | null) => {
    if (cursor) {
      return new Promise<SearchPage>((resolve) => {
        resolveLoadMore = resolve
      })
    }
    if (sourceId === 'github') {
      return Promise.resolve(page(['github:a'], 'next'))
    }
    return Promise.resolve({ ...page(['official:x']), stale: officialIsStale })
  },
  refreshMarketplace: () =>
    new Promise<SearchPage>((resolve, reject) => {
      resolveRefresh = resolve
      rejectRefresh = () => reject({ code: 'network' })
    }),
  openMarketplaceUrl: () => Promise.resolve(),
}))

import { MarketplacePage } from './MarketplacePage'

;(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true

let container: HTMLDivElement
let root: Root

beforeEach(async () => {
  vi.useFakeTimers()
  container = document.createElement('div')
  document.body.appendChild(container)
  root = createRoot(container)
  await act(async () => {
    root.render(<MarketplacePage servers={[]} onBack={() => {}} onInstall={() => {}} />)
  })
  // Let the search debounce fire and the first page resolve.
  await act(async () => {
    vi.advanceTimersByTime(400)
  })
})

afterEach(() => {
  officialIsStale = false
  act(() => root.unmount())
  container.remove()
  vi.useRealTimers()
})

const titles = () => [...container.querySelectorAll('.marketplace-row strong')].map((node) => node.textContent)
const button = (label: string) =>
  [...container.querySelectorAll('button')].find((node) => node.textContent?.includes(label))!

describe('MarketplacePage', () => {
  it('drops a load-more result that arrives after switching source', async () => {
    expect(titles()).toEqual(['github:a'])

    await act(async () => button('marketplaceLoadMore').click())
    await act(async () => button('Official').click())
    expect(titles()).toEqual(['official:x'])

    await act(async () => resolveLoadMore(page(['github:b'])))
    expect(titles()).toEqual(['official:x'])
  })

  it('shows a stale page at once and swaps in the refreshed one', async () => {
    officialIsStale = true
    await act(async () => button('Official').click())
    expect(titles()).toEqual(['official:x'])
    expect(container.querySelector('.marketplace-status')?.textContent).toBe('marketplaceRefreshing')

    await act(async () => resolveRefresh(page(['official:y'])))
    expect(titles()).toEqual(['official:y'])
    expect(container.querySelector('.marketplace-status')?.textContent).toBe('marketplaceCount')
  })

  it('keeps the stale page with a warning when the refresh fails', async () => {
    officialIsStale = true
    await act(async () => button('Official').click())
    await act(async () => rejectRefresh())
    expect(titles()).toEqual(['official:x'])
    expect(container.querySelector('.marketplace-status.is-stale')?.textContent).toBe('marketplaceStale')
  })
})

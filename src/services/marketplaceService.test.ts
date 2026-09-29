import { describe, expect, it, vi } from 'vitest'

vi.mock('./runtime', () => ({ isDesktopRuntime: () => false }))

import { searchMarketplace, listMarketplaceSources, toMarketplaceError } from './marketplaceService'

describe('toMarketplaceError', () => {
  it('passes through errors returned by the backend', () => {
    expect(toMarketplaceError({ code: 'network', message: 'timed out' })).toEqual({
      code: 'network',
      message: 'timed out',
    })
    expect(toMarketplaceError({ code: 'disabled' })).toEqual({ code: 'disabled' })
  })

  it('wraps anything else as a network error', () => {
    expect(toMarketplaceError('boom')).toEqual({ code: 'network', message: 'boom' })
    expect(toMarketplaceError(new Error('bad'))).toEqual({ code: 'network', message: 'Error: bad' })
  })
})

describe('browser preview fallback', () => {
  it('serves sample sources and filters sample entries by every term', async () => {
    const sources = await listMarketplaceSources()
    expect(sources.map((source) => source.id)).toEqual(['github', 'official'])

    const all = await searchMarketplace('github', '')
    expect(all.entries.length).toBeGreaterThan(3)
    expect(all.stale).toBe(false)

    const filtered = await searchMarketplace('github', 'mongodb server')
    expect(filtered.entries.map((entry) => entry.id)).toEqual(['io.github.mongodb-js/mongodb-mcp-server'])
  })
})

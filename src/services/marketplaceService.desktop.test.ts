import { describe, expect, it, vi } from 'vitest'

const calls: string[] = []
let releaseEnable: () => void = () => {}

vi.mock('./runtime', () => ({ isDesktopRuntime: () => true }))
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((command: string) => {
    calls.push(`start ${command}`)
    if (command === 'marketplace_set_enabled') {
      // The backend acknowledges the flag only when the test says so.
      return new Promise<void>((resolve) => {
        releaseEnable = () => {
          calls.push(`done ${command}`)
          resolve()
        }
      })
    }
    return Promise.resolve([])
  }),
}))

import { listMarketplaceSources, searchMarketplace, setMarketplaceEnabled } from './marketplaceService'

describe('marketplace requests on the desktop', () => {
  it('wait for the enable flag to reach the backend before querying it', async () => {
    const enabling = setMarketplaceEnabled(true)
    const sources = listMarketplaceSources()
    const search = searchMarketplace('github', '')

    await Promise.resolve()
    expect(calls).toEqual(['start marketplace_set_enabled'])

    releaseEnable()
    await Promise.all([enabling, sources, search])
    expect(calls).toEqual([
      'start marketplace_set_enabled',
      'done marketplace_set_enabled',
      'start marketplace_sources',
      'start marketplace_search',
    ])
  })
})

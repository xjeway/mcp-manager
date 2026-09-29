// @vitest-environment jsdom
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { MarketplaceSource } from '../types/marketplace'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))

const source = (id: string, builtin: boolean): MarketplaceSource => ({
  id,
  kind: 'mcp-registry',
  label: id,
  baseUrl: `https://${id}.dev`,
  trust: builtin ? 'curated' : 'community',
  serverSearch: true,
  builtin,
})

const addMarketplaceSource = vi.fn()
const removeMarketplaceSource = vi.fn()

vi.mock('../services/marketplaceService', () => ({
  listMarketplaceSources: () => Promise.resolve([source('github', true), source('acme', false)]),
  addMarketplaceSource: (...args: unknown[]) => addMarketplaceSource(...args),
  removeMarketplaceSource: (...args: unknown[]) => removeMarketplaceSource(...args),
}))

import { MarketplaceSourcesEditor } from './MarketplaceSourcesEditor'

;(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true

let container: HTMLDivElement
let root: Root

beforeEach(async () => {
  addMarketplaceSource.mockReset()
  removeMarketplaceSource.mockReset()
  container = document.createElement('div')
  document.body.appendChild(container)
  root = createRoot(container)
  await act(async () => root.render(<MarketplaceSourcesEditor />))
})

afterEach(() => {
  act(() => root.unmount())
  container.remove()
})

const labels = () => [...container.querySelectorAll('.marketplace-source-item strong')].map((node) => node.textContent)

function type(input: HTMLInputElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!
  setter.call(input, value)
  input.dispatchEvent(new Event('input', { bubbles: true }))
}

async function submit(url: string) {
  const [, urlInput] = container.querySelectorAll('input')
  await act(async () => type(urlInput, url))
  await act(async () => {
    container.querySelector('form')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }))
  })
}

describe('MarketplaceSourcesEditor', () => {
  it('only offers removal for user sources', () => {
    expect(labels()).toEqual(['github', 'acme'])
    const items = container.querySelectorAll('.marketplace-source-item')
    expect(items[0].querySelector('button')).toBeNull()
    expect(items[0].textContent).toContain('settingsMarketplaceSourceBuiltin')
    expect(items[1].querySelector('button')).not.toBeNull()
  })

  it('adds a source and clears the form', async () => {
    addMarketplaceSource.mockResolvedValue(source('new', false))
    await submit('https://new.dev')
    expect(addMarketplaceSource).toHaveBeenCalledWith('', 'https://new.dev')
    expect(labels()).toEqual(['github', 'acme', 'new'])
    expect(container.querySelectorAll('input')[1].value).toBe('')
  })

  it('shows why a source was rejected and keeps the input', async () => {
    addMarketplaceSource.mockRejectedValue({ code: 'duplicate-source', message: 'https://acme.dev' })
    await submit('https://acme.dev')
    expect(container.querySelector('[role="alert"]')?.textContent).toContain('marketplaceError.duplicate-source')
    expect(labels()).toEqual(['github', 'acme'])
    expect(container.querySelectorAll('input')[1].value).toBe('https://acme.dev')
  })

  it('removes a user source', async () => {
    removeMarketplaceSource.mockResolvedValue(undefined)
    await act(async () => container.querySelectorAll<HTMLButtonElement>('.marketplace-source-item button')[0].click())
    expect(removeMarketplaceSource).toHaveBeenCalledWith('acme')
    expect(labels()).toEqual(['github'])
  })
})

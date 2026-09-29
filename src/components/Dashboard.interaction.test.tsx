// @vitest-environment jsdom
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { Dashboard } from './Dashboard'
import type { WorkspaceRowViewModel, WorkspaceViewModel } from '../view-models/workspace'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: Record<string, unknown>) =>
      options && 'shown' in options ? `${key}:${options.shown}/${options.total}` : key,
  }),
}))

// React only runs act() warnings-free when told it is in a test environment.
;(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true

const row = (id: string, name: string, copyValue: string, enabledApps: WorkspaceRowViewModel['enabledApps']) => ({
  id,
  name,
  copyValue,
  enabledApps,
  transportLabel: 'STDIO',
  placements: [],
})

const workspace: WorkspaceViewModel = {
  stats: [
    { id: 'cursor', label: 'Cursor', accent: 'client-cursor', icon: null, count: 2 },
    { id: 'vscode', label: 'VS Code', accent: 'client-vscode', icon: null, count: 1 },
  ],
  rows: [
    row('github', 'GitHub', 'uvx mcp-server-github', ['vscode', 'cursor']),
    row('filesystem', 'Filesystem', 'npx @modelcontextprotocol/server-filesystem', ['cursor']),
    row('fetch', 'Fetch', 'uvx mcp-server-fetch', []),
  ],
}

let container: HTMLDivElement
let root: Root

beforeEach(() => {
  container = document.createElement('div')
  document.body.appendChild(container)
  root = createRoot(container)
  act(() => {
    root.render(
      <Dashboard
        busy="idle"
        canRollback={false}
        visibleApps={['cursor', 'vscode']}
        workspace={workspace}
        onAdd={() => {}}
        onOpenRepository={() => {}}
        onSyncLocalConfig={() => {}}
        onOpenSettings={() => {}}
        onDelete={() => {}}
        onEdit={() => {}}
        onRollback={() => {}}
        onToggleApp={() => {}}
        onBatchSetApp={() => {}}
        onCopyCommand={() => {}}
      />,
    )
  })
})

afterEach(() => {
  act(() => root.unmount())
  container.remove()
})

const renderedNames = () => [...container.querySelectorAll('.server-cell-name strong')].map((node) => node.textContent)
const status = () => container.querySelector('[role="status"]')?.textContent

function typeSearch(value: string) {
  const input = container.querySelector<HTMLInputElement>('input[type="search"]')!
  // React tracks the input's value; go through the native setter so onChange fires.
  const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!
  act(() => {
    setValue.call(input, value)
    input.dispatchEvent(new Event('input', { bubbles: true }))
  })
}

function clickClientCard(label: string) {
  const card = [...container.querySelectorAll<HTMLButtonElement>('button.stat-card')].find(
    (button) => button.querySelector('.stat-caption')?.textContent === label,
  )!
  act(() => card.click())
}

describe('Dashboard filtering', () => {
  it('combines the search query with the client filter and announces the result', () => {
    expect(renderedNames()).toEqual(['GitHub', 'Filesystem', 'Fetch'])
    expect(status()).toBe('')

    typeSearch('uvx')
    expect(renderedNames()).toEqual(['GitHub', 'Fetch'])
    expect(status()).toBe('serverSearchStatus:2/3')

    clickClientCard('Cursor')
    expect(renderedNames()).toEqual(['GitHub'])
    expect(status()).toBe('serverSearchStatus:1/3')

    typeSearch('filesystem')
    expect(renderedNames()).toEqual(['Filesystem'])

    clickClientCard('VS Code')
    expect(renderedNames()).toEqual([])
    expect(status()).toBe('serverSearchEmptyTitle')
  })

  it('clears both filters from the empty state', () => {
    typeSearch('nothing-matches')
    expect(renderedNames()).toEqual([])

    const clear = [...container.querySelectorAll('button')].find((button) => button.textContent === 'serverSearchClear')!
    act(() => clear.click())
    expect(renderedNames()).toEqual(['GitHub', 'Filesystem', 'Fetch'])
    expect(status()).toBe('')
  })

  it('does not bring back a selection hidden by the filter', () => {
    const githubCheckbox = container.querySelector<HTMLInputElement>('input[aria-label="batchSelectRow"]')!
    act(() => githubCheckbox.click())
    expect(githubCheckbox.checked).toBe(true)

    typeSearch('filesystem')
    typeSearch('')
    const restored = container.querySelector<HTMLInputElement>('input[aria-label="batchSelectRow"]')!
    expect(restored.checked).toBe(false)
  })
})

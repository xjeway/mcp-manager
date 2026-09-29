// @vitest-environment jsdom
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ConfigProblemDialog } from './ConfigProblemDialog'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

// React only runs act() warnings-free when told it is in a test environment.
;(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true

let container: HTMLDivElement
let root: Root
let trigger: HTMLButtonElement

beforeEach(() => {
  trigger = document.createElement('button')
  document.body.appendChild(trigger)
  trigger.focus()
  container = document.createElement('div')
  document.body.appendChild(container)
  root = createRoot(container)
  act(() => {
    root.render(
      <ConfigProblemDialog
        path="/data/config/servers.yaml"
        detail="bad yaml"
        busy={false}
        restorableCount={2}
        onRestore={() => {}}
        onShowFile={() => {}}
        onReload={() => {}}
        onStartOver={() => {}}
      />,
    )
  })
})

afterEach(() => {
  act(() => root.unmount())
  container.remove()
  trigger.remove()
})

function button(label: string) {
  return Array.from(container.querySelectorAll('button')).find((item) => item.textContent === label)!
}

function tab(shiftKey = false) {
  act(() => {
    document.activeElement?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey, bubbles: true }))
  })
}

describe('ConfigProblemDialog focus', () => {
  it('opens with focus on Reload rather than Start Over', () => {
    expect(document.activeElement).toBe(button('configUnreadableReload'))
  })

  it('wraps Tab and Shift+Tab around its buttons', () => {
    tab()
    expect(document.activeElement).toBe(button('configUnreadableStartOver'))

    tab(true)
    expect(document.activeElement).toBe(button('configUnreadableReload'))
  })

  it('gives focus back to what had it once the dialog closes', () => {
    act(() => root.unmount())
    expect(document.activeElement).toBe(trigger)
    root = createRoot(container)
  })
})

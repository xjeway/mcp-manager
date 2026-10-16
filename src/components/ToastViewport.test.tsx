// @vitest-environment jsdom
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { FeedbackItem } from '../view-models/workspace'
import { ToastViewport, toastDuration } from './ToastViewport'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

;(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true

let container: HTMLDivElement
let root: Root
let onDismiss: ReturnType<typeof vi.fn<(id: string) => void>>

function render(items: FeedbackItem[]) {
  act(() => {
    root.render(<ToastViewport items={items} onDismiss={onDismiss} />)
  })
}

function card(): HTMLElement {
  const element = document.querySelector<HTMLElement>('.toast-card')
  if (!element) {
    throw new Error('toast card not rendered')
  }
  return element
}

function advance(ms: number) {
  act(() => {
    vi.advanceTimersByTime(ms)
  })
}

beforeEach(() => {
  vi.useFakeTimers()
  vi.spyOn(document, 'hasFocus').mockReturnValue(true)
  onDismiss = vi.fn<(id: string) => void>()
  container = document.createElement('div')
  document.body.appendChild(container)
  root = createRoot(container)
})

afterEach(() => {
  act(() => root.unmount())
  container.remove()
  vi.useRealTimers()
  vi.restoreAllMocks()
})

describe('ToastViewport', () => {
  it('auto-dismisses success and info toasts', () => {
    expect(toastDuration('success')).toBe(4000)
    expect(toastDuration('info')).toBe(4000)

    render([{ id: 'a', kind: 'success', message: 'Saved' }])
    advance(3999)
    expect(onDismiss).not.toHaveBeenCalled()
    advance(1)
    expect(onDismiss).toHaveBeenCalledWith('a')
  })

  it('keeps errors and warnings until they are dismissed', () => {
    expect(toastDuration('error')).toBeNull()
    expect(toastDuration('warning')).toBeNull()

    render([
      { id: 'e', kind: 'error', message: 'Save failed' },
      { id: 'w', kind: 'warning', message: 'Headers not written' },
    ])
    advance(60_000)
    expect(onDismiss).not.toHaveBeenCalled()

    act(() => {
      document.querySelector<HTMLButtonElement>('.toast-error .toast-close')?.click()
    })
    expect(onDismiss).toHaveBeenCalledWith('e')
  })

  it('announces errors and warnings as alerts and the rest as status', () => {
    render([
      { id: 'e', kind: 'error', message: 'Save failed' },
      { id: 'w', kind: 'warning', message: 'Careful' },
      { id: 's', kind: 'success', message: 'Saved' },
      { id: 'i', kind: 'info', message: 'FYI' },
    ])

    expect(document.querySelector('.toast-error')?.getAttribute('role')).toBe('alert')
    expect(document.querySelector('.toast-warning')?.getAttribute('role')).toBe('alert')
    expect(document.querySelector('.toast-success')?.getAttribute('role')).toBe('status')
    expect(document.querySelector('.toast-info')?.getAttribute('role')).toBe('status')
  })

  it('localizes the dismiss button label', () => {
    render([{ id: 'a', kind: 'info', message: 'FYI' }])
    expect(document.querySelector('.toast-close')?.getAttribute('aria-label')).toBe('dismissNotification')
  })

  it('pauses the countdown while hovered and resumes with the time left', () => {
    render([{ id: 'a', kind: 'info', message: 'FYI' }])
    advance(1000)

    act(() => {
      card().dispatchEvent(new MouseEvent('mouseover', { bubbles: true, relatedTarget: document.body }))
    })
    advance(10_000)
    expect(onDismiss).not.toHaveBeenCalled()

    act(() => {
      card().dispatchEvent(new MouseEvent('mouseout', { bubbles: true, relatedTarget: document.body }))
    })
    advance(2999)
    expect(onDismiss).not.toHaveBeenCalled()
    advance(1)
    expect(onDismiss).toHaveBeenCalledWith('a')
  })

  it('pauses the countdown while the window is in the background', () => {
    render([{ id: 'a', kind: 'success', message: 'Saved' }])
    advance(1000)

    act(() => {
      window.dispatchEvent(new Event('blur'))
    })
    advance(60_000)
    expect(onDismiss).not.toHaveBeenCalled()

    act(() => {
      window.dispatchEvent(new Event('focus'))
    })
    advance(3000)
    expect(onDismiss).toHaveBeenCalledWith('a')
  })

  it('keeps a persistent toast even when its kind would auto-dismiss', () => {
    render([{ id: 'ready', kind: 'success', message: 'Update ready', persistent: true }])
    advance(60_000)
    expect(onDismiss).not.toHaveBeenCalled()
  })

  it('runs the action and dismisses the toast', () => {
    const onClick = vi.fn()
    render([{ id: 'ready', kind: 'success', message: 'Update ready', action: { label: 'Install now', onClick } }])

    const action = document.querySelector<HTMLButtonElement>('.toast-action')
    expect(action?.textContent).toBe('Install now')
    act(() => action?.click())

    expect(onClick).toHaveBeenCalledTimes(1)
    expect(onDismiss).toHaveBeenCalledWith('ready')
  })

  it('does not restart the countdown when the parent re-renders', () => {
    const items: FeedbackItem[] = [{ id: 'a', kind: 'info', message: 'FYI' }]
    render(items)
    advance(2500)
    // App passes a new onDismiss on every render.
    onDismiss = vi.fn<(id: string) => void>()
    render(items)
    advance(1500)
    expect(onDismiss).toHaveBeenCalledWith('a')
  })
})

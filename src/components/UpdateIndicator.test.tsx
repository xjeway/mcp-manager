// @vitest-environment jsdom
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { UpdatePhase } from '../services/updater'
import { UpdateIndicator } from './UpdateIndicator'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

;(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true

let container: HTMLDivElement
let root: Root
const onInstall = vi.fn()
const onShowProgress = vi.fn()

function render(phase: UpdatePhase, percent: number | null = null) {
  act(() => {
    root.render(
      <UpdateIndicator
        state={{ phase, percent, targetVersion: '0.4.0' }}
        onInstall={onInstall}
        onShowProgress={onShowProgress}
      />,
    )
  })
  return container.querySelector<HTMLButtonElement>('.update-indicator')
}

beforeEach(() => {
  onInstall.mockClear()
  onShowProgress.mockClear()
  container = document.createElement('div')
  document.body.appendChild(container)
  root = createRoot(container)
})

afterEach(() => {
  act(() => root.unmount())
  container.remove()
})

describe('UpdateIndicator', () => {
  it.each<UpdatePhase>(['idle', 'error'])('renders nothing while %s', (phase) => {
    expect(render(phase)).toBeNull()
  })

  it('offers a restart once the update is downloaded', () => {
    const button = render('downloaded')
    expect(button?.textContent).toBe('updateRestartToUpdate')
    expect(button?.classList.contains('update-indicator-ready')).toBe(true)

    act(() => button?.click())
    expect(onInstall).toHaveBeenCalledTimes(1)
    expect(onShowProgress).not.toHaveBeenCalled()
  })

  it('shows download progress and reopens the progress window', () => {
    const button = render('downloading', 45)
    expect(button?.textContent).toBe('updateWindowDownloadingTitle 45%')

    act(() => button?.click())
    expect(onShowProgress).toHaveBeenCalledTimes(1)
    expect(onInstall).not.toHaveBeenCalled()
  })

  it('omits the percentage when the download size is unknown', () => {
    expect(render('downloading', null)?.textContent).toBe('updateWindowDownloadingTitle')
  })

  it('is disabled while installing', () => {
    const button = render('installing')
    expect(button?.disabled).toBe(true)
    expect(button?.textContent).toBe('updateWindowInstallingTitle')
  })
})

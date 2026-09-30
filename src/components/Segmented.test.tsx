// @vitest-environment jsdom
import { act, useState } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { renderToStaticMarkup } from 'react-dom/server'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { Segmented } from './Segmented'
import { Tabs } from './Tabs'
import { Tooltip } from './Tooltip'

;(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true

const options = [
  { label: 'A', value: 'a' },
  { label: 'B', value: 'b' },
  { label: 'C', value: 'c' },
]

describe('Segmented markup', () => {
  it('marks the active option and positions the thumb under it', () => {
    const html = renderToStaticMarkup(<Segmented value="b" options={options} onChange={() => {}} />)

    expect(html).toContain('--segmented-count:3;--segmented-index:1')
    expect(html.match(/aria-checked="true"/g)).toHaveLength(1)
  })

  it('only the active option is in the tab order', () => {
    const html = renderToStaticMarkup(<Segmented value="b" options={options} onChange={() => {}} />)

    expect(html.match(/tabindex="0"/g)).toHaveLength(1)
    expect(html.match(/tabindex="-1"/g)).toHaveLength(2)
  })
})

describe('Tabs markup', () => {
  it('marks the selected tab and wires every tab to the shared panel', () => {
    const html = renderToStaticMarkup(<Tabs idPrefix="t" value="c" options={options} onChange={() => {}} />)

    expect(html.match(/aria-selected="true"/g)).toHaveLength(1)
    expect(html).toContain('id="t-tab-c"')
    expect(html.match(/aria-controls="t-panel"/g)).toHaveLength(3)
  })
})

let container: HTMLDivElement
let root: Root

beforeEach(() => {
  container = document.createElement('div')
  document.body.appendChild(container)
  root = createRoot(container)
})

afterEach(() => {
  act(() => root.unmount())
  container.remove()
})

function press(element: Element, key: string) {
  act(() => {
    element.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }))
  })
}

function Harness({ kind }: { kind: 'segmented' | 'tabs' }) {
  const [value, setValue] = useState('a')
  return kind === 'segmented' ? (
    <Segmented value={value} options={options} onChange={setValue} />
  ) : (
    <Tabs idPrefix="t" value={value} options={options} onChange={setValue} />
  )
}

describe.each([
  ['segmented', '[role="radio"]', 'aria-checked'],
  ['tabs', '[role="tab"]', 'aria-selected'],
] as const)('%s keyboard navigation', (kind, selector, checkedAttr) => {
  const items = () => [...container.querySelectorAll<HTMLButtonElement>(selector)]
  const selected = () => items().findIndex((item) => item.getAttribute(checkedAttr) === 'true')

  it('moves selection and focus with arrow keys, wrapping at both ends', () => {
    act(() => root.render(<Harness kind={kind} />))

    press(items()[0], 'ArrowLeft')
    expect(selected()).toBe(2)
    expect(document.activeElement).toBe(items()[2])

    press(items()[2], 'ArrowRight')
    expect(selected()).toBe(0)

    press(items()[0], 'ArrowRight')
    expect(selected()).toBe(1)
  })

  it('jumps to the first and last item with Home and End', () => {
    act(() => root.render(<Harness kind={kind} />))

    press(items()[0], 'End')
    expect(selected()).toBe(2)
    press(items()[2], 'Home')
    expect(selected()).toBe(0)
  })
})

describe('Tooltip', () => {
  it('closes on a click even when the child stops propagation', () => {
    vi.useFakeTimers()
    try {
      act(() => {
        root.render(
          <Tooltip content="hint">
            <button type="button" onClick={(event) => event.stopPropagation()}>
              x
            </button>
          </Tooltip>,
        )
      })
      const trigger = container.querySelector('.tooltip-trigger') as HTMLElement
      const button = container.querySelector('button') as HTMLButtonElement

      act(() => {
        trigger.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }))
        vi.advanceTimersByTime(600)
      })
      expect(document.querySelector('.tooltip-layer')).not.toBeNull()

      act(() => {
        button.click()
      })
      expect(document.querySelector('.tooltip-layer')).toBeNull()
    } finally {
      vi.useRealTimers()
    }
  })
})

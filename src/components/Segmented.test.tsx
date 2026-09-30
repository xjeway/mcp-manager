import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it } from 'vitest'
import { Segmented } from './Segmented'
import { Tabs } from './Tabs'

const options = [
  { label: 'A', value: 'a' },
  { label: 'B', value: 'b' },
  { label: 'C', value: 'c' },
]

describe('Segmented', () => {
  it('marks the active option and positions the thumb under it', () => {
    const html = renderToStaticMarkup(<Segmented value="b" options={options} onChange={() => {}} />)

    expect(html).toContain('--segmented-count:3;--segmented-index:1')
    expect(html).toContain('aria-checked="true" class="segmented-option is-active"')
    expect(html.match(/aria-checked="true"/g)).toHaveLength(1)
  })
})

describe('Tabs', () => {
  it('marks the selected tab', () => {
    const html = renderToStaticMarkup(<Tabs value="c" options={options} onChange={() => {}} />)

    expect(html).toContain('aria-selected="true" class="tab is-active"')
    expect(html.match(/aria-selected="true"/g)).toHaveLength(1)
  })
})

import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it } from 'vitest'
import { highlightJson } from './JsonEditor'

describe('highlightJson', () => {
  it('distinguishes keys, strings, numbers and keywords', () => {
    const html = renderToStaticMarkup(<>{highlightJson('{"command": "npx", "port": 8080, "disabled": false}')}</>)

    expect(html).toContain('<span class="json-token-key">&quot;command&quot;</span>')
    expect(html).toContain('<span class="json-token-string">&quot;npx&quot;</span>')
    expect(html).toContain('<span class="json-token-number">8080</span>')
    expect(html).toContain('<span class="json-token-keyword">false</span>')
  })

  it('keeps every character of incomplete input', () => {
    const text = '{\n  "args": ["a\\"b", '
    const html = renderToStaticMarkup(<pre>{highlightJson(text)}</pre>)
    const plain = html.replace(/<[^>]+>/g, '').replace(/&quot;/g, '"')

    expect(plain).toBe(text)
  })
})

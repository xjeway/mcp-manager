import type { ReactNode } from 'react'
import { useRef } from 'react'

type TokenKind = 'key' | 'string' | 'number' | 'keyword' | 'punct'

const TOKEN_PATTERN = /("(?:\\.|[^"\\])*")(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?|[{}[\],:]/g

/** Splits JSON-ish text into highlighted spans. Tolerates invalid JSON while the user is typing. */
export function highlightJson(text: string): ReactNode[] {
  const nodes: ReactNode[] = []
  let cursor = 0

  for (const match of text.matchAll(TOKEN_PATTERN)) {
    const index = match.index ?? 0
    if (index > cursor) {
      nodes.push(text.slice(cursor, index))
    }

    const [token, quoted, colon, keyword] = match
    let kind: TokenKind
    if (quoted !== undefined) {
      kind = colon !== undefined ? 'key' : 'string'
    } else if (keyword !== undefined) {
      kind = 'keyword'
    } else if (/^[{}[\],:]$/.test(token)) {
      kind = 'punct'
    } else {
      kind = 'number'
    }

    if (kind === 'key') {
      nodes.push(
        <span key={index} className="json-token-key">
          {quoted}
        </span>,
        <span key={`${index}:`} className="json-token-punct">
          {colon}
        </span>,
      )
    } else {
      nodes.push(
        <span key={index} className={`json-token-${kind}`}>
          {token}
        </span>,
      )
    }
    cursor = index + token.length
  }

  if (cursor < text.length) {
    nodes.push(text.slice(cursor))
  }
  return nodes
}

const MIN_LINES = 8
const MAX_LINES = 20

interface JsonEditorProps {
  ariaLabel: string
  placeholder?: string
  value: string
  onChange: (value: string) => void
}

/**
 * A plain textarea layered over a highlighted copy of its text, with a line
 * number gutter. The textarea keeps native editing, undo and selection; the
 * layer underneath only paints colours, so both must share identical metrics.
 */
export function JsonEditor({ ariaLabel, placeholder, value, onChange }: JsonEditorProps) {
  const highlightRef = useRef<HTMLPreElement | null>(null)
  const gutterRef = useRef<HTMLDivElement | null>(null)

  const lineCount = Math.max(value.split('\n').length, 1)
  // One spare line so a full editor does not show a scrollbar for the trailing caret line.
  const visibleLines = Math.min(Math.max(lineCount + 1, MIN_LINES), MAX_LINES)

  const syncScroll = (event: React.UIEvent<HTMLTextAreaElement>) => {
    const { scrollTop, scrollLeft } = event.currentTarget
    if (highlightRef.current) {
      highlightRef.current.scrollTop = scrollTop
      highlightRef.current.scrollLeft = scrollLeft
    }
    if (gutterRef.current) {
      gutterRef.current.scrollTop = scrollTop
    }
  }

  return (
    <div className="json-editor" style={{ '--json-editor-lines': visibleLines } as React.CSSProperties}>
      <div ref={gutterRef} className="json-editor-gutter" aria-hidden="true">
        {Array.from({ length: lineCount }, (_, index) => (
          <div key={index}>{index + 1}</div>
        ))}
      </div>
      <div className="json-editor-code">
        <pre ref={highlightRef} className="json-editor-highlight" aria-hidden="true">
          {highlightJson(value)}
          {/* Keeps the layer as tall as the textarea when the text ends with a newline. */}
          {'\n'}
        </pre>
        <textarea
          className="json-editor-input"
          aria-label={ariaLabel}
          value={value}
          placeholder={placeholder}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          wrap="off"
          onChange={(event) => onChange(event.target.value)}
          onScroll={syncScroll}
        />
      </div>
    </div>
  )
}

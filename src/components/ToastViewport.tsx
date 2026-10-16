import { AlertCircle, CheckCircle2, Info, TriangleAlert, X } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { useTranslation } from 'react-i18next'
import type { FeedbackItem } from '../view-models/workspace'

interface ToastViewportProps {
  items: FeedbackItem[]
  onDismiss: (id: string) => void
}

function toastIcon(kind: FeedbackItem['kind']) {
  switch (kind) {
    case 'success':
      return <CheckCircle2 size={16} />
    case 'warning':
      return <TriangleAlert size={16} />
    case 'error':
      return <AlertCircle size={16} />
    default:
      return <Info size={16} />
  }
}

/** Errors and warnings usually need the user to act, so they stay until dismissed. */
export function toastDuration(kind: FeedbackItem['kind']): number | null {
  return kind === 'error' || kind === 'warning' ? null : 4000
}

function isUrgent(kind: FeedbackItem['kind']) {
  return kind === 'error' || kind === 'warning'
}

function useWindowFocused() {
  const [focused, setFocused] = useState(() => typeof document === 'undefined' || document.hasFocus())

  useEffect(() => {
    const onFocus = () => setFocused(true)
    const onBlur = () => setFocused(false)
    window.addEventListener('focus', onFocus)
    window.addEventListener('blur', onBlur)
    return () => {
      window.removeEventListener('focus', onFocus)
      window.removeEventListener('blur', onBlur)
    }
  }, [])

  return focused
}

function ToastItem({
  item,
  paused,
  onDismiss,
}: {
  item: FeedbackItem
  paused: boolean
  onDismiss: (id: string) => void
}) {
  const { t } = useTranslation()
  const [hovered, setHovered] = useState(false)
  const [focusedWithin, setFocusedWithin] = useState(false)
  const duration = item.persistent ? null : toastDuration(item.kind)
  const remaining = useRef(duration ?? 0)

  // Count down only while the toast can actually be read: not while the
  // pointer or keyboard focus is on it, and not while the window is in the background.
  const running = duration !== null && !paused && !hovered && !focusedWithin

  useEffect(() => {
    if (!running) {
      return
    }

    const startedAt = Date.now()
    const timeout = window.setTimeout(() => onDismiss(item.id), remaining.current)
    return () => {
      window.clearTimeout(timeout)
      remaining.current = Math.max(0, remaining.current - (Date.now() - startedAt))
    }
  }, [running, item.id, onDismiss])

  return (
    <article
      className={`toast-card toast-${item.kind}`}
      role={isUrgent(item.kind) ? 'alert' : 'status'}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onFocus={() => setFocusedWithin(true)}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
          setFocusedWithin(false)
        }
      }}
    >
      <div className="toast-icon">{toastIcon(item.kind)}</div>
      <div className="toast-body">
        <p>{item.message}</p>
        {item.action ? (
          <button
            type="button"
            className="toast-action"
            onClick={() => {
              item.action?.onClick()
              onDismiss(item.id)
            }}
          >
            {item.action.label}
          </button>
        ) : null}
      </div>
      <button
        type="button"
        className="toast-close"
        onClick={() => onDismiss(item.id)}
        aria-label={t('dismissNotification')}
      >
        <X size={14} />
      </button>
    </article>
  )
}

export function ToastViewport({ items, onDismiss }: ToastViewportProps) {
  const windowFocused = useWindowFocused()

  if (typeof document === 'undefined' || items.length === 0) {
    return null
  }

  return createPortal(
    <section className="toast-viewport" aria-live="polite" aria-atomic="false">
      {items.map((item) => (
        <ToastItem key={item.id} item={item} paused={!windowFocused} onDismiss={onDismiss} />
      ))}
    </section>,
    document.body,
  )
}

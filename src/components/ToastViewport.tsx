import { AlertCircle, CheckCircle2, Info, TriangleAlert, X } from 'lucide-react'
import { useEffect } from 'react'
import { createPortal } from 'react-dom'
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

function toastDuration(kind: FeedbackItem['kind']) {
  switch (kind) {
    case 'error':
      return 5600
    case 'warning':
      return 4600
    default:
      return 3600
  }
}

function ToastItem({
  item,
  onDismiss,
}: {
  item: FeedbackItem
  onDismiss: (id: string) => void
}) {
  useEffect(() => {
    const timeout = window.setTimeout(() => onDismiss(item.id), toastDuration(item.kind))
    return () => window.clearTimeout(timeout)
  }, [item.id, item.kind, onDismiss])

  return (
    <article className={`toast-card toast-${item.kind}`}>
      <div className="toast-icon">{toastIcon(item.kind)}</div>
      <div className="toast-body">
        <p>{item.message}</p>
      </div>
      <button
        type="button"
        className="toast-close"
        onClick={() => onDismiss(item.id)}
        aria-label="Dismiss notification"
      >
        <X size={14} />
      </button>
    </article>
  )
}

export function ToastViewport({ items, onDismiss }: ToastViewportProps) {
  if (typeof document === 'undefined' || items.length === 0) {
    return null
  }

  return createPortal(
    <section className="toast-viewport" aria-live="polite" aria-atomic="false">
      {items.map((item) => (
        <ToastItem key={item.id} item={item} onDismiss={onDismiss} />
      ))}
    </section>,
    document.body,
  )
}

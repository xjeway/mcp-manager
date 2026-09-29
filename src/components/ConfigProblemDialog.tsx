import { AlertCircle } from 'lucide-react'
import { useEffect, useRef, type KeyboardEvent } from 'react'
import { useTranslation } from 'react-i18next'

interface ConfigProblemDialogProps {
  path: string
  detail: string
  busy: boolean
  /** Servers still shown from before the file broke, which can be saved back. */
  restorableCount: number
  onRestore: () => void
  onShowFile: () => void
  onReload: () => void
  onStartOver: () => void
}

/**
 * Covers the app while servers.yaml cannot be read: the list behind it is
 * missing or stale, and saving is refused until the file is fixed or replaced.
 */
export function ConfigProblemDialog({
  path,
  detail,
  busy,
  restorableCount,
  onRestore,
  onShowFile,
  onReload,
  onStartOver,
}: ConfigProblemDialogProps) {
  const { t } = useTranslation()
  const dialogRef = useRef<HTMLElement>(null)
  const reloadRef = useRef<HTMLButtonElement>(null)

  // Focus starts on Reload, the one choice that changes nothing, so a stray
  // Enter never starts over.
  useEffect(() => {
    ;(reloadRef.current ?? dialogRef.current)?.focus()
  }, [])

  // The app behind is covered, so Tab cycles through the dialog's buttons.
  const keepFocusInside = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key !== 'Tab') {
      return
    }
    const buttons = Array.from(dialogRef.current?.querySelectorAll('button:not(:disabled)') ?? [])
    if (buttons.length === 0) {
      event.preventDefault()
      return
    }
    const first = buttons[0] as HTMLElement
    const last = buttons[buttons.length - 1] as HTMLElement
    const active = document.activeElement
    if (event.shiftKey && (active === first || !buttons.includes(active as Element))) {
      event.preventDefault()
      last.focus()
    } else if (!event.shiftKey && (active === last || !buttons.includes(active as Element))) {
      event.preventDefault()
      first.focus()
    }
  }

  return (
    <div className="config-problem-backdrop">
      <section
        ref={dialogRef}
        className="config-problem-card"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="config-problem-title"
        aria-describedby="config-problem-body"
        tabIndex={-1}
        onKeyDown={keepFocusInside}
      >
        <header className="config-problem-header">
          <AlertCircle size={18} aria-hidden="true" />
          <h2 id="config-problem-title">{t('configUnreadableTitle')}</h2>
        </header>
        <p id="config-problem-body">{t('configUnreadableBody')}</p>
        <code className="config-problem-path">{path}</code>
        <pre className="config-problem-detail">{detail}</pre>
        <div className="config-problem-actions">
          <button type="button" className="ghost-button compact danger" disabled={busy} onClick={onStartOver}>
            {t('configUnreadableStartOver')}
          </button>
          {restorableCount > 0 ? (
            <button type="button" className="ghost-button compact" disabled={busy} onClick={onRestore}>
              {t('configUnreadableRestore', { count: restorableCount })}
            </button>
          ) : null}
          <span className="config-problem-spacer" />
          <button type="button" className="ghost-button compact" disabled={busy} onClick={onShowFile}>
            {t('configUnreadableShowFile')}
          </button>
          <button ref={reloadRef} type="button" className="primary-button" disabled={busy} onClick={onReload}>
            {t('configUnreadableReload')}
          </button>
        </div>
      </section>
    </div>
  )
}

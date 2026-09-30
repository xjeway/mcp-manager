import type { ReactNode } from 'react'
import { useEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'

const OPEN_DELAY_MS = 450
// Once one tooltip has been seen, neighbours open instantly for a short while.
const WARM_WINDOW_MS = 300
let lastClosedAt = 0

interface TooltipProps {
  children: ReactNode
  content: string
}

export function Tooltip({ children, content }: TooltipProps) {
  const triggerRef = useRef<HTMLSpanElement | null>(null)
  const [open, setOpen] = useState(false)
  const timerRef = useRef<number | null>(null)
  const [position, setPosition] = useState({ left: 0, top: 0 })
  const [placement, setPlacement] = useState<'top' | 'bottom'>('top')

  const syncPosition = () => {
    const rect = triggerRef.current?.getBoundingClientRect()
    if (!rect) {
      return
    }

    const nextPlacement = rect.top < 40 ? 'bottom' : 'top'
    setPlacement(nextPlacement)
    setPosition({
      left: rect.left + rect.width / 2,
      top: nextPlacement === 'top' ? rect.top - 14 : rect.bottom + 14,
    })
  }

  useEffect(() => {
    if (!open) {
      return
    }

    const handleViewportChange = () => {
      syncPosition()
    }

    window.addEventListener('scroll', handleViewportChange, true)
    window.addEventListener('resize', handleViewportChange)

    return () => {
      window.removeEventListener('scroll', handleViewportChange, true)
      window.removeEventListener('resize', handleViewportChange)
    }
  }, [open])

  const show = () => {
    syncPosition()
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current)
    }
    const warm = Date.now() - lastClosedAt < WARM_WINDOW_MS
    if (warm) {
      setOpen(true)
      return
    }
    timerRef.current = window.setTimeout(() => setOpen(true), OPEN_DELAY_MS)
  }

  const hide = () => {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current)
      timerRef.current = null
    }
    if (open) {
      lastClosedAt = Date.now()
    }
    setOpen(false)
  }

  useEffect(
    () => () => {
      if (timerRef.current !== null) {
        window.clearTimeout(timerRef.current)
      }
    },
    [],
  )

  return (
    <>
      <span
        ref={triggerRef}
        className="tooltip-trigger"
        onMouseEnter={show}
        onMouseLeave={hide}
        onFocus={show}
        onBlur={hide}
        onClick={hide}
      >
        {children}
      </span>
      {open && typeof document !== 'undefined'
        ? createPortal(
            <span
              className="tooltip-layer"
              style={{
                left: `${position.left}px`,
                top: `${position.top}px`,
              }}
              data-placement={placement}
            >
              {content}
            </span>,
            document.body,
          )
        : null}
    </>
  )
}

import type { ReactNode } from 'react'
import { useEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'

interface TooltipProps {
  children: ReactNode
  content: string
}

export function Tooltip({ children, content }: TooltipProps) {
  const triggerRef = useRef<HTMLSpanElement | null>(null)
  const [open, setOpen] = useState(false)
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

  return (
    <>
      <span
        ref={triggerRef}
        className="tooltip-trigger"
        onMouseEnter={() => {
          syncPosition()
          setOpen(true)
        }}
        onMouseLeave={() => setOpen(false)}
        onFocus={() => {
          syncPosition()
          setOpen(true)
        }}
        onBlur={() => setOpen(false)}
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

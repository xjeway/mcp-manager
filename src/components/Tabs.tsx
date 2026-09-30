import type { KeyboardEvent, ReactNode } from 'react'
import { useRef } from 'react'
import { nextIndexForKey } from './rovingFocus'

export interface TabOption<T extends string> {
  icon?: ReactNode
  label: string
  value: T
}

interface TabsProps<T extends string> {
  ariaLabel?: string
  className?: string
  /** Stable prefix shared with the panel: see `tabId` and `tabPanelId`. */
  idPrefix: string
  options: Array<TabOption<T>>
  value: T
  onChange: (value: T) => void
}

export function tabId(idPrefix: string, value: string) {
  return `${idPrefix}-tab-${value}`
}

export function tabPanelId(idPrefix: string) {
  return `${idPrefix}-panel`
}

/** Underlined tabs for switching between views or a variable-length list of sources. */
export function Tabs<T extends string>({ ariaLabel, className, idPrefix, options, value, onChange }: TabsProps<T>) {
  const buttonRefs = useRef<Array<HTMLButtonElement | null>>([])
  const activeIndex = Math.max(
    0,
    options.findIndex((option) => option.value === value),
  )

  const handleKeyDown = (event: KeyboardEvent, index: number) => {
    const next = nextIndexForKey(event, index, options.length)
    if (next === null) {
      return
    }
    event.preventDefault()
    buttonRefs.current[next]?.focus()
    onChange(options[next].value)
  }

  return (
    <div className={className ? `tabs ${className}` : 'tabs'} role="tablist" aria-label={ariaLabel}>
      {options.map((option, index) => (
        <button
          key={option.value}
          ref={(node) => {
            buttonRefs.current[index] = node
          }}
          id={tabId(idPrefix, option.value)}
          type="button"
          role="tab"
          aria-selected={option.value === value}
          aria-controls={tabPanelId(idPrefix)}
          tabIndex={index === activeIndex ? 0 : -1}
          className={option.value === value ? 'tab is-active' : 'tab'}
          onClick={() => onChange(option.value)}
          onKeyDown={(event) => handleKeyDown(event, index)}
        >
          {option.icon}
          <span>{option.label}</span>
        </button>
      ))}
    </div>
  )
}

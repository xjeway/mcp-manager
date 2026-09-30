import type { CSSProperties, KeyboardEvent, ReactNode } from 'react'
import { useRef } from 'react'
import { nextIndexForKey } from './rovingFocus'

export interface SegmentedOption<T extends string> {
  icon?: ReactNode
  label: string
  value: T
}

interface SegmentedProps<T extends string> {
  ariaLabel?: string
  options: Array<SegmentedOption<T>>
  value: T
  onChange: (value: T) => void
}

/** A fixed set of 2-3 mutually exclusive values, with a thumb that slides under the active one. */
export function Segmented<T extends string>({ ariaLabel, options, value, onChange }: SegmentedProps<T>) {
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
    <div
      className="segmented"
      role="radiogroup"
      aria-label={ariaLabel}
      style={{ '--segmented-count': options.length, '--segmented-index': activeIndex } as CSSProperties}
    >
      <span className="segmented-thumb" aria-hidden="true" />
      {options.map((option, index) => (
        <button
          key={option.value}
          ref={(node) => {
            buttonRefs.current[index] = node
          }}
          type="button"
          role="radio"
          aria-checked={option.value === value}
          tabIndex={index === activeIndex ? 0 : -1}
          className={option.value === value ? 'segmented-option is-active' : 'segmented-option'}
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

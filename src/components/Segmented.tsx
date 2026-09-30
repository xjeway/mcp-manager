import type { CSSProperties, ReactNode } from 'react'

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
  const activeIndex = Math.max(
    0,
    options.findIndex((option) => option.value === value),
  )

  return (
    <div
      className="segmented"
      role="radiogroup"
      aria-label={ariaLabel}
      style={{ '--segmented-count': options.length, '--segmented-index': activeIndex } as CSSProperties}
    >
      <span className="segmented-thumb" aria-hidden="true" />
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          role="radio"
          aria-checked={option.value === value}
          className={option.value === value ? 'segmented-option is-active' : 'segmented-option'}
          onClick={() => onChange(option.value)}
        >
          {option.icon}
          <span>{option.label}</span>
        </button>
      ))}
    </div>
  )
}

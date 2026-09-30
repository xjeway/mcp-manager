import type { ReactNode } from 'react'

export interface TabOption<T extends string> {
  icon?: ReactNode
  label: string
  value: T
}

interface TabsProps<T extends string> {
  ariaLabel?: string
  className?: string
  options: Array<TabOption<T>>
  value: T
  onChange: (value: T) => void
}

/** Underlined tabs for switching between views or a variable-length list of sources. */
export function Tabs<T extends string>({ ariaLabel, className, options, value, onChange }: TabsProps<T>) {
  return (
    <div className={className ? `tabs ${className}` : 'tabs'} role="tablist" aria-label={ariaLabel}>
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          role="tab"
          aria-selected={option.value === value}
          className={option.value === value ? 'tab is-active' : 'tab'}
          onClick={() => onChange(option.value)}
        >
          {option.icon}
          <span>{option.label}</span>
        </button>
      ))}
    </div>
  )
}

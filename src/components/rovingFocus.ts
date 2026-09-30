import type { KeyboardEvent } from 'react'

/**
 * Arrow-key navigation for a roving-tabindex group. Returns the index that
 * should receive focus, or null when the key is not a navigation key.
 */
export function nextIndexForKey(event: KeyboardEvent, current: number, count: number): number | null {
  switch (event.key) {
    case 'ArrowRight':
    case 'ArrowDown':
      return (current + 1) % count
    case 'ArrowLeft':
    case 'ArrowUp':
      return (current - 1 + count) % count
    case 'Home':
      return 0
    case 'End':
      return count - 1
    default:
      return null
  }
}

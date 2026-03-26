import { describe, expect, it } from 'vitest'
import { deriveVisibleApps } from './visibleApps'

describe('visibleApps', () => {
  it('returns installed apps in canonical UI order', () => {
    expect(deriveVisibleApps(new Set(['cursor', 'vscode']))).toEqual(['vscode', 'cursor'])
  })

  it('ignores unknown app ids', () => {
    expect(deriveVisibleApps(new Set(['cursor', 'unknown-app']))).toEqual(['cursor'])
  })
})

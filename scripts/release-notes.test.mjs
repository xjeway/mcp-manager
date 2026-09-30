import { describe, expect, it } from 'vitest'

import {
  buildReleaseNotes,
  collectSubjects,
  hasReleaseNotes,
  parseSubject,
  previousTag,
} from './release-notes.mjs'

describe('parseSubject', () => {
  it('reads the type, ignoring scope and breaking marker', () => {
    expect(parseSubject('feat(editor): edit HTTP headers')).toEqual({
      type: 'feat',
      text: 'Edit HTTP headers',
    })
    expect(parseSubject('fix!: drop the old path')).toEqual({ type: 'fix', text: 'Drop the old path' })
  })

  it('returns null for subjects without a conventional prefix', () => {
    expect(parseSubject('Merge pull request #30 from xjeway/x')).toBeNull()
  })
})

describe('buildReleaseNotes', () => {
  it('groups features and fixes and leaves the rest out', () => {
    const notes = buildReleaseNotes([
      'fix: keep unknown clients',
      'chore: bump deps',
      'feat: add server search',
      'ci: pin actions',
      'refactor: split modules',
      'feat(marketplace): add custom sources',
    ])

    expect(notes).toBe(
      [
        "## What's new",
        '',
        '* Add server search',
        '* Add custom sources',
        '',
        '## Fixes',
        '',
        '* Keep unknown clients',
      ].join('\n'),
    )
  })

  it('lists a repeated subject once', () => {
    expect(buildReleaseNotes(['fix: same thing', 'fix(cli): same thing'])).toBe(
      '## Fixes\n\n* Same thing',
    )
  })

  it('is empty when nothing is user-facing', () => {
    expect(buildReleaseNotes(['chore: release v1.0.0', 'docs: update readme'])).toBe('')
  })
})

describe('hasReleaseNotes', () => {
  it('recognises generated sections and rejects the bare boilerplate', () => {
    expect(hasReleaseNotes("## What's new\n\n* A")).toBe(true)
    expect(hasReleaseNotes('Fixes\n\n* A')).toBe(false)
    expect(hasReleaseNotes('Automated release for v1.0.0.')).toBe(false)
    expect(hasReleaseNotes(null)).toBe(false)
  })
})

describe('commit range', () => {
  const runner = (responses) => (command, args) => {
    const key = args.join(' ')
    if (!(key in responses)) {
      throw new Error(`unexpected git ${key}`)
    }
    const value = responses[key]
    if (value instanceof Error) {
      throw value
    }
    return value
  }

  it('walks from the previous tag to this one', () => {
    const run = runner({
      'describe --tags --abbrev=0 v0.2.0^': 'v0.1.7\n',
      'log --no-merges --format=%s v0.1.7..v0.2.0': 'feat: a\nfix: b\n',
    })

    expect(previousTag('v0.2.0', run)).toBe('v0.1.7')
    expect(collectSubjects('v0.2.0', run)).toEqual(['feat: a', 'fix: b'])
  })

  it('takes the whole history for the first tag', () => {
    const run = runner({
      'describe --tags --abbrev=0 v0.1.0^': new Error('no tags'),
      'log --no-merges --format=%s v0.1.0': 'feat: first\n',
    })

    expect(previousTag('v0.1.0', run)).toBeNull()
    expect(collectSubjects('v0.1.0', run)).toEqual(['feat: first'])
  })
})

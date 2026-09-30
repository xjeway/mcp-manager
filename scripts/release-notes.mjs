import { execFileSync } from 'node:child_process'
import process from 'node:process'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

import { isPrereleaseRef } from './release-args.mjs'

// Only user-facing commit types reach the notes; chore, ci, test, docs and
// refactor commits are left out.
const sections = [
  { type: 'feat', title: "What's new" },
  { type: 'fix', title: 'Fixes' },
]

const subjectPattern = /^(\w+)(?:\([^)]*\))?!?:\s*(.+)$/

export function parseSubject(subject) {
  const match = subjectPattern.exec(subject.trim())
  if (!match) {
    return null
  }
  const text = match[2].trim()
  return { type: match[1].toLowerCase(), text: text.charAt(0).toUpperCase() + text.slice(1) }
}

// Returns an empty string when no commit is user-facing, so the caller can
// tell "nothing to announce" from a release with notes.
export function buildReleaseNotes(subjects) {
  const parsed = subjects.map(parseSubject).filter(Boolean)
  const blocks = []

  for (const { type, title } of sections) {
    const items = [...new Set(parsed.filter((commit) => commit.type === type).map((commit) => commit.text))]
    if (items.length > 0) {
      blocks.push([`## ${title}`, '', ...items.map((item) => `* ${item}`)].join('\n'))
    }
  }

  return blocks.join('\n\n')
}

export function hasReleaseNotes(body) {
  return sections.some(({ title }) => new RegExp(`^#{1,3} ${title}$`, 'm').test(body ?? ''))
}

function git(args, run = execFileSync) {
  return run('git', args, { encoding: 'utf8' }).trim()
}

// The tag before `tagName` in its own history; null for the first release.
// A stable release counts from the previous stable one, so promoting a
// prerelease (even one on the same commit) still announces the whole cycle
// instead of an empty range.
export function previousTag(tagName, run = execFileSync) {
  const args = ['describe', '--tags', '--abbrev=0']
  if (!isPrereleaseRef(tagName)) {
    args.push('--exclude=*-*')
  }

  try {
    return git([...args, `${tagName}^`], run) || null
  } catch {
    return null
  }
}

export function collectSubjects(tagName, run = execFileSync) {
  const previous = previousTag(tagName, run)
  const range = previous ? `${previous}..${tagName}` : tagName
  const output = git(['log', '--no-merges', '--format=%s', range], run)
  return output ? output.split('\n') : []
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const tagName = process.argv[2]
  if (!tagName) {
    console.error('Usage: node scripts/release-notes.mjs <tag-name>')
    process.exit(1)
  }
  process.stdout.write(buildReleaseNotes(collectSubjects(tagName)))
}

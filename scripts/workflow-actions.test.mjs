import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const repoRoot = path.resolve(import.meta.dirname, '..')
const workflowsDir = path.join(repoRoot, '.github', 'workflows')

function readWorkflow(fileName) {
  return fs.readFileSync(path.join(workflowsDir, fileName), 'utf8')
}

describe('GitHub workflow action runtimes', () => {
  // Floating tags move. A full commit SHA keeps the major that understands Node 24.
  const pinned = (action) => new RegExp(`uses:\\s+${action}@[0-9a-f]{40}\\s+#\\s+v([5-9]|\\d{2,})\\b`)

  it('pins actions/checkout to a Node24-compatible commit', () => {
    for (const fileName of ['ci.yml', 'homebrew.yml', 'release.yml', 'issue-triage.yml']) {
      expect(readWorkflow(fileName)).toMatch(pinned('actions/checkout'))
    }
  })

  it('pins actions/setup-node to a Node24-compatible commit', () => {
    for (const fileName of ['ci.yml', 'homebrew.yml', 'release.yml']) {
      expect(readWorkflow(fileName)).toMatch(pinned('actions/setup-node'))
    }
  })
})

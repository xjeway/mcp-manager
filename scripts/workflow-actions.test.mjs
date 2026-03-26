import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const repoRoot = path.resolve(import.meta.dirname, '..')
const workflowsDir = path.join(repoRoot, '.github', 'workflows')

function readWorkflow(fileName) {
  return fs.readFileSync(path.join(workflowsDir, fileName), 'utf8')
}

describe('GitHub workflow action runtimes', () => {
  it('uses a Node24-compatible major for actions/checkout', () => {
    for (const fileName of ['ci.yml', 'release.yml']) {
      const contents = readWorkflow(fileName)
      expect(contents).toMatch(/uses:\s+actions\/checkout@v([5-9]|\d{2,})\b/)
    }
  })

  it('uses a Node24-compatible major for actions/setup-node', () => {
    for (const fileName of ['ci.yml', 'release.yml']) {
      const contents = readWorkflow(fileName)
      expect(contents).toMatch(/uses:\s+actions\/setup-node@v([5-9]|\d{2,})\b/)
    }
  })
})

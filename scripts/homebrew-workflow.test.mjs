import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import YAML from 'yaml'

const repoRoot = path.resolve(import.meta.dirname, '..')
const workflowPath = path.join(repoRoot, '.github', 'workflows', 'homebrew.yml')

function readWorkflow() {
  return YAML.parse(fs.readFileSync(workflowPath, 'utf8'))
}

describe('Homebrew workflow structure', () => {
  it('runs only after a GitHub release is published', () => {
    const workflow = readWorkflow()

    expect(workflow.on.release.types).toEqual(['published'])
  })

  it('can be manually dispatched for an existing release tag', () => {
    const workflow = readWorkflow()

    expect(workflow.on.workflow_dispatch.inputs.release_tag.required).toBe(true)
    expect(workflow.on.workflow_dispatch.inputs.release_tag.description).toContain('v0.1.6')
  })

  it('can be called by the release workflow with the tap PAT', () => {
    const workflow = readWorkflow()

    expect(workflow.on.workflow_call.inputs.release_tag.required).toBe(true)
    expect(workflow.on.workflow_call.secrets.HOMEBREW_TAP_PAT.required).toBe(true)
  })

  it('uses the tap PAT and updates the external cask repository', () => {
    const workflow = readWorkflow()
    const job = workflow.jobs['publish-homebrew-cask']

    expect(job).toBeDefined()
    expect(job.permissions.contents).toBe('read')

    const steps = job.steps ?? []
    expect(
      steps.some(
        (step) =>
          step.uses?.startsWith('actions/checkout@') &&
          step.with?.repository === 'xjeway/homebrew-mcp-manager' &&
          step.with?.token === '${{ secrets.HOMEBREW_TAP_PAT }}',
      ),
    ).toBe(true)
    expect(
      steps.some(
        (step) =>
          typeof step.run === 'string' &&
          step.run.includes('node scripts/homebrew-cask.mjs publish') &&
          step.run.includes('--tag "$RELEASE_TAG"'),
      ),
    ).toBe(true)
  })
})

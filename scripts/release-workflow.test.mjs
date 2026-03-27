import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import YAML from 'yaml'

const repoRoot = path.resolve(import.meta.dirname, '..')
const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release.yml')

function readWorkflow() {
  return YAML.parse(fs.readFileSync(workflowPath, 'utf8'))
}

function findTauriSteps(job) {
  return (job.steps ?? []).filter((step) => step.uses === 'tauri-apps/tauri-action@v0.6.0')
}

describe('release workflow structure', () => {
  it('prepares the release once before the matrix uploads run', () => {
    const workflow = readWorkflow()

    expect(workflow.jobs['prepare-release']).toBeDefined()
    expect(workflow.jobs['publish-tauri'].needs).toBe('prepare-release')
  })

  it('uploads matrix artifacts by release id and disables updater json in matrix jobs', () => {
    const workflow = readWorkflow()
    const tauriSteps = findTauriSteps(workflow.jobs['publish-tauri'])

    expect(tauriSteps.length).toBeGreaterThan(0)

    for (const step of tauriSteps) {
      expect(step.with.releaseId).toBe('${{ needs.prepare-release.outputs.release_id }}')
      expect(step.with.includeUpdaterJson).toBe(false)
      expect(step.with.tagName).toBeUndefined()
    }
  })

  it('publishes updater metadata in a dedicated final job', () => {
    const workflow = readWorkflow()
    const updaterJob = workflow.jobs['publish-updater']

    expect(updaterJob).toBeDefined()
    expect(updaterJob.needs).toEqual(['prepare-release', 'publish-tauri'])
    expect(
      updaterJob.steps.some(
        (step) =>
          typeof step.run === 'string' &&
          step.run.includes('node scripts/github-release.mjs upload-updater'),
      ),
    ).toBe(true)
  })
})

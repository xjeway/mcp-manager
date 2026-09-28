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
  return (job.steps ?? []).filter((step) => step.uses?.startsWith('tauri-apps/tauri-action@'))
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

  it('publishes the Homebrew cask after the release is complete', () => {
    const workflow = readWorkflow()
    const job = workflow.jobs['publish-homebrew']

    expect(job.uses).toBe('./.github/workflows/homebrew.yml')
    expect(job.needs).toBe('publish-updater')
    expect(job.with.release_tag).toBe('${{ github.ref_name }}')
    expect(job.secrets.HOMEBREW_TAP_PAT).toBe('${{ secrets.HOMEBREW_TAP_PAT }}')
  })

  // tauri-action v1 renames includeUpdaterJson, adds the version to .app.tar.gz
  // names (which the Homebrew cask relies on) and rewrites release notes.
  it('stays on tauri-action v0 until the release pipeline is migrated to v1', () => {
    const workflow = readWorkflow()
    const steps = findTauriSteps(workflow.jobs['publish-tauri'])

    expect(steps.length).toBeGreaterThan(0)
    for (const step of steps) {
      expect(step.uses).toMatch(/^tauri-apps\/tauri-action@v0\./)
    }
  })
})

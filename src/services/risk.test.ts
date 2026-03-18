import { describe, expect, it } from 'vitest'
import { evaluateApplyRisks } from './risk'

describe('evaluateApplyRisks', () => {
  it('returns blocking error for invalid http url', () => {
    const summary = evaluateApplyRisks([
      {
        id: 'bad-url',
        name: 'Bad URL',
        enabled: true,
        transport: { type: 'http', url: 'ftp://invalid' },
        apps: { vscode: true, cursor: false, claudeCode: false, codex: false },
      },
    ])

    expect(summary.blockingErrors.length).toBeGreaterThan(0)
  })
})

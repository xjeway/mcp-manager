import { describe, expect, it } from 'vitest'
import { shouldShowClientRedetectHint } from './clientDetectionUi'

describe('clientDetectionUi', () => {
  it('shows the redetect hint when some supported clients are hidden by detection', () => {
    expect(shouldShowClientRedetectHint(['cursor', 'vscode'])).toBe(true)
  })

  it('hides the redetect hint when all supported clients are visible', () => {
    expect(
      shouldShowClientRedetectHint([
        'vscode',
        'cursor',
        'claudeCode',
        'claudeDesktop',
        'codex',
        'openCode',
        'githubCopilot',
        'geminiCli',
        'antigravity',
        'iFlow',
        'qwenCode',
        'cline',
        'windsurf',
        'kiro',
      ]),
    ).toBe(false)
  })
})

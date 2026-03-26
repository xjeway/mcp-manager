import { describe, expect, it } from 'vitest'
import { mapConfigToWorkspaceView, serverToEditorDraft, editorDraftToServer } from './workspace'

describe('workspace view-models', () => {
  it('maps canonical config into workspace stats and rows', () => {
    const view = mapConfigToWorkspaceView(
      {
        version: 1,
        servers: [
          {
            id: 'github',
            name: 'GitHub',
            enabled: true,
            transport: { type: 'stdio' },
            command: { program: 'uvx', args: ['mcp-server-github'], env: {} },
            apps: {
              vscode: true,
              cursor: false,
              claudeCode: true,
              claudeDesktop: false,
              codex: false,
              openCode: false,
              githubCopilot: true,
              geminiCli: false,
              antigravity: false,
              iFlow: false,
              qwenCode: false,
              cline: false,
              windsurf: false,
              kiro: false,
            },
          },
        ],
      },
      ['vscode', 'cursor'],
      ((key: string) => key) as never,
    )

    expect(view.rows[0].copyValue).toContain('uvx')
    expect(view.rows[0].enabledApps).toEqual(['vscode'])
    expect(view.stats.map((stat) => stat.id)).toEqual(['cursor', 'vscode'])
    expect(view.stats.find((stat) => stat.id === 'vscode')?.count).toBe(1)
    expect(view.stats.find((stat) => stat.id === 'cursor')?.count).toBe(0)
  })

  it('round-trips a server through the editor draft mapper', () => {
    const draft = serverToEditorDraft({
      id: 'filesystem',
      name: 'Filesystem',
      enabled: true,
      transport: { type: 'http', url: 'https://example.com/mcp' },
      apps: {
        vscode: false,
        cursor: true,
        claudeCode: false,
        claudeDesktop: false,
        codex: true,
        openCode: true,
        githubCopilot: false,
        geminiCli: false,
        antigravity: false,
        iFlow: false,
        qwenCode: false,
        cline: false,
        windsurf: false,
        kiro: true,
      },
    })

    const restored = editorDraftToServer(draft)
    expect(restored.transport.type).toBe('http')
    expect(restored.transport.url).toBe('https://example.com/mcp')
    expect(restored.apps.cursor).toBe(true)
    expect(restored.apps.codex).toBe(true)
    expect(restored.apps.kiro).toBe(true)
  })

  it('orders visible stats and enabled apps by client name', () => {
    const view = mapConfigToWorkspaceView(
      {
        version: 1,
        servers: [
          {
            id: 'mixed-order',
            name: 'Mixed Order',
            enabled: true,
            transport: { type: 'stdio' },
            command: { program: 'npx', args: ['example'], env: {} },
            apps: {
              vscode: true,
              cursor: true,
              claudeCode: false,
              claudeDesktop: false,
              codex: false,
              openCode: false,
              githubCopilot: false,
              geminiCli: false,
              antigravity: false,
              iFlow: false,
              qwenCode: false,
              cline: false,
              windsurf: false,
              kiro: true,
            },
          },
        ],
      },
      ['vscode', 'kiro', 'cursor'],
      ((key: string) => key) as never,
    )

    expect(view.stats.map((stat) => stat.id)).toEqual(['cursor', 'kiro', 'vscode'])
    expect(view.rows[0].enabledApps).toEqual(['cursor', 'kiro', 'vscode'])
  })
})

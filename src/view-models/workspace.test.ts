import { describe, expect, it } from 'vitest'
import {
  editorDraftToServer,
  filterWorkspaceRows,
  retainVisibleSelection,
  mapConfigToWorkspaceView,
  serverToEditorDraft,
  setServerAppEnabled,
  setWorkspacePlacement,
  workspacePlacementPath,
} from './workspace'
import type { MCPServer } from '../types/config'
import type { WorkspaceRowViewModel } from './workspace'

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
              qoder: false,
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
        qoder: false,
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
              qoder: false,
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

  it('maps placements into workspace rows and stats', () => {
    const view = mapConfigToWorkspaceView(
      {
        version: 1,
        servers: [
          {
            id: 'playwright',
            name: 'Playwright',
            enabled: true,
            transport: { type: 'stdio' },
            command: { program: 'npx', args: ['@playwright/mcp@latest'], env: {} },
            apps: {
              vscode: false,
              cursor: false,
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
              kiro: false,
              qoder: false,
            },
            placements: [
              {
                app: 'vscode',
                scope: 'workspace',
                path: '/workspace/project/.vscode/mcp.json',
                enabled: true,
              },
              {
                app: 'cursor',
                scope: 'user',
                path: '/Users/test/.cursor/mcp.json',
                enabled: true,
              },
            ],
          },
        ],
      },
      ['vscode', 'cursor'],
      ((key: string) => key) as never,
    )

    expect(view.rows[0].enabledApps).toEqual(['cursor', 'vscode'])
    expect(view.rows[0].placements).toEqual([
      {
        app: 'cursor',
        label: 'Cursor',
        path: '/Users/test/.cursor/mcp.json',
        scope: 'user',
        scopeLabel: 'placementScopeUser',
      },
      {
        app: 'vscode',
        label: 'VS Code',
        path: '/workspace/project/.vscode/mcp.json',
        scope: 'workspace',
        scopeLabel: 'placementScopeWorkspace',
      },
    ])
    expect(view.stats.find((stat) => stat.id === 'vscode')?.count).toBe(1)
    expect(view.stats.find((stat) => stat.id === 'cursor')?.count).toBe(1)
  })

  it('reads workspace placement paths from the backend-provided table', () => {
    const workspace = {
      root: '/workspace/project',
      placementPaths: { vscode: '/workspace/project/.vscode/mcp.json' },
    }

    expect(workspacePlacementPath('vscode', workspace)).toBe('/workspace/project/.vscode/mcp.json')
    expect(workspacePlacementPath('codex', workspace)).toBeNull()
    expect(workspacePlacementPath('vscode', { root: '', placementPaths: {} })).toBeNull()
  })

  it('adds or updates only the matching workspace placement', () => {
    const userPlacement = {
      app: 'vscode' as const,
      scope: 'user' as const,
      path: '/Users/test/Library/Application Support/Code/User/mcp.json',
      enabled: true,
      managed: true,
    }
    const workspacePath = '/workspace/project/.vscode/mcp.json'

    const added = setWorkspacePlacement([userPlacement], 'vscode', workspacePath, true)
    expect(added).toEqual([
      userPlacement,
      { app: 'vscode', scope: 'workspace', path: workspacePath, enabled: true, managed: true },
    ])

    const disabled = setWorkspacePlacement(added, 'vscode', workspacePath, false)
    expect(disabled).toEqual([
      userPlacement,
      { app: 'vscode', scope: 'workspace', path: workspacePath, enabled: false, managed: true },
    ])
  })

  it('turns an app off at every scope but back on only at user level', () => {
    const server: MCPServer = {
      id: 'playwright',
      name: 'Playwright',
      enabled: true,
      transport: { type: 'stdio' },
      command: { program: 'npx', args: ['@playwright/mcp@latest'], env: {} },
      apps: {
        vscode: true,
        cursor: false,
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
        kiro: false,
        qoder: false,
      },
      placements: [
        { app: 'vscode', scope: 'user', path: '/user/mcp.json', enabled: true, managed: true },
        { app: 'vscode', scope: 'workspace', path: '/workspace/project/.vscode/mcp.json', enabled: true, managed: true },
      ],
    }

    const off = setServerAppEnabled(server, 'vscode', false, { includeWorkspace: true })
    expect(off.apps.vscode).toBe(false)
    expect(off.placements?.map((placement) => placement.enabled)).toEqual([false, false])

    const on = setServerAppEnabled(off, 'vscode', true, { includeWorkspace: true })
    expect(on.apps.vscode).toBe(true)
    expect(on.placements?.map((placement) => placement.enabled)).toEqual([true, false])

    const userOnlyOff = setServerAppEnabled(server, 'vscode', false)
    expect(userOnlyOff.placements?.map((placement) => placement.enabled)).toEqual([false, true])
  })
})

describe('filterWorkspaceRows', () => {
  const row = (id: string, name: string, copyValue: string, enabledApps: WorkspaceRowViewModel['enabledApps'] = []) => ({
    id,
    name,
    copyValue,
    enabledApps,
    transportLabel: copyValue.startsWith('http') ? 'HTTP' : 'STDIO',
    placements: [],
  })
  const rows: WorkspaceRowViewModel[] = [
    row('github', 'GitHub', 'uvx mcp-server-github', ['vscode', 'cursor']),
    row('filesystem', 'Filesystem', 'npx @modelcontextprotocol/server-filesystem', ['cursor']),
    row('linear', 'Linear', 'https://mcp.linear.app/sse', []),
  ]
  const ids = (result: WorkspaceRowViewModel[]) => result.map((item) => item.id)

  it('returns every row when the query is blank and no app is selected', () => {
    expect(ids(filterWorkspaceRows(rows, { query: '   ', app: null }))).toEqual(['github', 'filesystem', 'linear'])
  })

  it('matches the server name case-insensitively', () => {
    expect(ids(filterWorkspaceRows(rows, { query: 'GITHUB', app: null }))).toEqual(['github'])
  })

  it('matches the command or url summary', () => {
    expect(ids(filterWorkspaceRows(rows, { query: 'npx', app: null }))).toEqual(['filesystem'])
    expect(ids(filterWorkspaceRows(rows, { query: 'linear.app', app: null }))).toEqual(['linear'])
  })

  it('matches the transport label', () => {
    expect(ids(filterWorkspaceRows(rows, { query: 'http', app: null }))).toEqual(['linear'])
  })

  it('requires every whitespace-separated term to match', () => {
    expect(ids(filterWorkspaceRows(rows, { query: 'uvx github', app: null }))).toEqual(['github'])
    expect(ids(filterWorkspaceRows(rows, { query: 'uvx linear', app: null }))).toEqual([])
  })

  it('keeps only rows enabled for the selected app', () => {
    expect(ids(filterWorkspaceRows(rows, { query: '', app: 'cursor' }))).toEqual(['github', 'filesystem'])
    expect(ids(filterWorkspaceRows(rows, { query: 'file', app: 'vscode' }))).toEqual([])
  })
})

describe('retainVisibleSelection', () => {
  it('drops ids that are hidden or no longer exist', () => {
    expect(retainVisibleSelection(['a', 'b', 'c'], ['a', 'c', 'd'])).toEqual(['a', 'c'])
  })

  it('returns the same array when nothing changes so state updates can bail out', () => {
    const selection = ['a', 'b']
    expect(retainVisibleSelection(selection, ['a', 'b', 'c'])).toBe(selection)
  })
})

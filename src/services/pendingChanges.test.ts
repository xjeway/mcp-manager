import { describe, expect, it } from 'vitest'
import { carryUnknownFields, mergeServerIntoConfig, shouldPromptForPendingChanges } from './pendingChanges'

describe('pendingChanges', () => {
  it('does not prompt before opening settings from the dashboard', () => {
    expect(
      shouldPromptForPendingChanges({
        currentView: 'dashboard',
        nextAction: { kind: 'view', view: 'settings' },
        workspaceDirty: false,
        editorDirty: false,
      }),
    ).toBe(false)
  })

  it('does not prompt when returning from settings to dashboard', () => {
    expect(
      shouldPromptForPendingChanges({
        currentView: 'settings',
        nextAction: { kind: 'view', view: 'dashboard' },
        workspaceDirty: false,
        editorDirty: false,
      }),
    ).toBe(false)
  })

  it('prompts before leaving the editor when the editor draft is dirty', () => {
    expect(
      shouldPromptForPendingChanges({
        currentView: 'editor',
        nextAction: { kind: 'view', view: 'dashboard' },
        workspaceDirty: false,
        editorDirty: true,
      }),
    ).toBe(true)
  })

  it('prompts before closing the window when any pending changes exist', () => {
    expect(
      shouldPromptForPendingChanges({
        currentView: 'dashboard',
        nextAction: { kind: 'close' },
        workspaceDirty: false,
        editorDirty: false,
      }),
    ).toBe(false)
  })

  it('prompts before closing the window when the editor draft is dirty', () => {
    expect(
      shouldPromptForPendingChanges({
        currentView: 'editor',
        nextAction: { kind: 'close' },
        workspaceDirty: false,
        editorDirty: true,
      }),
    ).toBe(true)
  })

  it('merges an edited server into the current config for editor submit flow', () => {
    expect(
      mergeServerIntoConfig(
        {
          version: 1,
          servers: [
            {
              id: 'server-1',
              name: 'Server 1',
              enabled: true,
              transport: { type: 'stdio' },
              command: { program: 'npx', args: ['old'], env: {} },
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
            },
          ],
        },
        'server-1',
        {
          id: 'server-1',
          name: 'Server 1',
          enabled: true,
          transport: { type: 'stdio' },
          command: { program: 'npx', args: ['new'], env: {} },
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
            kiro: false,
            qoder: false,
          },
        },
      ).servers[0].command?.args,
    ).toEqual(['new'])
  })

  it('keeps fields a newer version saved when the editor replaces a server', () => {
    const saved = {
      id: 'linear',
      name: 'linear',
      tags: ['work'],
      transport: { type: 'http', url: 'https://old', auth: 'oauth' },
      command: { program: 'npx', args: [], env: {}, cwd: '/tmp' },
      apps: { cursor: true, zed: true },
    }
    const edited = {
      id: 'linear',
      name: 'Linear',
      transport: { type: 'http', url: 'https://new' },
      command: { program: 'npx', args: ['-y'], env: {} },
      apps: { cursor: false },
    }

    expect(carryUnknownFields(saved, edited)).toEqual({
      id: 'linear',
      name: 'Linear',
      tags: ['work'],
      transport: { type: 'http', url: 'https://new', auth: 'oauth' },
      command: { program: 'npx', args: ['-y'], env: {}, cwd: '/tmp' },
      apps: { cursor: false, zed: true },
    })
    expect(
      mergeServerIntoConfig({ version: 1, servers: [saved] }, 'linear', edited).servers[0],
    ).toMatchObject({ name: 'Linear', tags: ['work'] })
  })

  it('does not bring back fields the editor models and cleared', () => {
    const saved = { id: 'a', description: 'old', transport: { type: 'http', url: 'u', headers: { A: '1' } } }
    const edited = { id: 'a', transport: { type: 'stdio' } }

    expect(carryUnknownFields(saved, edited)).toEqual(edited)
  })

  it('keeps clients the editor does not offer when it rebuilds apps and placements', () => {
    const saved = {
      id: 'a',
      apps: { cursor: true, zed: true },
      placements: [
        { app: 'cursor', scope: 'user', enabled: true },
        { app: 'zed', scope: 'user', enabled: true },
      ],
    }
    // As the JSON mode produces it: only known clients, and cursor turned off.
    const edited = { id: 'a', apps: { cursor: false, vscode: true }, placements: [] }

    expect(carryUnknownFields(saved, edited)).toEqual({
      id: 'a',
      apps: { cursor: false, vscode: true, zed: true },
      placements: [{ app: 'zed', scope: 'user', enabled: true }],
    })
  })
})

import { describe, expect, it } from 'vitest'
import { mergeServerIntoConfig, shouldPromptForPendingChanges } from './pendingChanges'

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
          },
        },
      ).servers[0].command?.args,
    ).toEqual(['new'])
  })
})

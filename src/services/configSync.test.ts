import { describe, expect, it } from 'vitest'
import { areConfigsEquivalent } from './configSync'

describe('configSync', () => {
  it('treats equivalent configs with different ordering as equal', () => {
    const left = {
      version: 1,
      servers: [
        {
          id: 'b',
          name: 'B',
          enabled: true,
          description: undefined,
          homepage: undefined,
          transport: { type: 'stdio' as const },
          command: { program: 'npx', args: ['b'], env: { BAR: '2', FOO: '1' } },
          apps: {
            vscode: false,
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
        {
          id: 'a',
          name: 'A',
          enabled: true,
          description: undefined,
          homepage: undefined,
          transport: { type: 'http' as const, url: 'https://example.com' },
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
    }

    const right = {
      version: 1,
      servers: [
        {
          id: 'a',
          name: 'A',
          enabled: true,
          description: undefined,
          homepage: undefined,
          transport: { type: 'http' as const, url: 'https://example.com' },
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
        {
          id: 'b',
          name: 'B',
          enabled: true,
          description: undefined,
          homepage: undefined,
          transport: { type: 'stdio' as const },
          command: { program: 'npx', args: ['b'], env: { FOO: '1', BAR: '2' } },
          apps: {
            vscode: false,
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
      ],
    }

    expect(areConfigsEquivalent(left, right)).toBe(true)
  })
})

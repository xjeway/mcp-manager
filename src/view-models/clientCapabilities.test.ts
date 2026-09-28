import { describe, expect, it } from 'vitest'
import { SUPPORTED_APPS } from '../types/config'
import {
  CLIENT_CAPABILITIES,
  getClientCapability,
  getProjectConfigurableApps,
} from './clientCapabilities'

describe('client capabilities', () => {
  it('defines exactly one capability record for every supported app', () => {
    expect(Object.keys(CLIENT_CAPABILITIES).sort()).toEqual([...SUPPORTED_APPS].sort())
  })

  it('lists the clients whose project config can be managed directly', () => {
    expect(getProjectConfigurableApps()).toEqual([
      'claudeCode',
      'cursor',
      'geminiCli',
      'iFlow',
      'kiro',
      'openCode',
      'qwenCode',
      'vscode',
    ])
  })

  it('models Claude Code as user, shared project, and private local project scopes', () => {
    expect(getClientCapability('claudeCode').scopes.map((scope) => scope.id)).toEqual([
      'localProject',
      'project',
      'user',
    ])
  })

  it('keeps GitHub Copilot out of direct project toggles because workspace MCP is owned by VS Code', () => {
    const capability = getClientCapability('githubCopilot')

    expect(capability.projectManagement).toBe('viaOtherClient')
    expect(capability.projectViaApp).toBe('vscode')
  })
})

import type { PlacementScope, SupportedApp } from '../types/config'

export type ClientScopeId = 'user' | 'project' | 'localProject' | 'agent'
export type ProjectManagementMode = 'direct' | 'viaOtherClient' | 'unsupported'
export type ConfigMergePolicy = 'mergeJsonField' | 'mergeTomlTable' | 'replaceJson' | 'managedExternally'

export interface ClientScopeCapability {
  id: ClientScopeId
  labelKey: string
  pathKind: 'userConfig' | 'workspaceFile' | 'claudeProjectEntry' | 'agentConfig'
  placementScope?: PlacementScope
  precedence: number
  sharedWithTeam: boolean
  writable: boolean
}

export interface ClientCapability {
  app: SupportedApp
  configFormat: 'json' | 'jsonc' | 'toml'
  mergePolicy: ConfigMergePolicy
  projectManagement: ProjectManagementMode
  projectViaApp?: SupportedApp
  scopes: ClientScopeCapability[]
}

const userScope = (precedence = 10): ClientScopeCapability => ({
  id: 'user',
  labelKey: 'placementScopeUser',
  pathKind: 'userConfig',
  placementScope: 'user',
  precedence,
  sharedWithTeam: false,
  writable: true,
})

const projectScope = (precedence = 20): ClientScopeCapability => ({
  id: 'project',
  labelKey: 'placementScopeWorkspace',
  pathKind: 'workspaceFile',
  placementScope: 'workspace',
  precedence,
  sharedWithTeam: true,
  writable: true,
})

const localProjectScope = (precedence = 30): ClientScopeCapability => ({
  id: 'localProject',
  labelKey: 'placementScopeLocalProject',
  pathKind: 'claudeProjectEntry',
  precedence,
  sharedWithTeam: false,
  writable: false,
})

function userOnly(
  app: SupportedApp,
  configFormat: ClientCapability['configFormat'],
  mergePolicy: ConfigMergePolicy,
): ClientCapability {
  return {
    app,
    configFormat,
    mergePolicy,
    projectManagement: 'unsupported',
    scopes: [userScope()],
  }
}

export const CLIENT_CAPABILITIES: Record<SupportedApp, ClientCapability> = {
  vscode: {
    app: 'vscode',
    configFormat: 'json',
    mergePolicy: 'mergeJsonField',
    projectManagement: 'direct',
    scopes: [projectScope(20), userScope(10)],
  },
  cursor: {
    app: 'cursor',
    configFormat: 'json',
    mergePolicy: 'mergeJsonField',
    projectManagement: 'direct',
    scopes: [projectScope(20), userScope(10)],
  },
  claudeCode: {
    app: 'claudeCode',
    configFormat: 'json',
    mergePolicy: 'mergeJsonField',
    projectManagement: 'direct',
    scopes: [localProjectScope(30), projectScope(20), userScope(10)],
  },
  claudeDesktop: userOnly('claudeDesktop', 'json', 'mergeJsonField'),
  codex: userOnly('codex', 'toml', 'mergeTomlTable'),
  openCode: {
    app: 'openCode',
    configFormat: 'jsonc',
    mergePolicy: 'mergeJsonField',
    projectManagement: 'direct',
    scopes: [projectScope(20), userScope(10)],
  },
  githubCopilot: {
    app: 'githubCopilot',
    configFormat: 'json',
    mergePolicy: 'mergeJsonField',
    projectManagement: 'viaOtherClient',
    projectViaApp: 'vscode',
    scopes: [userScope()],
  },
  geminiCli: {
    app: 'geminiCli',
    configFormat: 'json',
    mergePolicy: 'mergeJsonField',
    projectManagement: 'direct',
    scopes: [projectScope(20), userScope(10)],
  },
  antigravity: userOnly('antigravity', 'json', 'mergeJsonField'),
  iFlow: {
    app: 'iFlow',
    configFormat: 'json',
    mergePolicy: 'mergeJsonField',
    projectManagement: 'direct',
    scopes: [projectScope(20), userScope(10)],
  },
  qwenCode: {
    app: 'qwenCode',
    configFormat: 'json',
    mergePolicy: 'mergeJsonField',
    projectManagement: 'direct',
    scopes: [projectScope(20), userScope(10)],
  },
  cline: userOnly('cline', 'json', 'mergeJsonField'),
  windsurf: userOnly('windsurf', 'json', 'mergeJsonField'),
  kiro: {
    app: 'kiro',
    configFormat: 'json',
    mergePolicy: 'mergeJsonField',
    projectManagement: 'direct',
    scopes: [
      {
        id: 'agent',
        labelKey: 'placementScopeAgent',
        pathKind: 'agentConfig',
        precedence: 30,
        sharedWithTeam: true,
        writable: false,
      },
      projectScope(20),
      userScope(10),
    ],
  },
}

export function getClientCapability(app: SupportedApp): ClientCapability {
  return CLIENT_CAPABILITIES[app]
}

export function getProjectConfigurableApps(): SupportedApp[] {
  return Object.values(CLIENT_CAPABILITIES)
    .filter((capability) => capability.projectManagement === 'direct')
    .map((capability) => capability.app)
    .sort((left, right) => left.localeCompare(right))
}

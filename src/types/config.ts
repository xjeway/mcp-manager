export const SUPPORTED_APPS = [
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
  'qoder',
] as const

export type SupportedApp = (typeof SUPPORTED_APPS)[number]

export type TransportType = 'stdio' | 'http'
export type PlacementScope = 'user' | 'workspace'

export interface CommandSpec {
  program: string
  args: string[]
  env: Record<string, string>
}

export interface MCPServer {
  description?: string
  homepage?: string
  id: string
  name: string
  enabled: boolean
  transport: {
    type: TransportType
    url?: string
  }
  command?: CommandSpec
  apps: Record<SupportedApp, boolean>
  placements?: ServerPlacement[]
}

/** The project the app was started from, as reported by the backend. */
export interface WorkspaceContext {
  /** Empty when there is no current project. */
  root: string
  /** Project-level config file per app that supports one. */
  placementPaths: Partial<Record<SupportedApp, string>>
}

export const EMPTY_WORKSPACE: WorkspaceContext = { root: '', placementPaths: {} }

export interface ServerPlacement {
  app: SupportedApp
  scope: PlacementScope
  path?: string
  enabled: boolean
  managed?: boolean
}

export interface MCPConfig {
  version: number
  servers: MCPServer[]
}

export interface ParseWarning {
  message: string
}

export interface ParseError {
  message: string
}

export interface ParseResult {
  servers: MCPServer[]
  warnings: ParseWarning[]
  errors: ParseError[]
}

export interface ApplyRiskSummary {
  warnings: string[]
  blockingErrors: string[]
}

export interface AppSettings {
  language: 'zh-CN' | 'en-US'
  autoUpdatePolicy: 'prompt_confirm'
}

export interface LocalConfigSource {
  app: SupportedApp | 'yaml'
  path: string
  exists: boolean
  format?: 'yaml' | 'json'
  priority?: number
  content?: string
}

export interface ImportDetectedResult {
  config: MCPConfig
  sources: LocalConfigSource[]
  warnings: string[]
  errors: string[]
}

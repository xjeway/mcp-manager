import type { TFunction } from 'i18next'
import type { ReactNode } from 'react'
import { getVisibleClients } from '../components/clientMeta'
import type { MCPConfig, MCPServer, SupportedApp } from '../types/config'

export interface FeedbackItem {
  id: string
  kind: 'success' | 'warning' | 'error' | 'info'
  message: string
}

export interface WorkspaceStatViewModel {
  accent: string
  count: number
  icon: ReactNode
  id: SupportedApp
  label: string
}

export interface WorkspaceRowViewModel {
  copyValue: string
  enabledApps: SupportedApp[]
  id: string
  name: string
  transportLabel: string
}

export interface WorkspaceViewModel {
  rows: WorkspaceRowViewModel[]
  stats: WorkspaceStatViewModel[]
}

export type EditorMode = 'form' | 'json'

export interface EditorDraft {
  apps: MCPServer['apps']
  args: string[]
  description: string
  enabled: boolean
  envEntries: Array<{ key: string; value: string }>
  homepage: string
  id: string
  name: string
  program: string
  transportType: 'stdio' | 'http'
  url: string
}

function emptyApps(): MCPServer['apps'] {
  return {
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
  }
}

function commandSummary(server: MCPServer, t: TFunction): string {
  if (server.transport.type === 'http') {
    return server.transport.url ?? t('missingHttpUrl')
  }

  if (!server.command?.program) {
    return t('missingCommand')
  }

  const args = server.command.args.join(' ')
  return args ? `${server.command.program} ${args}` : server.command.program
}

export function mapConfigToWorkspaceView(
  config: MCPConfig,
  visibleApps: SupportedApp[],
  t: TFunction,
): WorkspaceViewModel {
  const visibleClients = getVisibleClients(visibleApps)
  return {
    stats: visibleClients.map((client) => ({
      ...client,
      count: config.servers.filter((server) => server.enabled && server.apps[client.id]).length,
    })),
    rows: config.servers.map((server) => ({
      id: server.id,
      name: server.name,
      transportLabel: server.transport.type === 'http' ? t('transportHttp') : t('transportStdio'),
      copyValue: commandSummary(server, t),
      enabledApps: visibleClients.filter((client) => server.apps[client.id]).map((client) => client.id),
    })),
  }
}

export function createEmptyEditorDraft(): EditorDraft {
  return {
    description: '',
    id: '',
    homepage: '',
    name: '',
    enabled: true,
    transportType: 'stdio',
    program: '',
    args: [],
    url: '',
    envEntries: [],
    apps: emptyApps(),
  }
}

export function serverToEditorDraft(server: MCPServer | null | undefined): EditorDraft {
  if (!server) {
    return createEmptyEditorDraft()
  }

  const envEntries = Object.entries(server.command?.env ?? {}).map(([key, value]) => ({ key, value }))

  return {
    description: server.description ?? '',
    id: server.id,
    homepage: server.homepage ?? '',
    name: server.name,
    enabled: server.enabled,
    transportType: server.transport.type,
    program: server.command?.program ?? '',
    args: server.command?.args ?? [],
    url: server.transport.url ?? '',
    envEntries,
    apps: { ...server.apps },
  }
}

function slugify(input: string): string {
  return input
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

export function editorDraftToServer(draft: EditorDraft): MCPServer {
  const id = draft.id.trim() || slugify(draft.name) || 'new-server'
  const name = draft.name.trim() || id
  const env = draft.envEntries.reduce<Record<string, string>>((acc, entry) => {
    if (entry.key.trim()) {
      acc[entry.key.trim()] = entry.value
    }
    return acc
  }, {})

  return {
    description: draft.description.trim() || undefined,
    homepage: draft.homepage.trim() || undefined,
    id,
    name,
    enabled: draft.enabled,
    transport:
      draft.transportType === 'http'
        ? { type: 'http', url: draft.url.trim() }
        : { type: 'stdio' },
    command:
      draft.transportType === 'stdio'
        ? {
            program: draft.program.trim(),
            args: draft.args,
            env,
          }
        : undefined,
    apps: { ...draft.apps },
  }
}

export function serverToJsonText(server: MCPServer | null | undefined): string {
  if (!server) {
    return JSON.stringify(
      {
        mcpServers: {
          example: {
            command: 'npx',
            args: ['-y', '@modelcontextprotocol/server-example'],
            env: {},
          },
        },
      },
      null,
      2,
    )
  }

  if (server.transport.type === 'http') {
    return JSON.stringify(
      {
        mcpServers: {
          [server.id]: {
            description: server.description,
            homepage: server.homepage,
            type: 'http',
            url: server.transport.url ?? '',
            name: server.name,
          },
        },
      },
      null,
      2,
    )
  }

  return JSON.stringify(
    {
      mcpServers: {
        [server.id]: {
          description: server.description,
          homepage: server.homepage,
          name: server.name,
          command: server.command?.program ?? '',
          args: server.command?.args ?? [],
          env: server.command?.env ?? {},
        },
      },
    },
    null,
    2,
  )
}

import type { TFunction } from 'i18next'
import type { ReactNode } from 'react'
import { CLIENTS, getVisibleClients } from '../components/clientMeta'
import type {
  ApplyWarning,
  MCPConfig,
  MCPServer,
  PlacementScope,
  ServerPlacement,
  SupportedApp,
  WorkspaceContext,
} from '../types/config'

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
  placements: WorkspacePlacementViewModel[]
  transportLabel: string
}

export interface WorkspaceViewModel {
  rows: WorkspaceRowViewModel[]
  stats: WorkspaceStatViewModel[]
}

export interface WorkspacePlacementViewModel {
  app: SupportedApp
  label: string
  path?: string
  scope: PlacementScope
  scopeLabel: string
}

export type EditorMode = 'form' | 'json'

export interface EditorDraft {
  apps: MCPServer['apps']
  args: string[]
  description: string
  enabled: boolean
  envEntries: Array<{ key: string; value: string }>
  headerEntries: Array<{ key: string; value: string }>
  homepage: string
  id: string
  name: string
  placements: ServerPlacement[]
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
    qoder: false,
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

export function getServerPlacements(server: MCPServer): ServerPlacement[] {
  return server.placements?.filter((placement) => placement.enabled) ?? []
}

type AppPlacementTarget = Pick<MCPServer, 'apps' | 'placements'>

/** Whether the server is installed for `app` at any scope. */
export function isServerAppEnabled(server: AppPlacementTarget, app: SupportedApp): boolean {
  return (
    server.apps[app] || server.placements?.some((placement) => placement.enabled && placement.app === app) || false
  )
}

/** Whether the server is installed for `app` at user level. */
export function isUserScopeEnabled(server: AppPlacementTarget, app: SupportedApp): boolean {
  return (
    server.apps[app] ||
    server.placements?.some((placement) => placement.enabled && placement.app === app && placement.scope === 'user') ||
    false
  )
}

/**
 * Turns `app` on or off for a server. The user-level flag and user-scope
 * placements always follow `enabled`. With `includeWorkspace`, turning the app
 * off also disables its project placements; turning it on never re-enables
 * them, so a project is only ever changed explicitly.
 */
export function setServerAppEnabled<T extends AppPlacementTarget>(
  server: T,
  app: SupportedApp,
  enabled: boolean,
  { includeWorkspace = false }: { includeWorkspace?: boolean } = {},
): T {
  return {
    ...server,
    apps: { ...server.apps, [app]: enabled },
    placements: server.placements?.map((placement) =>
      placement.app === app && (placement.scope === 'user' || (includeWorkspace && !enabled))
        ? { ...placement, enabled }
        : placement,
    ),
  }
}

export interface WorkspaceRowFilter {
  app: SupportedApp | null
  query: string
}

export function filterWorkspaceRows(rows: WorkspaceRowViewModel[], filter: WorkspaceRowFilter): WorkspaceRowViewModel[] {
  const terms = filter.query.trim().toLowerCase().split(/\s+/).filter(Boolean)
  return rows.filter((row) => {
    if (filter.app && !row.enabledApps.includes(filter.app)) {
      return false
    }
    const haystack = [row.name, row.copyValue, row.transportLabel].join(' ').toLowerCase()
    return terms.every((term) => haystack.includes(term))
  })
}

/** Keeps only selected ids that are still visible; returns `selectedIds` itself if none were dropped. */
export function retainVisibleSelection(selectedIds: string[], visibleIds: string[]): string[] {
  const next = selectedIds.filter((id) => visibleIds.includes(id))
  return next.length === selectedIds.length ? selectedIds : next
}

function placementScopeLabel(scope: PlacementScope): string {
  return scope === 'workspace' ? 'placementScopeWorkspace' : 'placementScopeUser'
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
      count: config.servers.filter((server) => server.enabled && isServerAppEnabled(server, client.id)).length,
    })),
    rows: config.servers.map((server) => ({
      id: server.id,
      name: server.name,
      transportLabel: server.transport.type === 'http' ? t('transportHttp') : t('transportStdio'),
      copyValue: commandSummary(server, t),
      enabledApps: visibleClients.filter((client) => isServerAppEnabled(server, client.id)).map((client) => client.id),
      placements: visibleClients
        .flatMap((client) =>
          getServerPlacements(server)
            .filter((placement) => placement.app === client.id)
            .map((placement) => ({
              app: client.id,
              label: client.label,
              path: placement.path,
              scope: placement.scope,
              scopeLabel: placementScopeLabel(placement.scope),
            })),
        ),
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
    placements: [],
    transportType: 'stdio',
    program: '',
    args: [],
    url: '',
    envEntries: [],
    headerEntries: [],
    apps: emptyApps(),
  }
}

export function serverToEditorDraft(server: MCPServer | null | undefined): EditorDraft {
  if (!server) {
    return createEmptyEditorDraft()
  }

  const envEntries = Object.entries(server.command?.env ?? {}).map(([key, value]) => ({ key, value }))
  const headerEntries = Object.entries(server.transport.headers ?? {}).map(([key, value]) => ({ key, value }))

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
    headerEntries,
    apps: { ...server.apps },
    placements: getServerPlacements(server),
  }
}

function slugify(input: string): string {
  return input
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

function entriesToRecord(entries: EditorDraft['envEntries']): Record<string, string> {
  return entries.reduce<Record<string, string>>((acc, entry) => {
    if (entry.key.trim()) {
      acc[entry.key.trim()] = entry.value
    }
    return acc
  }, {})
}

export function editorDraftToServer(draft: EditorDraft): MCPServer {
  const id = draft.id.trim() || slugify(draft.name) || 'new-server'
  const name = draft.name.trim() || id
  const env = entriesToRecord(draft.envEntries)
  const headers = entriesToRecord(draft.headerEntries)

  const placements = draft.placements.filter((placement) => placement.enabled)
  const apps = { ...draft.apps }

  return {
    description: draft.description.trim() || undefined,
    homepage: draft.homepage.trim() || undefined,
    id,
    name,
    enabled: draft.enabled,
    transport:
      draft.transportType === 'http'
        ? { type: 'http', url: draft.url.trim(), ...(Object.keys(headers).length > 0 ? { headers } : {}) }
        : { type: 'stdio' },
    command:
      draft.transportType === 'stdio'
        ? {
            program: draft.program.trim(),
            args: draft.args,
            env,
        }
        : undefined,
    apps,
    placements,
  }
}

export function workspacePlacementPath(app: SupportedApp, workspace: WorkspaceContext): string | null {
  return workspace.root.trim() ? workspace.placementPaths[app] ?? null : null
}

/** Enables or disables the project-level placement of `app` at `path`, adding it if missing. */
export function setWorkspacePlacement(
  placements: ServerPlacement[],
  app: SupportedApp,
  path: string,
  enabled: boolean,
): ServerPlacement[] {
  const isTarget = (placement: ServerPlacement) =>
    placement.app === app && placement.scope === 'workspace' && placement.path === path

  if (placements.some(isTarget)) {
    return placements.map((placement) => (isTarget(placement) ? { ...placement, enabled } : placement))
  }

  return [...placements, { app, scope: 'workspace', path, enabled, managed: true }]
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
            headers: server.transport.headers,
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

/**
 * User-facing text for the non-blocking problems an apply reports: settings a client
 * cannot store, and a change that was applied but could not be recorded for undo.
 */
export function applyWarningMessages(
  result: { warnings?: readonly ApplyWarning[]; historyError?: string },
  servers: readonly MCPServer[],
  t: TFunction,
): string[] {
  const messages = (result.warnings ?? []).map((warning) =>
    t('applyWarningHttpHeadersUnsupported', {
      client: CLIENTS.find((client) => client.id === warning.app)?.label ?? warning.app,
      server: servers.find((server) => server.id === warning.serverId)?.name ?? warning.serverId,
    }),
  )
  if (result.historyError) {
    messages.push(t('applyWarningUndoNotRecorded', { error: result.historyError }))
  }
  return messages
}

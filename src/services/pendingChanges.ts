import { SUPPORTED_APPS } from '../types/config'

export type GuardView = 'dashboard' | 'editor' | 'settings' | 'marketplace'

export type PendingChangesAction =
  | { kind: 'view'; view: GuardView }
  | { kind: 'close' }

export function mergeServerIntoConfig<
  TConfig extends {
    version: number
    servers: TServer[]
  },
  TServer extends {
    id: string
  },
>(config: TConfig, editingId: string | null | undefined, server: TServer): TConfig {
  const targetId = editingId ?? server.id
  const exists = config.servers.some((item) => item.id === targetId)

  return {
    ...config,
    servers: exists
      ? config.servers.map((item) => (item.id === targetId ? carryUnknownFields(item, server) : item))
      : [...config.servers, server],
  }
}

// What the editor models of a server. A newer MCP Manager or mcpmgr may have
// saved more; the editor rebuilds a server from these fields alone, so the
// rest is carried over from the saved server instead of being dropped.
const KNOWN_SERVER_KEYS = ['description', 'homepage', 'id', 'name', 'enabled', 'transport', 'command', 'apps', 'placements']
const KNOWN_TRANSPORT_KEYS = ['type', 'url', 'headers']
const KNOWN_COMMAND_KEYS = ['program', 'args', 'env']

type Fields = Record<string, unknown>

function isFields(value: unknown): value is Fields {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function withUnknown(previous: unknown, next: unknown, known: string[]): unknown {
  if (!isFields(previous) || !isFields(next)) {
    return next
  }
  const unknown = Object.fromEntries(Object.entries(previous).filter(([key]) => !known.includes(key)))
  return { ...unknown, ...next }
}

function isKnownApp(app: unknown): boolean {
  return (SUPPORTED_APPS as readonly unknown[]).includes(app)
}

// The editor only offers the clients this build knows (its JSON mode rebuilds
// `apps` and `placements` from them), so entries for any other client always
// come from the saved server, even if a stale draft still carries an older
// value; known clients follow the edit, removals included.
function withUnknownClients(previous: Fields, merged: Fields): Fields {
  const unknownApps = isFields(previous.apps)
    ? Object.fromEntries(Object.entries(previous.apps).filter(([app]) => !isKnownApp(app)))
    : {}
  const unknownPlacements = Array.isArray(previous.placements)
    ? previous.placements.filter((placement) => isFields(placement) && !isKnownApp(placement.app))
    : []
  return {
    ...(Object.keys(unknownApps).length > 0 || isFields(merged.apps)
      ? {
          apps: {
            ...unknownApps,
            ...(isFields(merged.apps)
              ? Object.fromEntries(Object.entries(merged.apps).filter(([app]) => isKnownApp(app)))
              : {}),
          },
        }
      : {}),
    ...(unknownPlacements.length > 0
      ? {
          placements: [
            ...(Array.isArray(merged.placements)
              ? merged.placements.filter((placement) => !isFields(placement) || isKnownApp(placement.app))
              : []),
            ...unknownPlacements,
          ],
        }
      : {}),
  }
}

export function carryUnknownFields<TServer>(previous: TServer, next: TServer): TServer {
  const merged = withUnknown(previous, next, KNOWN_SERVER_KEYS) as Fields
  if (!isFields(previous) || !isFields(merged)) {
    return next
  }
  return {
    ...merged,
    ...(isFields(merged.transport)
      ? { transport: withUnknown(previous.transport, merged.transport, KNOWN_TRANSPORT_KEYS) }
      : {}),
    ...(isFields(merged.command) ? { command: withUnknown(previous.command, merged.command, KNOWN_COMMAND_KEYS) } : {}),
    ...withUnknownClients(previous, merged),
  } as TServer
}

export function shouldPromptForPendingChanges({
  currentView,
  nextAction,
  workspaceDirty: _workspaceDirty,
  editorDirty,
}: {
  currentView: GuardView
  nextAction: PendingChangesAction
  workspaceDirty: boolean
  editorDirty: boolean
}): boolean {
  if (!editorDirty || currentView !== 'editor') {
    return false
  }

  if (nextAction.kind === 'close') {
    return true
  }

  return nextAction.view !== 'editor'
}

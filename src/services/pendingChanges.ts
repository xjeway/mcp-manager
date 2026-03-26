export type GuardView = 'dashboard' | 'editor' | 'settings'

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
      ? config.servers.map((item) => (item.id === targetId ? server : item))
      : [...config.servers, server],
  }
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

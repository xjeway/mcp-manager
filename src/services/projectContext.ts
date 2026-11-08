import type { MCPConfig, ServerPlacement, WorkspaceContext } from '../types/config'

const SELECTION_KEY = 'ui-project-selection'
const RECENT_KEY = 'ui-recent-projects'

/** Keep native Windows extended paths comparable to saved ordinary Windows paths. */
function normalize(path: string): string {
  const value = path.replace(/\\/g, '/').replace(/^\/\/\?\/UNC\//i, '//').replace(/^\/\/\?\//, '').replace(/\/+$/, '')
  return /^[a-z]:\//i.test(value) || value.startsWith('//') ? value.toLowerCase() : value
}

export function placementInProject(placement: ServerPlacement, workspace: WorkspaceContext): boolean {
  if (placement.scope === 'user') return true
  if (!workspace.root) return false
  if (!placement.path) return true
  const path = normalize(placement.path)
  if (workspace.contextPaths?.some((known) => normalize(known) === path)) return true
  const root = normalize(workspace.root)
  const parent = root.slice(0, root.lastIndexOf('/'))
  return path.startsWith(`${root}/`)
    || (placement.app === 'agents' && placement.managed === false && path === `${parent}/.agents/mcp.json`)
}

/** Present only this project's placements while retaining the shared server definitions. */
export function configForProject(config: MCPConfig, workspace: WorkspaceContext): MCPConfig {
  return { ...config, servers: config.servers.flatMap((server) => {
    const placements = (server.placements ?? []).filter((p) => placementInProject(p, workspace))
    const foreign = placements.length !== (server.placements ?? []).length
    if (foreign && placements.length === 0 && !Object.values(server.apps).some(Boolean)) return []
    return [{ ...server, placements }]
  }) }
}

/** Local deletion/disabling must not erase another project's installation metadata. */
export function mergeProjectChange(previous: MCPConfig, next: MCPConfig, workspace: WorkspaceContext): MCPConfig {
  const servers = [...next.servers]
  for (const old of previous.servers) {
    const foreign = (old.placements ?? []).filter((p) => !placementInProject(p, workspace))
    if (!foreign.length) continue
    const index = servers.findIndex((s) => s.id === old.id)
    if (index === -1) {
      servers.push({ ...old, apps: Object.fromEntries(Object.keys(old.apps).map((app) => [app, false])) as typeof old.apps, placements: foreign })
      continue
    }
    let server = servers[index]
    if (!old.enabled && server.enabled) throw new Error('This server is disabled across projects. Its shared enabled state cannot be changed from one project.')
    if (old.enabled && !server.enabled) {
      server = { ...server, enabled: old.enabled, apps: Object.fromEntries(Object.keys(server.apps).map((app) => [app, false])) as typeof server.apps, placements: server.placements?.map((p) => ({ ...p, enabled: false })) }
    }
    servers[index] = { ...server, placements: [...(server.placements ?? []).filter((p) => placementInProject(p, workspace)), ...foreign] }
  }
  return { ...next, servers }
}

export function readProjectSelection(): string | undefined {
  try { return typeof window === 'undefined' ? undefined : window.localStorage.getItem(SELECTION_KEY) ?? undefined }
  catch { return undefined }
}

export function readRecentProjects(): string[] {
  try {
    const value: unknown = JSON.parse(window.localStorage.getItem(RECENT_KEY) ?? '[]')
    return Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string' && item.length > 0).slice(0, 5) : []
  } catch { return [] }
}

export function saveProjectSelection(root: string): void {
  window.localStorage.setItem(SELECTION_KEY, root)
  if (root) window.localStorage.setItem(RECENT_KEY, JSON.stringify([root, ...readRecentProjects().filter((path) => path !== root)].slice(0, 5)))
}

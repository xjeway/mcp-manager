import type { ImportDetectedResult, MCPConfig, MCPServer, ServerPlacement, SupportedApp } from '../types/config'
import { isServerAppEnabled, setServerAppEnabled } from '../view-models/workspace'

interface SaveAndSyncConfigOptions {
  applyConfig: (config: MCPConfig, previousConfig: MCPConfig) => Promise<{ backups: string[] }>
  nextConfig: MCPConfig
  previousConfig: MCPConfig
  saveConfig: (config: MCPConfig) => Promise<void>
}

interface ImportConfigOnLaunchOptions {
  autoImportOnLaunch: boolean
  importDetectedConfigs: () => Promise<ImportDetectedResult>
}

export async function saveAndSyncConfig({
  applyConfig,
  nextConfig,
  previousConfig,
  saveConfig,
}: SaveAndSyncConfigOptions): Promise<{ backups: string[] }> {
  await saveConfig(nextConfig)

  try {
    return await applyConfig(nextConfig, previousConfig)
  } catch (error) {
    await saveConfig(previousConfig)
    throw error
  }
}

export async function persistImportedConfig(
  nextConfig: MCPConfig,
  saveConfig: (config: MCPConfig) => Promise<void>,
): Promise<void> {
  await saveConfig(nextConfig)
}

export async function importConfigOnLaunch({
  autoImportOnLaunch,
  importDetectedConfigs,
}: ImportConfigOnLaunchOptions): Promise<ImportDetectedResult | null> {
  if (!autoImportOnLaunch) {
    return null
  }

  return await importDetectedConfigs()
}

export function deleteServerFromConfig(config: MCPConfig, serverId: string): MCPConfig {
  return {
    ...config,
    servers: config.servers.filter((server) => server.id !== serverId),
  }
}

export function toggleServerAppInConfig(config: MCPConfig, serverId: string, app: SupportedApp): MCPConfig {
  return {
    ...config,
    servers: config.servers.map((server) =>
      server.id === serverId
        ? // The pill reflects every scope, so turning it off removes the server from the
          // project too, while turning it on only installs it at user level.
          setServerAppEnabled(server, app, !isServerAppEnabled(server, app), { includeWorkspace: true })
        : server,
    ),
  }
}

/** Turns `app` on or off for every server in `serverIds`, across all scopes. */
export function setServersAppInConfig(
  config: MCPConfig,
  serverIds: readonly string[],
  app: SupportedApp,
  enabled: boolean,
): MCPConfig {
  const targets = new Set(serverIds)
  return {
    ...config,
    servers: config.servers.map((server) =>
      targets.has(server.id) ? setServerAppEnabled(server, app, enabled, { includeWorkspace: true }) : server,
    ),
  }
}

/**
 * Adds `servers` to the config. A server whose id already exists takes the
 * incoming definition but keeps the clients and placements it already had, so
 * re-importing never uninstalls anything.
 */
export function importServersIntoConfig(
  config: MCPConfig,
  servers: readonly MCPServer[],
): { config: MCPConfig; added: string[]; updated: string[] } {
  const added: string[] = []
  const updated: string[] = []
  const nextServers = [...config.servers]

  for (const incoming of servers) {
    const index = nextServers.findIndex((server) => server.id === incoming.id)
    if (index === -1) {
      nextServers.push(incoming)
      added.push(incoming.id)
      continue
    }

    const current = nextServers[index]
    const apps = { ...current.apps }
    for (const [app, enabled] of Object.entries(incoming.apps) as Array<[SupportedApp, boolean]>) {
      apps[app] = apps[app] || enabled
    }
    const samePlacement = (left: ServerPlacement, right: ServerPlacement) =>
      left.app === right.app && left.scope === right.scope && left.path === right.path
    const incomingPlacements = incoming.placements ?? []
    const placements = [
      ...(current.placements ?? []).map((existing) =>
        incomingPlacements.some((placement) => placement.enabled && samePlacement(existing, placement))
          ? { ...existing, enabled: true }
          : existing,
      ),
      ...incomingPlacements.filter(
        (placement) => !current.placements?.some((existing) => samePlacement(existing, placement)),
      ),
    ]
    nextServers[index] = { ...incoming, apps, placements }
    updated.push(incoming.id)
  }

  return { config: { ...config, servers: nextServers }, added, updated }
}

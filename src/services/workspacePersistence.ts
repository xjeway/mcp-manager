import type { MCPConfig, SupportedApp } from '../types/config'

interface SaveAndSyncConfigOptions {
  applyConfig: (config: MCPConfig) => Promise<{ backups: string[] }>
  nextConfig: MCPConfig
  previousConfig: MCPConfig
  saveConfig: (config: MCPConfig) => Promise<void>
}

export async function saveAndSyncConfig({
  applyConfig,
  nextConfig,
  previousConfig,
  saveConfig,
}: SaveAndSyncConfigOptions): Promise<{ backups: string[] }> {
  await saveConfig(nextConfig)

  try {
    return await applyConfig(nextConfig)
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
        ? { ...server, apps: { ...server.apps, [app]: !server.apps[app] } }
        : server,
    ),
  }
}

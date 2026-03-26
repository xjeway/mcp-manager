import { SUPPORTED_APPS, type MCPConfig, type MCPServer } from '../types/config'

function normalizeServer(server: MCPServer) {
  return {
    id: server.id,
    name: server.name,
    enabled: server.enabled,
    description: server.description ?? null,
    homepage: server.homepage ?? null,
    transport: {
      type: server.transport.type,
      url: server.transport.url ?? null,
    },
    command: server.command
      ? {
          program: server.command.program,
          args: [...server.command.args],
          env: Object.fromEntries(Object.entries(server.command.env).sort(([left], [right]) => left.localeCompare(right))),
        }
      : null,
    apps: Object.fromEntries(SUPPORTED_APPS.map((app) => [app, Boolean(server.apps[app])])),
  }
}

function normalizeConfig(config: MCPConfig) {
  return {
    version: config.version,
    servers: [...config.servers]
      .map(normalizeServer)
      .sort((left, right) => left.id.localeCompare(right.id)),
  }
}

export function areConfigsEquivalent(left: MCPConfig, right: MCPConfig): boolean {
  return JSON.stringify(normalizeConfig(left)) === JSON.stringify(normalizeConfig(right))
}

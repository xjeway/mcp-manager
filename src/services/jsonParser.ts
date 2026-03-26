import type { MCPConfig, MCPServer, ParseResult } from '../types/config'
import { SUPPORTED_APPS } from '../types/config'

function defaultApps(): MCPServer['apps'] {
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

function normalizeServer(id: string, input: Record<string, unknown>): MCPServer {
  const apps = defaultApps()
  const command = typeof input.command === 'string' ? input.command : undefined
  const args = Array.isArray(input.args) ? input.args.map((x) => String(x)) : []
  const envInput = typeof input.env === 'object' && input.env !== null ? (input.env as Record<string, unknown>) : {}
  const env: Record<string, string> = {}
  for (const [k, v] of Object.entries(envInput)) {
    env[k] = String(v)
  }

  const type = typeof input.type === 'string' ? input.type : undefined
  const url =
    typeof input.url === 'string'
      ? input.url
      : typeof input.httpUrl === 'string'
        ? input.httpUrl
        : typeof input.serverUrl === 'string'
          ? input.serverUrl
          : undefined
  const disabled = input.disabled === true
  const transport =
    type === 'http' || type === 'sse' || type === 'streamable-http' || url
      ? { type: 'http' as const, url }
      : { type: 'stdio' as const }

  const server: MCPServer = {
    description: typeof input.description === 'string' && input.description.trim() ? input.description : undefined,
    homepage: typeof input.homepage === 'string' && input.homepage.trim() ? input.homepage : undefined,
    id,
    name: typeof input.name === 'string' && input.name.trim() ? input.name : id,
    enabled: disabled ? false : input.enabled === undefined ? true : Boolean(input.enabled),
    transport,
    apps,
  }

  if (command) {
    server.command = { program: command, args, env }
  }

  if (Array.isArray(input.apps)) {
    for (const app of input.apps) {
      if (SUPPORTED_APPS.includes(app as (typeof SUPPORTED_APPS)[number])) {
        apps[app as keyof typeof apps] = true
      }
    }
  }

  return server
}

export function parseMcpJson(jsonText: string): ParseResult {
  const warnings: { message: string }[] = []
  const errors: { message: string }[] = []
  const servers: MCPServer[] = []

  let parsed: unknown
  try {
    parsed = JSON.parse(jsonText)
  } catch (error) {
    return {
      servers: [],
      warnings: [],
      errors: [{ message: `JSON 解析失败: ${String(error)}` }],
    }
  }

  if (typeof parsed !== 'object' || parsed === null) {
    return {
      servers: [],
      warnings: [],
      errors: [{ message: 'JSON 顶层必须是对象' }],
    }
  }

  const obj = parsed as Record<string, unknown>
  const container =
    obj.mcpServers && typeof obj.mcpServers === 'object' && obj.mcpServers !== null
      ? (obj.mcpServers as Record<string, unknown>)
      : obj.servers && typeof obj.servers === 'object' && obj.servers !== null
        ? (obj.servers as Record<string, unknown>)
        : null

  if (container) {
    for (const [id, value] of Object.entries(container)) {
      if (typeof value !== 'object' || value === null) {
        warnings.push({ message: `server ${id} 不是对象，已忽略` })
        continue
      }
      const server = normalizeServer(id, value as Record<string, unknown>)
      if (!server.command?.program && !server.transport.url) {
        errors.push({ message: `server ${id} 缺少 command/url` })
        continue
      }
      servers.push(server)
    }
  } else {
    const id = typeof obj.id === 'string' && obj.id.trim() ? obj.id : typeof obj.name === 'string' ? obj.name : 'imported-server'
    const server = normalizeServer(id, obj)
    if (!server.command?.program && !server.transport.url) {
      errors.push({ message: `server ${id} 缺少 command/url` })
    } else {
      servers.push(server)
    }
  }

  return { servers, warnings, errors }
}

export function buildConfig(servers: MCPServer[]): MCPConfig {
  return {
    version: 1,
    servers,
  }
}

import type { ApplyRiskSummary, MCPServer } from '../types/config'

export function evaluateApplyRisks(servers: MCPServer[]): ApplyRiskSummary {
  const warnings: string[] = []
  const blockingErrors: string[] = []

  if (servers.length === 0) {
    warnings.push('当前没有可应用的 MCP server')
  }

  for (const server of servers) {
    if (!server.id.trim()) {
      blockingErrors.push('存在空的 server id')
    }

    if (server.transport.type === 'stdio' && !server.command?.program) {
      blockingErrors.push(`server ${server.id} 缺少 command.program`)
    }

    if (server.transport.type === 'http') {
      const url = server.transport.url ?? ''
      if (!url.startsWith('http://') && !url.startsWith('https://')) {
        blockingErrors.push(`server ${server.id} 的 URL 非法`)
      }
    }

    const enabledApps = Object.values(server.apps).filter(Boolean).length
    if (enabledApps === 0) {
      warnings.push(`server ${server.id} 未启用任何 App`)
    }
  }

  return { warnings, blockingErrors }
}

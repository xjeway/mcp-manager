import type { ApplyRiskSummary, MCPServer } from '../types/config'

// Mirrors TransportSpec::sends_headers_insecurely / duplicate_header in
// crates/mcp-manager-core/src/core/mod.rs.

/** Headers would cross the network in cleartext: plain http to a host other than this machine. */
export function sendsHeadersInsecurely(transport: MCPServer['transport']): boolean {
  if (transport.type === 'stdio' || Object.keys(transport.headers ?? {}).length === 0) {
    return false
  }
  const rest = transport.url?.startsWith('http://') ? transport.url.slice('http://'.length) : null
  if (rest === null) {
    return false
  }
  const authority = rest.split(/[/?#]/)[0]
  const hostPort = authority.split('@').pop() ?? ''
  const host = (hostPort.startsWith('[') ? `${hostPort.split(']')[0]}]` : hostPort.split(':')[0]).toLowerCase()
  // Match real addresses, not prefixes: `127.evil.com` is a remote host.
  const loopback = host === 'localhost' || host.endsWith('.localhost') || isLoopbackIpv4(host) || host === '[::1]'
  return !loopback
}

function isLoopbackIpv4(host: string): boolean {
  const octets = host.split('.')
  return (
    octets.length === 4 &&
    octets[0] === '127' &&
    octets.every((octet) => /^\d{1,3}$/.test(octet) && Number(octet) <= 255)
  )
}

/** A header name (lowercased) that appears more than once ignoring case; header names are case-insensitive. */
export function duplicateHeader(headers: Record<string, string> | undefined): string | null {
  const seen = new Set<string>()
  for (const name of Object.keys(headers ?? {}).sort()) {
    const lower = name.toLowerCase()
    if (seen.has(lower)) {
      return lower
    }
    seen.add(lower)
  }
  return null
}

export function duplicateHeaderMessage(serverId: string, header: string): string {
  return `server ${serverId} 的请求头 ${header} 重复（名称不区分大小写）`
}

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

    if (sendsHeadersInsecurely(server.transport)) {
      blockingErrors.push(`server ${server.id} 的请求头会以明文发送，请改用 https:// 地址`)
    }
    const duplicate = duplicateHeader(server.transport.headers)
    if (duplicate) {
      blockingErrors.push(duplicateHeaderMessage(server.id, duplicate))
    }

    const enabledApps = Object.values(server.apps).filter(Boolean).length
    if (enabledApps === 0) {
      warnings.push(`server ${server.id} 未启用任何 App`)
    }
  }

  return { warnings, blockingErrors }
}

import { describe, expect, it } from 'vitest'
import type { MCPServer } from '../types/config'
import { parseMcpJson } from './jsonParser'

describe('parseMcpJson', () => {
  it('parses single server shape', () => {
    const result = parseMcpJson(
      JSON.stringify({
        name: 'github',
        command: 'uvx',
        args: ['mcp-server-github'],
      }),
    )

    expect(result.errors).toHaveLength(0)
    expect(result.servers).toHaveLength(1)
    expect(result.servers[0].id).toBe('github')
    expect(result.servers[0].transport.type).toBe('stdio')
  })

  it('parses mcpServers container shape', () => {
    const result = parseMcpJson(
      JSON.stringify({
        mcpServers: {
          github: {
            command: 'uvx',
            args: ['mcp-server-github'],
          },
        },
      }),
    )

    expect(result.errors).toHaveLength(0)
    expect(result.servers).toHaveLength(1)
    expect(result.servers[0].id).toBe('github')
  })

  it('parses vscode servers container shape', () => {
    const result = parseMcpJson(
      JSON.stringify({
        servers: {
          github: {
            command: 'uvx',
            args: ['mcp-server-github'],
          },
        },
      }),
    )

    expect(result.errors).toHaveLength(0)
    expect(result.servers).toHaveLength(1)
    expect(result.servers[0].id).toBe('github')
  })

  it('returns error for invalid json', () => {
    const result = parseMcpJson('{')
    expect(result.errors.length).toBeGreaterThan(0)
  })

  it('defaults imported app flags to false', () => {
    const result = parseMcpJson(
      JSON.stringify({
        mcpServers: {
          filesystem: {
            command: 'npx',
            args: ['-y', '@modelcontextprotocol/server-filesystem'],
          },
        },
      }),
    )

    expect(result.errors).toHaveLength(0)
    expect(result.servers[0].apps).toEqual({
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
    } satisfies MCPServer['apps'])
  })

  it('parses http aliases and disabled flags', () => {
    const result = parseMcpJson(
      JSON.stringify({
        mcpServers: {
          linear: {
            serverUrl: 'https://mcp.linear.app/sse',
            type: 'sse',
            disabled: true,
          },
        },
      }),
    )

    expect(result.errors).toHaveLength(0)
    expect(result.servers[0].transport.type).toBe('http')
    expect(result.servers[0].transport.url).toBe('https://mcp.linear.app/sse')
    expect(result.servers[0].enabled).toBe(false)
  })

  it('parses remote headers, including Codex http_headers', () => {
    const result = parseMcpJson(
      JSON.stringify({
        mcpServers: {
          linear: { url: 'https://mcp.linear.app/mcp', headers: { Authorization: 'Bearer t', 'X-Retries': 3 } },
          figma: { url: 'https://mcp.figma.com/mcp', http_headers: { 'X-Region': 'us' } },
          plain: { url: 'https://example.com/mcp' },
        },
      }),
    )

    expect(result.errors).toHaveLength(0)
    expect(result.servers[0].transport.headers).toEqual({ Authorization: 'Bearer t', 'X-Retries': '3' })
    expect(result.servers[1].transport.headers).toEqual({ 'X-Region': 'us' })
    expect(result.servers[2].transport).not.toHaveProperty('headers')
  })

  it('ignores headers on stdio servers', () => {
    const result = parseMcpJson(JSON.stringify({ command: 'npx', headers: { Authorization: 'x' } }))
    expect(result.servers[0].transport).not.toHaveProperty('headers')
  })

  it('rejects a server whose header names differ only by case', () => {
    const result = parseMcpJson(
      JSON.stringify({
        mcpServers: {
          dupe: {
            url: 'https://mcp.example.com/mcp',
            headers: { Authorization: 'Bearer a', authorization: 'Bearer b' },
          },
          fine: { url: 'https://ok.example.com/mcp', headers: { Authorization: 'Bearer a' } },
        },
      }),
    )
    expect(result.servers.map((server) => server.id)).toEqual(['fine'])
    expect(result.errors).toEqual([{ message: 'server dupe 的请求头 authorization 重复（名称不区分大小写）' }])
  })
})

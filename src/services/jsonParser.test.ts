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
      codex: false,
    } satisfies MCPServer['apps'])
  })
})

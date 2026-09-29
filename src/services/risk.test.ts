import { describe, expect, it } from 'vitest'
import { evaluateApplyRisks } from './risk'

describe('evaluateApplyRisks', () => {
  it('returns blocking error for invalid http url', () => {
    const summary = evaluateApplyRisks([
      {
        id: 'bad-url',
        name: 'Bad URL',
        enabled: true,
        transport: { type: 'http', url: 'ftp://invalid' },
        apps: {
          vscode: true,
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
        },
      },
    ])

    expect(summary.blockingErrors.length).toBeGreaterThan(0)
  })

  const remote = (id: string, url: string, headers: Record<string, string>) => ({
    id,
    name: id,
    enabled: true,
    transport: { type: 'http' as const, url, headers },
    apps: { vscode: true } as never,
  })

  it('blocks request headers sent over plain http to a remote host', () => {
    const summary = evaluateApplyRisks([
      remote('leaky', 'http://mcp.example.com/mcp', { Authorization: 'Bearer t' }),
      remote('secure', 'https://mcp.example.com/mcp', { Authorization: 'Bearer t' }),
      remote('local', 'http://localhost:3000/mcp', { Authorization: 'Bearer t' }),
      remote('loopback', 'http://127.0.0.1:3000/mcp', { 'X-Key': 'k' }),
      remote('ipv6', 'http://[::1]:3000/mcp', { 'X-Key': 'k' }),
      remote('plain', 'http://mcp.example.com/mcp', {}),
      // Hostnames that merely look like loopback are remote.
      remote('lookalike', 'http://127.evil.com/mcp', { Authorization: 'Bearer t' }),
      remote('bad-octet', 'http://127.0.0.256/mcp', { Authorization: 'Bearer t' }),
      remote('loopback-range', 'http://127.1.2.3/mcp', { Authorization: 'Bearer t' }),
    ])
    expect(summary.blockingErrors).toEqual([
      'server leaky 的请求头会以明文发送，请改用 https:// 地址',
      'server lookalike 的请求头会以明文发送，请改用 https:// 地址',
      'server bad-octet 的请求头会以明文发送，请改用 https:// 地址',
    ])
  })

  it('blocks header names that differ only by case', () => {
    const summary = evaluateApplyRisks([
      remote('dupe', 'https://mcp.example.com/mcp', { Authorization: 'Bearer a', authorization: 'Bearer b' }),
    ])
    expect(summary.blockingErrors).toEqual(['server dupe 的请求头 authorization 重复（名称不区分大小写）'])
  })
})

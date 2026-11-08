import { describe, expect, it } from 'vitest'
import { evaluateApplyRisks, sharedProjectSecretWarnings } from './risk'

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
          kiloCode: false,
          factoryDroid: false,
          kimiCli: false,
          agents: false,
          codeBuddy: false,
          auggie: false,
          zed: false,
          mistralVibe: false,
          hermes: false,
          goose: false,
          amp: false,
          grok: false,
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

describe('shared project secrets', () => {
  const workspace = { root: '/project', placementPaths: { claudeCode: '/project/.mcp.json' } }
  const server = (): import('../types/config').MCPServer => ({
    id: 'github', name: 'GitHub', enabled: true,
    transport: { type: 'http', url: 'https://example.com', headers: { Authorization: 'Bearer very-private' } },
    apps: { claudeCode: true } as never,
    placements: [{ app: 'claudeCode', scope: 'workspace', enabled: true }],
  })
  const t = ((key: string, args?: Record<string, string>) => `${key}: ${JSON.stringify(args)}`) as import('i18next').TFunction

  it('names the target and key, never the value, without blocking apply', () => {
    const item = server()
    const messages = sharedProjectSecretWarnings([item], workspace, t)
    expect(messages).toHaveLength(1)
    expect(messages[0]).toContain('GitHub')
    expect(messages[0]).toContain('Claude Code')
    expect(messages[0]).toContain('/project/.mcp.json')
    expect(messages[0]).toContain('Authorization')
    expect(messages[0]).toContain('secretAdviceInterpolation')
    expect(messages[0]).not.toContain('very-private')
    expect(evaluateApplyRisks([item]).blockingErrors).toEqual([])
  })

  it.each(['', '   ', '${TOKEN}', '${env:TOKEN}', '{env:TOKEN}', 'Bearer ${TOKEN}'])('ignores placeholders %s', (value) => {
    const item = server()
    item.transport.headers = { Authorization: value }
    expect(sharedProjectSecretWarnings([item], workspace, t)).toEqual([])
  })

  it('warns for sensitive env keys, while ignoring ordinary settings', () => {
    const item = server()
    item.transport = { type: 'stdio' }
    item.command = { program: 'node', args: [], env: { GITHUB_TOKEN: 'private', LOG_LEVEL: 'debug', API_KEY: '${VAR}' } }
    expect(sharedProjectSecretWarnings([item], workspace, t)).toHaveLength(1)
    expect(sharedProjectSecretWarnings([item], workspace, t)[0]).toContain('GITHUB_TOKEN')
  })

  it('ignores disabled servers, disabled placements, user writes and private Devin files', () => {
    const disabled = server(); disabled.enabled = false
    const off = server(); off.placements![0].enabled = false
    const user = server(); user.placements![0].scope = 'user'
    const implicitUser = server(); implicitUser.placements = []
    const privateDevin = server(); privateDevin.placements = [{ app: 'windsurf', scope: 'workspace', enabled: true, path: '/project/.devin/mcp_config.local.json' }]
    expect(sharedProjectSecretWarnings([disabled, off, user, implicitUser, privateDevin], workspace, t)).toEqual([])
  })

  it('warns for explicit paths, including shared Devin config', () => {
    const item = server()
    item.placements = [{ app: 'windsurf', scope: 'workspace', enabled: true, path: '/project/.devin/mcp_config.json' }]
    expect(sharedProjectSecretWarnings([item], workspace, t)[0]).toContain('secretAdviceDevin')
    expect(sharedProjectSecretWarnings([item], workspace, t)[0]).toContain('/project/.devin/mcp_config.json')
  })
})


describe('Auggie shared and private project configuration', () => {
  const t = ((key: string) => key) as import('i18next').TFunction
  const server: import('../types/config').MCPServer = {
    id: 'api', name: 'API', enabled: true,
    apps: { auggie: false } as never,
    transport: { type: 'http', url: 'https://example.com', headers: { Authorization: 'Bearer private' } },
    placements: [{ app: 'auggie', scope: 'workspace', path: '/project/.augment/settings.local.json', enabled: true }],
  }
  it('warns for the shared file and exempts the local file', () => {
    const workspace = { root: '/project', placementPaths: { auggie: '/project/.augment/settings.json' } }
    expect(sharedProjectSecretWarnings([server], workspace, t)).toEqual([])
    const shared = { ...server, placements: [{ ...server.placements![0], path: undefined }] }
    expect(sharedProjectSecretWarnings([shared], workspace, t)).toEqual(['sharedProjectSecretWarning'])
    const windowsPrivate = { ...server, placements: [{ ...server.placements![0], path: 'C:\\project\\.augment\\settings.local.json' }] }
    expect(sharedProjectSecretWarnings([windowsPrivate], workspace, t)).toEqual([])
  })
})

describe('shared .agents configuration', () => {
  const t = ((key: string) => key) as import('i18next').TFunction
  const workspace = { root: '/project', placementPaths: { agents: '/project/.agents/mcp.json' } }
  const make = (value: string): import('../types/config').MCPServer => ({
    id: 'api', name: 'API', enabled: true, apps: { agents: false } as never,
    transport: { type: 'http', url: 'https://example.com', headers: { Authorization: value } },
    placements: [{ app: 'agents', scope: 'workspace', enabled: true, managed: true }],
  })
  it.each(['$TOKEN', 'Bearer $TOKEN'])('preserves supported remote environment references %s', (value) => {
    expect(sharedProjectSecretWarnings([make(value)], workspace, t)).toEqual([])
  })
  it.each(['${TOKEN}', 'Bearer ${TOKEN}', 'session=literal-secret; region=$REGION'])('warns for unsupported or mixed references %s', (value) => {
    expect(sharedProjectSecretWarnings([make(value)], workspace, t)).toEqual(['sharedProjectSecretWarning'])
  })
  it('warns for literals and exempts read-only parent sources', () => {
    const server = make('literal-secret')
    expect(sharedProjectSecretWarnings([server], workspace, t)).toEqual(['sharedProjectSecretWarning'])
    server.placements![0].managed = false
    expect(sharedProjectSecretWarnings([server], workspace, t)).toEqual([])
  })
  it('only treats whole stdio env values as dollar references', () => {
    const server = make('')
    server.transport = { type: 'stdio' }
    server.command = { program: 'node', args: [], env: { API_KEY: '$api_key' } }
    expect(sharedProjectSecretWarnings([server], workspace, t)).toEqual([])
    server.command.env.API_KEY = 'literal $api_key'
    expect(sharedProjectSecretWarnings([server], workspace, t)).toHaveLength(1)
  })
})

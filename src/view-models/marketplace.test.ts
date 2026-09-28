import { describe, expect, it } from 'vitest'
import {
  buildServerFromOption,
  commandPreview,
  defaultOptionIndex,
  isEntryAdded,
  manualHeaders,
  missingRequiredInputs,
  serverIdFor,
} from './marketplace'
import type { InstallInput, InstallOption, MarketplaceEntry } from '../types/marketplace'
import type { MCPServer } from '../types/config'

const input = (key: string, overrides: Partial<InstallInput> = {}): InstallInput => ({
  key,
  description: null,
  required: false,
  secret: false,
  defaultValue: null,
  ...overrides,
})

const npmOption: InstallOption = {
  kind: 'stdio',
  packageType: 'npm',
  identifier: '@upstash/context7-mcp',
  program: 'npx',
  argGroups: [['-y'], ['@upstash/context7-mcp@4.1.1']],
  env: { CONTEXT7_API_KEY: '{CONTEXT7_API_KEY}', MODE: 'stdio' },
  inputs: [input('CONTEXT7_API_KEY', { required: true, secret: true })],
  inferred: false,
}

const dockerOption: InstallOption = {
  kind: 'stdio',
  packageType: 'oci',
  identifier: 'ghcr.io/github/github-mcp-server:1.12.2',
  program: 'docker',
  argGroups: [
    ['run', '-i', '--rm'],
    ['-e', 'GITHUB_PERSONAL_ACCESS_TOKEN={token}'],
    ['--region', '{region}'],
    ['ghcr.io/github/github-mcp-server:1.12.2'],
  ],
  env: { LOG_LEVEL: '{LOG_LEVEL}' },
  inputs: [input('token', { secret: true }), input('region', { defaultValue: 'us' }), input('LOG_LEVEL')],
  inferred: true,
}

const httpOption: InstallOption = {
  kind: 'http',
  transport: 'streamable-http',
  url: 'https://{tenant}.example.com/mcp',
  headers: { Authorization: 'Bearer {API_TOKEN}' },
  inputs: [input('tenant', { required: true }), input('API_TOKEN', { required: true, secret: true })],
}

const unsupported: InstallOption = { kind: 'unsupported', packageType: 'mcpb', identifier: 'https://x/y.mcpb' }

const entry = (overrides: Partial<MarketplaceEntry> = {}): MarketplaceEntry => ({
  sourceId: 'github',
  id: 'io.github.upstash/context7',
  title: 'Context7',
  description: 'Up-to-date code docs',
  version: '4.1.1',
  repositoryUrl: 'https://github.com/upstash/context7',
  websiteUrl: null,
  stars: 1000,
  readmeExcerpt: null,
  installOptions: [npmOption],
  ...overrides,
})

const existing = (id: string, homepage?: string) => ({ id, homepage }) as MCPServer

describe('serverIdFor', () => {
  it('uses the last segment of the registry name', () => {
    expect(serverIdFor(entry(), [])).toBe('context7')
  })

  it('normalizes characters that are awkward in config keys', () => {
    expect(serverIdFor(entry({ id: 'com.Example/My Server!' }), [])).toBe('my-server')
  })

  it('adds a numeric suffix on conflict', () => {
    expect(serverIdFor(entry(), ['context7'])).toBe('context7-2')
    expect(serverIdFor(entry(), ['context7', 'context7-2'])).toBe('context7-3')
  })
})

describe('defaultOptionIndex', () => {
  it('prefers a declared npm or pypi package', () => {
    expect(defaultOptionIndex([unsupported, dockerOption, httpOption, npmOption])).toBe(3)
  })

  it('falls back to any runnable option, then the first one', () => {
    expect(defaultOptionIndex([unsupported, dockerOption, httpOption])).toBe(1)
    expect(defaultOptionIndex([unsupported, httpOption])).toBe(1)
    expect(defaultOptionIndex([unsupported])).toBe(0)
  })
})

describe('missingRequiredInputs', () => {
  it('reports required inputs left empty', () => {
    expect(missingRequiredInputs(npmOption, {}).map((item) => item.key)).toEqual(['CONTEXT7_API_KEY'])
    expect(missingRequiredInputs(npmOption, { CONTEXT7_API_KEY: '   ' })).toHaveLength(1)
    expect(missingRequiredInputs(npmOption, { CONTEXT7_API_KEY: 'k' })).toEqual([])
  })

  it('does not require header-only inputs because headers are configured manually', () => {
    expect(missingRequiredInputs(httpOption, {}).map((item) => item.key)).toEqual(['tenant'])
  })
})

describe('manualHeaders', () => {
  it('lists headers of an http option', () => {
    expect(manualHeaders(httpOption)).toEqual([{ name: 'Authorization', value: 'Bearer {API_TOKEN}' }])
    expect(manualHeaders(npmOption)).toEqual([])
  })
})

describe('buildServerFromOption', () => {
  it('builds a stdio server with substituted env and every client disabled', () => {
    const server = buildServerFromOption(entry(), npmOption, { CONTEXT7_API_KEY: 'secret' }, [])
    expect(server).toMatchObject({
      id: 'context7',
      name: 'Context7',
      enabled: true,
      description: 'Up-to-date code docs',
      homepage: 'https://github.com/upstash/context7',
      transport: { type: 'stdio' },
      command: {
        program: 'npx',
        args: ['-y', '@upstash/context7-mcp@4.1.1'],
        env: { CONTEXT7_API_KEY: 'secret', MODE: 'stdio' },
      },
    })
    expect(Object.values(server.apps).every((enabled) => !enabled)).toBe(true)
  })

  it('drops optional argument groups and env entries left empty, and applies defaults', () => {
    const server = buildServerFromOption(entry({ installOptions: [dockerOption] }), dockerOption, {}, [])
    expect(server.command?.args).toEqual([
      'run',
      '-i',
      '--rm',
      '--region',
      'us',
      'ghcr.io/github/github-mcp-server:1.12.2',
    ])
    expect(server.command?.env).toEqual({})
  })

  it('keeps optional groups once filled in', () => {
    const server = buildServerFromOption(entry(), dockerOption, { token: 'ghp_x', LOG_LEVEL: 'debug' }, [])
    expect(server.command?.args).toContain('GITHUB_PERSONAL_ACCESS_TOKEN=ghp_x')
    expect(server.command?.env).toEqual({ LOG_LEVEL: 'debug' })
  })

  it('builds an http server from the url', () => {
    const server = buildServerFromOption(entry(), httpOption, { tenant: 'acme' }, ['context7'])
    expect(server.id).toBe('context7-2')
    expect(server.transport).toEqual({ type: 'http', url: 'https://acme.example.com/mcp' })
    expect(server.command).toBeUndefined()
  })

  it('falls back to the website when there is no repository', () => {
    const server = buildServerFromOption(
      entry({ repositoryUrl: null, websiteUrl: 'https://context7.com' }),
      npmOption,
      { CONTEXT7_API_KEY: 'k' },
      [],
    )
    expect(server.homepage).toBe('https://context7.com')
  })
})

describe('commandPreview', () => {
  it('shows the command with inputs substituted and secrets masked', () => {
    expect(commandPreview(dockerOption, { token: 'ghp_secret' })).toBe(
      'docker run -i --rm -e GITHUB_PERSONAL_ACCESS_TOKEN=•••• --region us ghcr.io/github/github-mcp-server:1.12.2',
    )
  })

  it('shows the url for http options', () => {
    expect(commandPreview(httpOption, {})).toBe('https://{tenant}.example.com/mcp')
  })

  it('quotes arguments containing spaces', () => {
    const option: InstallOption = { ...npmOption, argGroups: [['--name', 'two words']] }
    expect(commandPreview(option, {})).toBe("npx --name 'two words'")
  })
})

describe('isEntryAdded', () => {
  it('matches by repository url or generated id', () => {
    expect(isEntryAdded(entry(), [existing('other', 'https://github.com/upstash/context7')])).toBe(true)
    expect(isEntryAdded(entry(), [existing('context7')])).toBe(true)
    expect(isEntryAdded(entry(), [existing('context7-docs', 'https://github.com/someone/else')])).toBe(false)
  })

  it('ignores a trailing slash or .git in repository urls', () => {
    expect(isEntryAdded(entry(), [existing('x', 'https://github.com/upstash/context7.git')])).toBe(true)
    expect(isEntryAdded(entry(), [existing('x', 'https://github.com/Upstash/context7/')])).toBe(true)
  })
})

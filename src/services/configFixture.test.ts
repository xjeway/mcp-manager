import { describe, expect, it } from 'vitest'
import YAML from 'yaml'
import fixture from '../../crates/mcp-manager-core/tests/fixtures/servers.frontend.yaml?raw'
import type { MCPConfig } from '../types/config'

// The Rust core parses this same file (crates/mcp-manager-core/src/store.rs),
// so the app and the CLI agree on the servers.yaml the frontend writes.
const config: MCPConfig = {
  version: 1,
  servers: [
    {
      id: 'linear',
      name: 'linear',
      enabled: true,
      transport: { type: 'http', url: 'https://mcp.linear.app/mcp' },
      apps: {
        vscode: false,
        cursor: true,
        claudeCode: true,
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
    {
      description: 'Docs lookup',
      id: 'context7',
      name: 'context7',
      enabled: true,
      transport: { type: 'stdio' },
      command: { program: 'npx', args: ['-y', '@upstash/context7-mcp@latest'], env: { TOKEN: '${env:TOKEN}' } },
      apps: {
        vscode: false,
        cursor: true,
        claudeCode: false,
        claudeDesktop: false,
        codex: true,
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
      placements: [{ app: 'claudeCode', scope: 'workspace', enabled: true, managed: true }],
    },
  ],
}

describe('servers.yaml fixture shared with the Rust core', () => {
  it('matches what the frontend writes', () => {
    expect(fixture).toBe(YAML.stringify(config))
  })
})

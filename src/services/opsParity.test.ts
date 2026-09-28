import { describe, expect, it } from 'vitest'
import sharedCases from '../../crates/mcp-manager-core/tests/fixtures/ops-cases.json'
import type { MCPConfig, MCPServer, SupportedApp } from '../types/config'
import { setWorkspacePlacement } from '../view-models/workspace'
import { mergeServerIntoConfig } from './pendingChanges'
import { deleteServerFromConfig, importServersIntoConfig, setServersAppInConfig } from './workspacePersistence'

// The same cases run against the Rust port in crates/mcp-manager-core/src/ops.rs,
// so the app and the CLI edit servers.yaml the same way.
interface OpsCase {
  name: string
  op: 'upsertServer' | 'removeServer' | 'setAppEnabled' | 'setWorkspacePlacement' | 'importServers'
  args: Record<string, unknown>
  input: MCPConfig
  expected: MCPConfig
  expectedAdded?: string[]
  expectedUpdated?: string[]
}

const cases = sharedCases as unknown as OpsCase[]

/** Drops `undefined` fields and empty placement lists, which both sides treat as absent. */
function normalize(config: MCPConfig): MCPConfig {
  const plain = JSON.parse(JSON.stringify(config)) as MCPConfig
  for (const server of plain.servers) {
    if (server.placements?.length === 0) {
      delete server.placements
    }
  }
  return plain
}

function run(testCase: OpsCase): { config: MCPConfig; added?: string[]; updated?: string[] } {
  const { args, input } = testCase
  switch (testCase.op) {
    case 'upsertServer':
      return { config: mergeServerIntoConfig(input, args.editingId as string | null, args.server as MCPServer) }
    case 'removeServer':
      return { config: deleteServerFromConfig(input, args.serverId as string) }
    case 'setAppEnabled':
      return {
        config: setServersAppInConfig(
          input,
          args.serverIds as string[],
          args.app as SupportedApp,
          args.enabled as boolean,
        ),
      }
    case 'setWorkspacePlacement':
      return {
        config: {
          ...input,
          servers: input.servers.map((server) =>
            server.id === args.serverId
              ? {
                  ...server,
                  placements: setWorkspacePlacement(
                    server.placements ?? [],
                    args.app as SupportedApp,
                    args.path as string,
                    args.enabled as boolean,
                  ),
                }
              : server,
          ),
        },
      }
    case 'importServers':
      return importServersIntoConfig(input, args.servers as MCPServer[])
  }
}

describe('config operations shared with the Rust core', () => {
  it.each(cases.map((testCase) => [testCase.name, testCase] as const))('%s', (_name, testCase) => {
    const result = run(testCase)

    expect(normalize(result.config)).toEqual(normalize(testCase.expected))
    if (testCase.expectedAdded) {
      expect(result.added).toEqual(testCase.expectedAdded)
      expect(result.updated).toEqual(testCase.expectedUpdated)
    }
  })
})

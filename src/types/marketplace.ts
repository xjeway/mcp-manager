// Mirrors src-tauri/src/marketplace/mod.rs (serde camelCase).

export type SourceTrust = 'curated' | 'official' | 'community'

export interface MarketplaceSource {
  id: string
  kind: 'mcp-registry'
  label: string
  baseUrl: string
  trust: SourceTrust
  serverSearch: boolean
}

export interface InstallInput {
  key: string
  description: string | null
  required: boolean
  secret: boolean
  defaultValue: string | null
}

export type PackageType = 'npm' | 'pypi' | 'oci' | 'other'

export type InstallOption =
  | {
      kind: 'stdio'
      packageType: PackageType
      identifier: string
      program: string
      /** A named flag and its value share a group so they are dropped together. */
      argGroups: string[][]
      env: Record<string, string>
      inputs: InstallInput[]
      /** The package type or runtime was guessed rather than declared by the publisher. */
      inferred: boolean
    }
  | {
      kind: 'http'
      transport: string
      url: string
      headers: Record<string, string>
      inputs: InstallInput[]
    }
  | {
      kind: 'unsupported'
      packageType: string
      identifier: string
    }

export interface MarketplaceEntry {
  sourceId: string
  id: string
  title: string
  description: string
  version: string | null
  repositoryUrl: string | null
  websiteUrl: string | null
  stars: number | null
  readmeExcerpt: string | null
  installOptions: InstallOption[]
}

export interface SearchPage {
  entries: MarketplaceEntry[]
  nextCursor: string | null
  skipped: number
  stale: boolean
  /** Unix seconds. */
  fetchedAt: number
}

export type MarketplaceErrorCode = 'disabled' | 'unknown-source' | 'network' | 'parse' | 'unavailable'

export interface MarketplaceError {
  code: MarketplaceErrorCode
  message?: string
}

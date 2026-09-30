import { invoke } from '@tauri-apps/api/core'
import YAML from 'yaml'
import type { ApplyWarning, ImportDetectedResult, MCPConfig, SupportedApp, WorkspaceContext } from '../types/config'
import { EMPTY_WORKSPACE, SUPPORTED_APPS } from '../types/config'
import { isDesktopRuntime } from './runtime'

const CONFIG_PATH = 'config/servers.yaml'
const BROWSER_CONFIG_KEY = 'mcp-manager-browser-config'

function defaultConfig(): MCPConfig {
  return {
    version: 1,
    servers: [
      {
        id: 'chrome-devtools',
        name: 'chrome-devtools',
        enabled: true,
        transport: { type: 'stdio' },
        command: {
          program: 'npx',
          args: ['chrome-devtools-mcp@latest'],
          env: {},
        },
        apps: {
          vscode: true,
          cursor: true,
          claudeCode: true,
          claudeDesktop: true,
          codex: true,
          openCode: true,
          githubCopilot: true,
          geminiCli: true,
          antigravity: true,
          iFlow: true,
          qwenCode: true,
          cline: true,
          windsurf: true,
          kiro: true,
          qoder: false,
        },
      },
      {
        id: 'linear',
        name: 'linear',
        enabled: true,
        transport: { type: 'http', url: 'https://mcp.linear.app/mcp' },
        apps: {
          vscode: false,
          cursor: true,
          claudeCode: true,
          claudeDesktop: true,
          codex: false,
          openCode: true,
          githubCopilot: false,
          geminiCli: true,
          antigravity: true,
          iFlow: true,
          qwenCode: true,
          cline: true,
          windsurf: true,
          kiro: true,
          qoder: false,
        },
      },
      {
        id: 'context7',
        name: 'context7',
        enabled: true,
        transport: { type: 'stdio' },
        command: {
          program: 'npx',
          args: ['-y', '@upstash/context7-mcp@latest'],
          env: {},
        },
        apps: {
          vscode: false,
          cursor: true,
          claudeCode: false,
          claudeDesktop: false,
          codex: false,
          openCode: true,
          githubCopilot: true,
          geminiCli: true,
          antigravity: true,
          iFlow: true,
          qwenCode: true,
          cline: true,
          windsurf: true,
          kiro: true,
          qoder: false,
        },
      },
    ],
  }
}

function loadBrowserConfig(): MCPConfig {
  const stored = window.localStorage.getItem(BROWSER_CONFIG_KEY)
  if (!stored) {
    const seeded = defaultConfig()
    window.localStorage.setItem(BROWSER_CONFIG_KEY, JSON.stringify(seeded))
    return seeded
  }

  try {
    const parsed = JSON.parse(stored) as MCPConfig
    return parsed?.version ? parsed : defaultConfig()
  } catch {
    return defaultConfig()
  }
}

/** Prefix of the backend error for a save that lost a race with another program. */
const CONFLICT_ERROR = 'CONFIG_CONFLICT'

/** Thrown by `saveConfig` when servers.yaml changed on disk since it was loaded (e.g. by the CLI). */
export class ConfigConflictError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'ConfigConflictError'
  }
}

/** Prefix of the backend error for servers.yaml saved in a newer format than this build writes. */
const TOO_NEW_ERROR = 'CONFIG_TOO_NEW'

/** Thrown by `saveConfig` when a newer MCP Manager or mcpmgr saved servers.yaml; this build only reads it. */
export class ConfigTooNewError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'ConfigTooNewError'
  }
}

interface StoredText {
  content: string
  fingerprint: string
}

/**
 * Thrown by `loadConfig` when servers.yaml exists but cannot be read or parsed,
 * and by `saveConfig` until it loads again or `startOverConfig` replaces it, so
 * nothing is ever saved over servers the app could not see.
 */
export class ConfigUnreadableError extends Error {
  constructor(
    readonly path: string,
    readonly detail: string,
  ) {
    super(`${path} could not be read: ${detail}`)
    this.name = 'ConfigUnreadableError'
  }
}

// Fingerprint of servers.yaml as last loaded or saved by this window; `null`
// until the first successful load, which leaves saves unchecked.
let loadedFingerprint: string | null = null
// Set while servers.yaml is unreadable; saves are refused until it is cleared.
let unreadable: ConfigUnreadableError | null = null

/** The config in `content`; throws when it is not a server list this app wrote. */
function parseStoredConfig(content: string): MCPConfig {
  // The backend reads an empty file as an empty list too.
  if (!content.trim()) {
    return { version: 1, servers: [] }
  }
  const parsed: unknown = YAML.parse(content)
  if (
    typeof parsed !== 'object' ||
    parsed === null ||
    typeof (parsed as MCPConfig).version !== 'number' ||
    !Array.isArray((parsed as MCPConfig).servers)
  ) {
    throw new Error('expected a `version` number and a `servers` list')
  }
  return parsed as MCPConfig
}

async function configFilePath(): Promise<string> {
  try {
    return await invoke<string>('yaml_config_path', { relativePath: CONFIG_PATH })
  } catch {
    return CONFIG_PATH
  }
}

async function markUnreadable(error: unknown): Promise<never> {
  const detail = error instanceof Error ? error.message : String(error)
  unreadable = new ConfigUnreadableError(await configFilePath(), detail)
  throw unreadable
}

export async function loadConfig(): Promise<MCPConfig> {
  if (!isDesktopRuntime()) {
    return loadBrowserConfig()
  }

  let stored: StoredText
  try {
    stored = await invoke<StoredText>('load_yaml_config', { relativePath: CONFIG_PATH })
  } catch (error) {
    return markUnreadable(error)
  }
  // Kept even for an unreadable file, so a later edit to it is noticed.
  loadedFingerprint = stored.fingerprint
  try {
    const config = parseStoredConfig(stored.content)
    unreadable = null
    return config
  } catch (error) {
    return markUnreadable(error)
  }
}

/**
 * Replaces an unreadable servers.yaml with `config` (an empty list by
 * default), after the backend copies it to `servers.yaml.broken-<timestamp>`
 * (returned as `backupPath`). Throws `ConfigConflictError` if the file changed
 * meanwhile.
 */
export async function startOverConfig(
  config: MCPConfig = { version: 1, servers: [] },
): Promise<{ config: MCPConfig; backupPath: string }> {
  try {
    const result = await invoke<{ backupPath: string; fingerprint: string }>('reset_yaml_config', {
      relativePath: CONFIG_PATH,
      content: YAML.stringify(config),
      expectedFingerprint: loadedFingerprint ?? '',
    })
    loadedFingerprint = result.fingerprint
    unreadable = null
    return { config, backupPath: result.backupPath }
  } catch (error) {
    if (String(error).startsWith(CONFLICT_ERROR)) {
      throw new ConfigConflictError(String(error))
    }
    throw error
  }
}

export async function saveConfig(config: MCPConfig): Promise<void> {
  if (!isDesktopRuntime()) {
    window.localStorage.setItem(BROWSER_CONFIG_KEY, JSON.stringify(config))
    return
  }

  if (unreadable) {
    throw unreadable
  }

  const text = YAML.stringify(config)
  try {
    loadedFingerprint = await invoke<string>('save_yaml_config', {
      relativePath: CONFIG_PATH,
      content: text,
      expectedFingerprint: loadedFingerprint,
    })
  } catch (error) {
    if (String(error).startsWith(CONFLICT_ERROR)) {
      throw new ConfigConflictError(String(error))
    }
    if (String(error).startsWith(TOO_NEW_ERROR)) {
      throw new ConfigTooNewError(String(error))
    }
    throw error
  }
}

/** Whether servers.yaml was changed by another program since this window loaded or saved it. */
export async function hasExternalConfigChange(): Promise<boolean> {
  if (!isDesktopRuntime()) {
    return false
  }
  if (loadedFingerprint === null) {
    // Never read it, so there is nothing to compare; retry while that is why.
    return unreadable !== null
  }

  const current = await invoke<string>('yaml_config_fingerprint', { relativePath: CONFIG_PATH })
  return current !== loadedFingerprint
}

export interface ApplyResult {
  backups: string[]
  warnings?: ApplyWarning[]
  /** Set when the change was applied but could not be recorded, so it cannot be undone. */
  historyError?: string
}

export async function applyConfig(
  config: MCPConfig,
  previousConfig: MCPConfig = config,
): Promise<ApplyResult> {
  if (!isDesktopRuntime()) {
    window.localStorage.setItem(BROWSER_CONFIG_KEY, JSON.stringify(config))
    browserUndoStack.push(previousConfig)
    return { backups: ['browser-preview-backup'] }
  }

  try {
    // The fingerprint `saveConfig` just got back: the backend refuses to apply
    // if another program saved servers.yaml in between.
    return await invoke<ApplyResult>('apply_config', {
      config,
      previousConfig,
      expectedFingerprint: loadedFingerprint,
    })
  } catch (error) {
    if (String(error).startsWith(CONFLICT_ERROR)) {
      throw new ConfigConflictError(String(error))
    }
    throw error
  }
}

// Browser preview has no history on disk: it remembers the config before each change it applied.
const browserUndoStack: MCPConfig[] = []

/** How many changes `rollback` can undo, one after another. */
export async function getRollbackDepth(): Promise<number> {
  if (!isDesktopRuntime()) {
    return browserUndoStack.length
  }

  return invoke<number>('rollback_depth')
}

/** Undoes the most recent change in servers.yaml and the client files; reload the config afterwards. */
export async function rollback(): Promise<void> {
  if (!isDesktopRuntime()) {
    const previous = browserUndoStack.pop()
    if (previous) {
      window.localStorage.setItem(BROWSER_CONFIG_KEY, JSON.stringify(previous))
    }
    return
  }

  await invoke('rollback_last_change')
}

export async function importDetectedConfigs(): Promise<ImportDetectedResult> {
  if (!isDesktopRuntime()) {
    const config = defaultConfig()
    window.localStorage.setItem(BROWSER_CONFIG_KEY, JSON.stringify(config))
    return {
      config,
      sources: [
        { app: 'yaml', path: '~/config/servers.yaml', exists: true, format: 'yaml', priority: 1 },
        { app: 'cursor', path: '~/.cursor/mcp.json', exists: true, format: 'json', priority: 2 },
      ],
      warnings: [],
      errors: [],
    }
  }

  return invoke<ImportDetectedResult>('import_detected_configs')
}

export async function detectInstalledApps(): Promise<SupportedApp[]> {
  if (!isDesktopRuntime()) {
    return [...SUPPORTED_APPS]
  }

  return invoke<SupportedApp[]>('detect_installed_apps')
}

export async function getCurrentWorkspace(): Promise<WorkspaceContext> {
  if (!isDesktopRuntime()) {
    return EMPTY_WORKSPACE
  }

  return invoke<WorkspaceContext>('current_workspace')
}

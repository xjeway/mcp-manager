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

interface StoredText {
  content: string
  fingerprint: string
}

// Fingerprint of servers.yaml as last loaded or saved by this window; `null`
// until the first successful load, which leaves saves unchecked.
let loadedFingerprint: string | null = null

export async function loadConfig(): Promise<MCPConfig> {
  if (!isDesktopRuntime()) {
    return loadBrowserConfig()
  }

  try {
    const stored = await invoke<StoredText>('load_yaml_config', { relativePath: CONFIG_PATH })
    loadedFingerprint = stored.fingerprint
    const parsed = YAML.parse(stored.content) as MCPConfig
    return parsed?.version ? parsed : defaultConfig()
  } catch {
    return defaultConfig()
  }
}

export async function saveConfig(config: MCPConfig): Promise<void> {
  if (!isDesktopRuntime()) {
    window.localStorage.setItem(BROWSER_CONFIG_KEY, JSON.stringify(config))
    return
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
    throw error
  }
}

/** Whether servers.yaml was changed by another program since this window loaded or saved it. */
export async function hasExternalConfigChange(): Promise<boolean> {
  if (!isDesktopRuntime() || loadedFingerprint === null) {
    return false
  }

  const current = await invoke<string>('yaml_config_fingerprint', { relativePath: CONFIG_PATH })
  return current !== loadedFingerprint
}

export interface ApplyResult {
  backups: string[]
  warnings?: ApplyWarning[]
}

export async function applyConfig(
  config: MCPConfig,
  previousConfig: MCPConfig = config,
): Promise<ApplyResult> {
  if (!isDesktopRuntime()) {
    window.localStorage.setItem(BROWSER_CONFIG_KEY, JSON.stringify(config))
    return { backups: ['browser-preview-backup'] }
  }

  return invoke<ApplyResult>('apply_config', { config, previousConfig })
}

export async function rollback(backups: string[]): Promise<void> {
  if (!isDesktopRuntime()) {
    if (backups.length > 0) {
      window.localStorage.setItem(BROWSER_CONFIG_KEY, JSON.stringify(defaultConfig()))
    }
    return
  }

  await invoke('rollback_from_backups', { backups })
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

import { useMemo } from 'react'
import type { MCPConfig, SupportedApp } from '../types/config'
import { SUPPORTED_APPS } from '../types/config'

export function useAppStats(config: MCPConfig): Record<SupportedApp, number> {
  return useMemo(() => {
    return SUPPORTED_APPS.reduce<Record<SupportedApp, number>>((acc, app) => {
      acc[app] = config.servers.filter((s) => s.enabled && s.apps[app]).length
      return acc
    }, { vscode: 0, cursor: 0, claudeCode: 0, codex: 0 })
  }, [config.servers])
}

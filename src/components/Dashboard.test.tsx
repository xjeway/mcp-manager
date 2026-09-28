import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it, vi } from 'vitest'
import { Dashboard } from './Dashboard'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

describe('Dashboard', () => {
  it('does not render the re-detect hint banner', () => {
    const html = renderToStaticMarkup(
      <Dashboard
        busy="idle"
        canRollback={false}
        visibleApps={['vscode']}
        workspace={{
          stats: [],
          rows: [
            {
              id: 'server-1',
              name: 'Server 1',
              transportLabel: 'STDIO',
              copyValue: 'npx example',
              enabledApps: ['vscode'],
              placements: [],
            },
          ],
        }}
        onAdd={() => {}}
        onOpenRepository={() => {}}
        onSyncLocalConfig={() => {}}
        onOpenSettings={() => {}}
        onDelete={() => {}}
        onEdit={() => {}}
        onRollback={() => {}}
        onToggleApp={() => {}}
        onCopyCommand={() => {}}
      />,
    )

    expect(html).not.toContain('clientListAutoDetectedHint')
    expect(html).not.toContain('clientListRedetectAction')
  })

  it('does not render the floating save panel when there are pending changes', () => {
    const html = renderToStaticMarkup(
      <Dashboard
        busy="idle"
        canRollback={false}
        visibleApps={['vscode']}
        workspace={{
          stats: [],
          rows: [],
        }}
        onAdd={() => {}}
        onOpenRepository={() => {}}
        onSyncLocalConfig={() => {}}
        onOpenSettings={() => {}}
        onDelete={() => {}}
        onEdit={() => {}}
        onRollback={() => {}}
        onToggleApp={() => {}}
        onCopyCommand={() => {}}
      />,
    )

    expect(html).not.toContain('dashboard-floating-save-wrap')
    expect(html).not.toContain('saveChanges')
    expect(html).not.toContain('pendingApply')
  })

  it('does not render the apply button or risk summary callout', () => {
    const html = renderToStaticMarkup(
      <Dashboard
        busy="idle"
        canRollback={false}
        visibleApps={['vscode']}
        workspace={{
          stats: [],
          rows: [],
        }}
        onAdd={() => {}}
        onOpenRepository={() => {}}
        onSyncLocalConfig={() => {}}
        onOpenSettings={() => {}}
        onDelete={() => {}}
        onEdit={() => {}}
        onRollback={() => {}}
        onToggleApp={() => {}}
        onCopyCommand={() => {}}
      />,
    )

    expect(html).not.toContain('aria-label="apply"')
    expect(html).not.toContain('riskSummary')
  })

  it('renders visible client pills in alphabetical order by name', () => {
    const html = renderToStaticMarkup(
      <Dashboard
        busy="idle"
        canRollback={false}
        visibleApps={['vscode', 'kiro', 'cursor']}
        workspace={{
          stats: [],
          rows: [
            {
              id: 'server-1',
              name: 'Server 1',
              transportLabel: 'STDIO',
              copyValue: 'npx example',
              enabledApps: ['cursor', 'kiro', 'vscode'],
              placements: [
                { app: 'cursor', label: 'Cursor', scope: 'user', scopeLabel: 'placementScopeUser' },
                { app: 'kiro', label: 'Kiro', scope: 'user', scopeLabel: 'placementScopeUser' },
                { app: 'vscode', label: 'VS Code', scope: 'user', scopeLabel: 'placementScopeUser' },
              ],
            },
          ],
        }}
        onAdd={() => {}}
        onOpenRepository={() => {}}
        onSyncLocalConfig={() => {}}
        onOpenSettings={() => {}}
        onDelete={() => {}}
        onEdit={() => {}}
        onRollback={() => {}}
        onToggleApp={() => {}}
        onCopyCommand={() => {}}
      />,
    )

    expect(html.indexOf('alt="Cursor"')).toBeLessThan(html.indexOf('alt="Kiro"'))
    expect(html.indexOf('alt="Kiro"')).toBeLessThan(html.indexOf('alt="VS Code"'))
  })

  it('keeps placement scope details out of the compact server list', () => {
    const html = renderToStaticMarkup(
      <Dashboard
        busy="idle"
        canRollback={false}
        visibleApps={['vscode', 'cursor']}
        workspace={{
          stats: [],
          rows: [
            {
              id: 'server-1',
              name: 'Server 1',
              transportLabel: 'STDIO',
              copyValue: 'npx example',
              enabledApps: ['cursor', 'vscode'],
              placements: [
                { app: 'cursor', label: 'Cursor', scope: 'workspace', scopeLabel: 'placementScopeWorkspace' },
                { app: 'vscode', label: 'VS Code', scope: 'user', scopeLabel: 'placementScopeUser' },
              ],
            },
          ],
        }}
        onAdd={() => {}}
        onOpenRepository={() => {}}
        onSyncLocalConfig={() => {}}
        onOpenSettings={() => {}}
        onDelete={() => {}}
        onEdit={() => {}}
        onRollback={() => {}}
        onToggleApp={() => {}}
        onCopyCommand={() => {}}
      />,
    )

    expect(html).toContain('alt="Cursor"')
    expect(html).toContain('alt="VS Code"')
    expect(html).not.toContain('placementScopeUser')
    expect(html).not.toContain('placementScopeWorkspace')
  })
})

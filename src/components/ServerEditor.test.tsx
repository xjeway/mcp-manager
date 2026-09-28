import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it, vi } from 'vitest'
import { ServerEditor } from './ServerEditor'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

describe('ServerEditor', () => {
  it('exposes an explicit action for the current workspace in the placement section', () => {
    const html = renderToStaticMarkup(
      <ServerEditor
        busy={false}
        server={null}
        workspace={{
          root: '/workspace/project',
          placementPaths: { vscode: '/workspace/project/.vscode/mcp.json' },
        }}
        visibleApps={['vscode']}
        onCancel={() => {}}
        onDraftChange={() => {}}
        onSave={() => {}}
      />,
    )

    expect(html).toContain('appScopeProject')
    expect(html).toContain('aria-label="openWorkspace"')
    expect(html).toContain('title=".vscode/mcp.json"')
  })

  it('hides the project section when there is no current project', () => {
    const html = renderToStaticMarkup(
      <ServerEditor
        busy={false}
        server={null}
        workspace={{ root: '', placementPaths: {} }}
        visibleApps={['vscode']}
        onCancel={() => {}}
        onDraftChange={() => {}}
        onSave={() => {}}
      />,
    )

    expect(html).not.toContain('appScopeProject')
  })
})

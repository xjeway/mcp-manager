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
        onSaveMany={() => {}}
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
        onSaveMany={() => {}}
      />,
    )

    expect(html).not.toContain('appScopeProject')
  })

  it('prefills a new server from initialServer while staying in add mode', () => {
    const html = renderToStaticMarkup(
      <ServerEditor
        busy={false}
        server={null}
        initialServer={{
          id: 'context7',
          name: 'Context7',
          enabled: true,
          transport: { type: 'stdio' },
          command: { program: 'npx', args: ['-y', '@upstash/context7-mcp@4.1.1'], env: {} },
          apps: { vscode: false } as never,
        }}
        workspace={{ root: '', placementPaths: {} }}
        visibleApps={['vscode']}
        onCancel={() => {}}
        onDraftChange={() => {}}
        onSave={() => {}}
        onSaveMany={() => {}}
      />,
    )

    expect(html).toContain('<h1>add</h1>')
    expect(html).toContain('value="Context7"')
    expect(html).toContain('value="@upstash/context7-mcp@4.1.1"')
  })
})

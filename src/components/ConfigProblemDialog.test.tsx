import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it, vi } from 'vitest'
import { ConfigProblemDialog } from './ConfigProblemDialog'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

function render(restorableCount: number) {
  return renderToStaticMarkup(
    <ConfigProblemDialog
      path="/data/config/servers.yaml"
      detail="Unexpected flow-seq-end at line 2"
      busy={false}
      restorableCount={restorableCount}
      onRestore={() => {}}
      onShowFile={() => {}}
      onReload={() => {}}
      onStartOver={() => {}}
    />,
  )
}

describe('ConfigProblemDialog', () => {
  it('names the file and the parse error, and offers reload and start over', () => {
    const html = render(0)

    expect(html).toContain('role="alertdialog"')
    expect(html).toContain('/data/config/servers.yaml')
    expect(html).toContain('Unexpected flow-seq-end at line 2')
    expect(html).toContain('configUnreadableReload')
    expect(html).toContain('configUnreadableStartOver')
    expect(html).not.toContain('configUnreadableRestore')
  })

  it('offers to restore the servers still shown from before the file broke', () => {
    expect(render(3)).toContain('configUnreadableRestore')
  })
})

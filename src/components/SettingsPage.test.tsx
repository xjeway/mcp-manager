import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it, vi } from 'vitest'
import { SettingsPage } from './SettingsPage'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

describe('SettingsPage', () => {
  it('does not render detected clients or auto-sync help copy', () => {
    const html = renderToStaticMarkup(
      <SettingsPage
        autoSyncOnLaunch
        busy={false}
        language="zh-CN"
        onOpenRepository={() => {}}
        onAutoSyncOnLaunchChange={() => {}}
        theme="system"
        onBack={() => {}}
        onCheckUpdates={() => {}}
        onLanguageChange={() => {}}
        onThemeChange={() => {}}
      />,
    )

    expect(html).not.toContain('settingsDetectedClients')
    expect(html).not.toContain('settingsDetectedClientsHelp')
    expect(html).not.toContain('settingsAutoSyncOnLaunchHelp')
    expect(html).toContain('settingsAutoSyncOnLaunch')
  })
})

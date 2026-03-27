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
        appVersion="9.9.9"
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

  it('renders settings icons and compact control hooks', () => {
    const html = renderToStaticMarkup(
      <SettingsPage
        appVersion="9.9.9"
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

    expect(html).toContain('settings-nav-item-icon')
    expect(html).toContain('settings-item-label')
    expect(html).toContain('settings-item-icon')
    expect(html).toContain('settings-select-compact')
    expect(html).toContain('settings-segment-control-compact')
    expect(html).toContain('settings-switch-compact')
    expect(html).toContain('settings-button-compact')
  })

  it('keeps the theme control at row end and applies compact value hooks', () => {
    const html = renderToStaticMarkup(
      <SettingsPage
        appVersion="9.9.9"
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

    expect(html).not.toContain('settings-item-stacked')
    expect(html).toContain('settings-action-slot-theme')
    expect(html).toContain('settings-item-value-compact')
  })

  it('renders the provided app version in the about section', () => {
    const html = renderToStaticMarkup(
      <SettingsPage
        appVersion="2.3.4"
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

    expect(html).toContain('v2.3.4')
    expect(html).toContain('>2.3.4<')
  })
})

import { invoke } from '@tauri-apps/api/core'
import { ArrowLeft, BadgeInfo, Boxes, ExternalLink, FolderGit2, Languages, Library, Monitor, Moon, Palette, RefreshCw, Search, Shield, Store, SunMedium } from 'lucide-react'
import { type ReactNode, useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  readAutomaticUpdateChecks,
  readPreferPinnedMarketplaceInstalls,
  saveAutomaticUpdateChecks,
  savePreferPinnedMarketplaceInstalls,
} from '../services/appPreferences'
import { AppLogo } from './AppLogo'
import { Segmented } from './Segmented'
import { LanguageMenu } from './LanguageMenu'
import { MarketplaceSourcesEditor } from './MarketplaceSourcesEditor'

interface SettingsPageProps {
  appVersion: string
  autoImportOnLaunch: boolean
  busy: boolean
  checkingUpdates: boolean
  language: string
  marketplaceEnabled: boolean
  onMarketplaceEnabledChange: (enabled: boolean) => void
  onOpenRepository: () => void
  onAutoImportOnLaunchChange: (enabled: boolean) => void
  theme: 'light' | 'dark' | 'system'
  onBack: () => void
  onCheckUpdates: () => void
  onLanguageChange: (language: 'zh-CN' | 'en-US') => void
  onThemeChange: (theme: 'light' | 'dark' | 'system') => void
}

function PrivacyControls() {
  const { t } = useTranslation()
  const [automaticUpdates, setAutomaticUpdates] = useState(readAutomaticUpdateChecks)
  const [preferPinned, setPreferPinned] = useState(readPreferPinnedMarketplaceInstalls)

  useEffect(() => {
    let cancelled = false
    void invoke<{ automaticUpdateChecks: boolean; preferPinnedMarketplaceInstalls: boolean }>('privacy_settings')
      .then((settings) => {
        if (cancelled) {
          return
        }
        setAutomaticUpdates(settings.automaticUpdateChecks)
        setPreferPinned(settings.preferPinnedMarketplaceInstalls)
        saveAutomaticUpdateChecks(settings.automaticUpdateChecks)
        savePreferPinnedMarketplaceInstalls(settings.preferPinnedMarketplaceInstalls)
      })
      .catch(() => undefined)
    return () => {
      cancelled = true
    }
  }, [])

  const persist = (automaticUpdateChecks: boolean, preferPinnedMarketplaceInstalls: boolean) => {
    saveAutomaticUpdateChecks(automaticUpdateChecks)
    savePreferPinnedMarketplaceInstalls(preferPinnedMarketplaceInstalls)
    void invoke('save_privacy_settings', {
      settings: { automaticUpdateChecks, preferPinnedMarketplaceInstalls },
    }).catch(() => undefined)
  }

  return (
    <>
      <div className="settings-item settings-item-block">
        <SettingsItemLabel icon={<Shield size={14} />} label={t('settingsPrivacy')} />
        <p className="settings-help">{t('settingsTelemetry')}</p>
        <p className="settings-help">{t('settingsTelemetryHelp')}</p>
      </div>
      <div className="settings-item">
        <div className="settings-item-stack">
          <SettingsItemLabel icon={<RefreshCw size={14} />} label={t('settingsAutomaticUpdates')} />
          <p className="settings-help">{t('settingsAutomaticUpdatesHelp')}</p>
        </div>
        <button
          type="button"
          className={automaticUpdates ? 'switch-control settings-switch-compact checked' : 'switch-control settings-switch-compact'}
          onClick={() => {
            const next = !automaticUpdates
            setAutomaticUpdates(next)
            persist(next, preferPinned)
          }}
          aria-pressed={automaticUpdates}
          aria-label={t('settingsAutomaticUpdates')}
        >
          <span className="switch-thumb" />
        </button>
      </div>
      <div className="settings-item">
        <div className="settings-item-stack">
          <SettingsItemLabel icon={<Shield size={14} />} label={t('settingsPreferPinned')} />
          <p className="settings-help">{t('settingsPreferPinnedHelp')}</p>
        </div>
        <button
          type="button"
          className={preferPinned ? 'switch-control settings-switch-compact checked' : 'switch-control settings-switch-compact'}
          onClick={() => {
            const next = !preferPinned
            setPreferPinned(next)
            persist(automaticUpdates, next)
          }}
          aria-pressed={preferPinned}
          aria-label={t('settingsPreferPinned')}
        >
          <span className="switch-thumb" />
        </button>
      </div>
    </>
  )
}

function SettingsItemLabel({ icon, label }: { icon: ReactNode; label: string }) {
  return (
    <div className="settings-item-label">
      <span className="settings-item-icon" aria-hidden="true">
        {icon}
      </span>
      <span>{label}</span>
    </div>
  )
}

export function SettingsPage({
  appVersion,
  autoImportOnLaunch,
  busy,
  checkingUpdates,
  language,
  marketplaceEnabled,
  onMarketplaceEnabledChange,
  onOpenRepository,
  onAutoImportOnLaunchChange,
  theme,
  onBack,
  onCheckUpdates,
  onLanguageChange,
  onThemeChange,
}: SettingsPageProps) {
  const { t } = useTranslation()
  const [query, setQuery] = useState('')
  const [activeSection, setActiveSection] = useState<'basic' | 'about'>('basic')
  const sectionRefs = {
    basic: useRef<HTMLElement | null>(null),
    about: useRef<HTMLElement | null>(null),
  }

  const sections = useMemo(
    () => [
      {
        id: 'basic' as const,
        title: t('settingsGroupGeneral'),
        icon: <Boxes size={14} />,
        keywords: [t('language'), t('theme'), t('settingsAutoImportOnLaunch'), t('syncLocalConfig'), t('settingsMarketplace'), t('settingsMarketplaceSources')]
          .join(' ')
          .toLowerCase(),
      },
      {
        id: 'about' as const,
        title: t('settingsGroupAbout'),
        icon: <BadgeInfo size={14} />,
        keywords: [t('settingsVersion'), t('settingsStack'), t('settingsAboutHelp'), t('checkUpdates'), t('settingsRepository')]
          .join(' ')
          .toLowerCase(),
      },
    ],
    [t],
  )

  const normalizedQuery = query.trim().toLowerCase()
  const visibleSections = sections.filter((section) =>
    !normalizedQuery || section.title.toLowerCase().includes(normalizedQuery) || section.keywords.includes(normalizedQuery),
  )

  const scrollToSection = (id: 'basic' | 'about') => {
    setActiveSection(id)
    sectionRefs[id].current?.scrollIntoView({ block: 'start', behavior: 'smooth' })
  }

  return (
    <div className="app-shell shell-flat settings-shell">
      <div className="mac-window-drag-region" data-tauri-drag-region aria-hidden="true" />
      <div className="page-header">
        <div className="brand-block">
          <button type="button" className="icon-button toolbar-icon" onClick={onBack} aria-label={t('back')}>
            <ArrowLeft size={16} />
          </button>
          <div>
            <p className="eyebrow">{t('settings')}</p>
            <h1 className="shell-title">{t('settingsTitle')}</h1>
          </div>
        </div>

        <div className="header-controls" data-tauri-no-drag>
          <div className="settings-header-hint">{t('settingsTitle')}</div>
        </div>
      </div>

      <div className="settings-workspace">
        <aside className="settings-sidebar">
          <div className="settings-search">
            <Search size={14} />
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t('settingsSearchPlaceholder')}
            />
          </div>

          <nav className="settings-nav">
            {visibleSections.map((section) => (
              <button
                key={section.id}
                type="button"
                className={activeSection === section.id ? 'settings-nav-item active' : 'settings-nav-item'}
                onClick={() => scrollToSection(section.id)}
              >
                <span className="settings-nav-item-icon" aria-hidden="true">
                  {section.icon}
                </span>
                <span className="settings-nav-item-copy">{section.title}</span>
              </button>
            ))}
          </nav>
        </aside>

        <div className="settings-content">
          {visibleSections.length === 0 ? (
            <section className="settings-card settings-card-empty">
              <p className="settings-help">{t('settingsSearchEmpty')}</p>
            </section>
          ) : null}

          {visibleSections.some((section) => section.id === 'basic') ? (
            <section ref={sectionRefs.basic} className="settings-card">
              <p className="settings-label">{t('settingsGroupGeneral')}</p>
              <div className="settings-list">
                <div className="settings-item">
                  <SettingsItemLabel icon={<Languages size={14} />} label={t('language')} />
                  <div className="settings-action-slot">
                    <LanguageMenu language={language} onChange={onLanguageChange} selectClassName="settings-select-compact" />
                  </div>
                </div>
                <div className="settings-item">
                  <SettingsItemLabel icon={<Palette size={14} />} label={t('theme')} />
                  <div className="settings-action-slot settings-action-slot-theme">
                    <Segmented
                      ariaLabel={t('theme')}
                      value={theme}
                      onChange={onThemeChange}
                      options={[
                        { icon: <SunMedium size={13} />, label: t('light'), value: 'light' },
                        { icon: <Moon size={13} />, label: t('dark'), value: 'dark' },
                        { icon: <Monitor size={13} />, label: t('system'), value: 'system' },
                      ]}
                    />
                  </div>
                </div>
                <div className="settings-item">
                  <SettingsItemLabel icon={<RefreshCw size={14} />} label={t('settingsAutoImportOnLaunch')} />
                  <button
                    type="button"
                    className={autoImportOnLaunch ? 'switch-control settings-switch-compact checked' : 'switch-control settings-switch-compact'}
                    onClick={() => onAutoImportOnLaunchChange(!autoImportOnLaunch)}
                    aria-pressed={autoImportOnLaunch}
                    aria-label={t('settingsAutoImportOnLaunch')}
                  >
                    <span className="switch-thumb" />
                  </button>
                </div>
                <div className="settings-item">
                  <div className="settings-item-stack">
                    <SettingsItemLabel icon={<Store size={14} />} label={t('settingsMarketplace')} />
                    <p className="settings-help">{t('settingsMarketplaceHelp')}</p>
                  </div>
                  <button
                    type="button"
                    className={marketplaceEnabled ? 'switch-control settings-switch-compact checked' : 'switch-control settings-switch-compact'}
                    onClick={() => onMarketplaceEnabledChange(!marketplaceEnabled)}
                    aria-pressed={marketplaceEnabled}
                    aria-label={t('settingsMarketplace')}
                  >
                    <span className="switch-thumb" />
                  </button>
                </div>
                {marketplaceEnabled ? (
                  <div className="settings-item settings-item-block">
                    <SettingsItemLabel icon={<Library size={14} />} label={t('settingsMarketplaceSources')} />
                    <p className="settings-help">{t('settingsMarketplaceSourcesHelp')}</p>
                    <MarketplaceSourcesEditor />
                  </div>
                ) : null}
                <PrivacyControls />
              </div>
            </section>
          ) : null}

          {visibleSections.some((section) => section.id === 'about') ? (
            <section ref={sectionRefs.about} className="settings-card">
              <p className="settings-label">{t('settingsGroupAbout')}</p>
              <div className="settings-about-hero">
                <button
                  type="button"
                  className="settings-about-logo-button"
                  onClick={onOpenRepository}
                  aria-label={t('settingsRepositoryAction')}
                >
                  <AppLogo className="brand-logo settings-about-logo" alt={t('title')} />
                </button>
                <div className="settings-about-copy">
                  <div className="settings-about-topline">
                    <div className="settings-value">{t('title')}</div>
                    <span className="settings-about-version">v{appVersion}</span>
                  </div>
                  <p className="settings-about-subtitle">{t('settingsAboutSubtitle')}</p>
                  <div className="settings-chip-row">
                    <span className="settings-chip">Tauri 2</span>
                    <span className="settings-chip">React 19</span>
                    <span className="settings-chip">TypeScript</span>
                  </div>
                </div>
              </div>
              <div className="settings-list">
                <div className="settings-item">
                  <SettingsItemLabel icon={<Boxes size={14} />} label={t('settingsStack')} />
                  <strong className="settings-item-value-compact">Tauri 2 / React / Rust</strong>
                </div>
                <div className="settings-item">
                  <SettingsItemLabel icon={<BadgeInfo size={14} />} label={t('settingsVersion')} />
                  <strong className="settings-item-value-compact">{appVersion}</strong>
                </div>
                <div className="settings-item">
                  <SettingsItemLabel icon={<FolderGit2 size={14} />} label={t('settingsRepository')} />
                  <button
                    type="button"
                    className="ghost-button compact settings-link-button settings-button-compact"
                    onClick={onOpenRepository}
                  >
                    <ExternalLink size={14} />
                    {t('settingsRepositoryAction')}
                  </button>
                </div>
                <div className="settings-item">
                  <SettingsItemLabel icon={<Monitor size={14} />} label={t('checkUpdates')} />
                  <button
                    type="button"
                    className={
                      checkingUpdates
                        ? 'ghost-button compact settings-update-button settings-button-compact is-checking'
                        : 'ghost-button compact settings-update-button settings-button-compact'
                    }
                    onClick={onCheckUpdates}
                    disabled={busy || checkingUpdates}
                    aria-busy={checkingUpdates}
                  >
                    <span className="settings-update-button-icon" aria-hidden="true">
                      <RefreshCw size={14} />
                    </span>
                    <span>{checkingUpdates ? t('checkingUpdates') : t('checkUpdates')}</span>
                  </button>
                </div>
              </div>
            </section>
          ) : null}
        </div>
      </div>
    </div>
  )
}

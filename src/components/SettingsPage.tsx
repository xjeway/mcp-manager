import { ArrowLeft, Monitor, Moon, RefreshCw, Search, SunMedium } from 'lucide-react'
import { useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { AppLogo } from './AppLogo'
import { CLIENTS, PLANNED_CLIENTS } from './clientMeta'
import { LanguageMenu } from './LanguageMenu'
import { Tooltip } from './Tooltip'

interface SettingsPageProps {
  busy: boolean
  language: string
  theme: 'light' | 'dark' | 'system'
  onBack: () => void
  onCheckUpdates: () => void
  onLanguageChange: (language: 'zh-CN' | 'en-US') => void
  onThemeChange: (theme: 'light' | 'dark' | 'system') => void
}

export function SettingsPage({
  busy,
  language,
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
        keywords: [t('language'), t('theme')].join(' ').toLowerCase(),
      },
      {
        id: 'about' as const,
        title: t('settingsGroupAbout'),
        keywords: [t('settingsVersion'), t('settingsStack'), t('settingsAboutHelp'), t('checkUpdates')]
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
          <Tooltip content={t('back')}>
            <button type="button" className="icon-button toolbar-icon" onClick={onBack} aria-label={t('back')}>
              <ArrowLeft size={16} />
            </button>
          </Tooltip>
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
                {section.title}
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
                  <span>{t('language')}</span>
                  <div className="settings-action-slot">
                    <LanguageMenu language={language} onChange={onLanguageChange} />
                  </div>
                </div>
                <div className="settings-item settings-item-stacked">
                  <span>{t('theme')}</span>
                  <div className="segment-control settings-segment-control">
                    {([
                      { icon: <SunMedium size={14} />, label: t('light'), value: 'light' as const },
                      { icon: <Moon size={14} />, label: t('dark'), value: 'dark' as const },
                      { icon: <Monitor size={14} />, label: t('system'), value: 'system' as const },
                    ]).map((option) => (
                      <button
                        key={option.value}
                        type="button"
                        className={theme === option.value ? 'segment-button active' : 'segment-button'}
                        onClick={() => onThemeChange(option.value)}
                      >
                        {option.icon}
                        <span>{option.label}</span>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            </section>
          ) : null}

          {visibleSections.some((section) => section.id === 'about') ? (
            <section ref={sectionRefs.about} className="settings-card">
              <p className="settings-label">{t('settingsGroupAbout')}</p>
              <div className="settings-about-hero">
                <AppLogo className="brand-logo settings-about-logo" alt={t('title')} />
                <div className="settings-about-copy">
                  <div className="settings-about-topline">
                    <div className="settings-value">{t('title')}</div>
                    <span className="settings-about-version">v0.1.0</span>
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
                  <span>{t('settingsStack')}</span>
                  <strong>Tauri 2 / React / Rust</strong>
                </div>
                <div className="settings-item">
                  <span>{t('settingsVersion')}</span>
                  <strong>0.1.0</strong>
                </div>
                <div className="settings-item settings-item-stacked">
                  <span>{t('settingsSupportedClients')}</span>
                  <div className="settings-chip-row">
                    {CLIENTS.map((client) => (
                      <span key={client.id} className="settings-chip">
                        {client.label}
                      </span>
                    ))}
                  </div>
                </div>
                <div className="settings-item settings-item-stacked">
                  <span>{t('settingsPlannedClients')}</span>
                  <div className="settings-chip-row">
                    {PLANNED_CLIENTS.map((client) => (
                      <span key={client} className="settings-chip">
                        {client}
                      </span>
                    ))}
                  </div>
                </div>
                <div className="settings-item">
                  <span>{t('checkUpdates')}</span>
                  <button type="button" className="ghost-button compact settings-update-button" onClick={onCheckUpdates} disabled={busy}>
                    <RefreshCw size={14} />
                    {t('checkUpdates')}
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

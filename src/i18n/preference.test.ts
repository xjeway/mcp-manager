// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

function stubSystemLanguages(languages: string[]) {
  vi.spyOn(window.navigator, 'languages', 'get').mockReturnValue(languages)
  vi.spyOn(window.navigator, 'language', 'get').mockReturnValue(languages[0])
}

async function loadI18n() {
  vi.resetModules()
  return import('./index')
}

describe('language preference', () => {
  beforeEach(() => {
    window.localStorage.clear()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('follows the system language by default', async () => {
    stubSystemLanguages(['ja-JP', 'en-US'])
    const { default: i18n, getLanguagePreference } = await loadI18n()

    expect(getLanguagePreference()).toBe('system')
    expect(i18n.language).toBe('ja-JP')
    expect(document.documentElement.lang).toBe('ja-JP')
  })

  it('uses the first supported system language', async () => {
    stubSystemLanguages(['it-IT', 'de-CH'])
    const { default: i18n } = await loadI18n()

    expect(i18n.language).toBe('de-DE')
  })

  it('falls back to English when no system language is supported', async () => {
    stubSystemLanguages(['it-IT'])
    const { default: i18n } = await loadI18n()

    expect(i18n.language).toBe('en-US')
  })

  it('keeps an explicitly chosen language', async () => {
    stubSystemLanguages(['ja-JP'])
    window.localStorage.setItem('ui-language-preference', 'fr-FR')
    const { default: i18n, getLanguagePreference } = await loadI18n()

    expect(getLanguagePreference()).toBe('fr-FR')
    expect(i18n.language).toBe('fr-FR')
  })

  it('switches between system and explicit preferences', async () => {
    stubSystemLanguages(['ko-KR'])
    const { default: i18n, getLanguagePreference, setLanguagePreference } = await loadI18n()

    setLanguagePreference('ru-RU')
    expect(i18n.language).toBe('ru-RU')
    expect(getLanguagePreference()).toBe('ru-RU')

    setLanguagePreference('system')
    expect(i18n.language).toBe('ko-KR')
    expect(getLanguagePreference()).toBe('system')
  })

  it('follows OS language changes only while set to system', async () => {
    stubSystemLanguages(['en-US'])
    const { default: i18n, setLanguagePreference } = await loadI18n()

    stubSystemLanguages(['zh-TW'])
    window.dispatchEvent(new Event('languagechange'))
    expect(i18n.language).toBe('zh-TW')

    setLanguagePreference('es-ES')
    stubSystemLanguages(['pt-BR'])
    window.dispatchEvent(new Event('languagechange'))
    expect(i18n.language).toBe('es-ES')
  })

  it('migrates a legacy language that matches the system to "system"', async () => {
    stubSystemLanguages(['zh-CN'])
    window.localStorage.setItem('ui-language', 'zh-CN')
    const { getLanguagePreference } = await loadI18n()

    expect(getLanguagePreference()).toBe('system')
    expect(window.localStorage.getItem('ui-language')).toBeNull()
  })

  it('keeps a legacy language that differs from the system as an explicit choice', async () => {
    stubSystemLanguages(['en-US'])
    window.localStorage.setItem('ui-language', 'zh-CN')
    const { default: i18n, getLanguagePreference } = await loadI18n()

    expect(getLanguagePreference()).toBe('zh-CN')
    expect(i18n.language).toBe('zh-CN')
  })
})

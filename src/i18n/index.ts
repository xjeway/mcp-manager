import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import {
  DEFAULT_LANGUAGE,
  isLanguagePreference,
  isSupportedLanguage,
  matchLanguage,
  type AppLanguage,
  type LanguagePreference,
} from './languages'
import deDE from './locales/de-DE'
import enUS from './locales/en-US'
import esES from './locales/es-ES'
import frFR from './locales/fr-FR'
import itIT from './locales/it-IT'
import jaJP from './locales/ja-JP'
import koKR from './locales/ko-KR'
import ptBR from './locales/pt-BR'
import ruRU from './locales/ru-RU'
import zhCN from './locales/zh-CN'
import zhTW from './locales/zh-TW'

const PREFERENCE_KEY = 'ui-language-preference'
// Earlier versions stored the active language here on every launch, so it
// does not tell an explicit choice apart from the auto-detected default.
const LEGACY_LANGUAGE_KEY = 'ui-language'

export const resources = {
  'en-US': { translation: enUS },
  'zh-CN': { translation: zhCN },
  'zh-TW': { translation: zhTW },
  'ja-JP': { translation: jaJP },
  'ko-KR': { translation: koKR },
  'de-DE': { translation: deDE },
  'fr-FR': { translation: frFR },
  'es-ES': { translation: esES },
  'it-IT': { translation: itIT },
  'pt-BR': { translation: ptBR },
  'ru-RU': { translation: ruRU },
} as const satisfies Record<AppLanguage, unknown>

export function resolveSystemLanguage(): AppLanguage {
  const navigator = typeof window === 'undefined' ? undefined : window.navigator
  if (!navigator) {
    return DEFAULT_LANGUAGE
  }

  const preferred = navigator.languages?.length ? navigator.languages : [navigator.language]
  for (const tag of preferred) {
    const match = matchLanguage(tag)
    if (match) {
      return match
    }
  }

  return DEFAULT_LANGUAGE
}

export function resolveLanguage(preference: LanguagePreference): AppLanguage {
  return preference === 'system' ? resolveSystemLanguage() : preference
}

function readStorage(key: string): string | null {
  try {
    return window.localStorage.getItem(key)
  } catch {
    return null
  }
}

function writeStorage(key: string, value: string | null) {
  try {
    if (value === null) {
      window.localStorage.removeItem(key)
    } else {
      window.localStorage.setItem(key, value)
    }
  } catch {
    // Storage can be unavailable (private mode, quota); the preference then lasts for this session only.
  }
}

export function getLanguagePreference(): LanguagePreference {
  if (typeof window === 'undefined') {
    return 'system'
  }

  const stored = readStorage(PREFERENCE_KEY)
  if (isLanguagePreference(stored)) {
    return stored
  }

  // Keep a legacy language only when it differs from what the system would
  // pick, i.e. the user must have chosen it; otherwise follow the system.
  const legacy = readStorage(LEGACY_LANGUAGE_KEY)
  const migrated: LanguagePreference = isSupportedLanguage(legacy) && legacy !== resolveSystemLanguage() ? legacy : 'system'
  writeStorage(PREFERENCE_KEY, migrated)
  writeStorage(LEGACY_LANGUAGE_KEY, null)
  return migrated
}

export function setLanguagePreference(preference: LanguagePreference) {
  if (typeof window !== 'undefined') {
    writeStorage(PREFERENCE_KEY, preference)
  }
  void i18n.changeLanguage(resolveLanguage(preference))
}

function syncDocumentLanguage(language: string) {
  if (typeof document !== 'undefined') {
    document.documentElement.lang = language
  }
}

i18n.use(initReactI18next).init({
  resources,
  lng: resolveLanguage(getLanguagePreference()),
  fallbackLng: DEFAULT_LANGUAGE,
  interpolation: { escapeValue: false },
})

syncDocumentLanguage(i18n.language)
i18n.on('languageChanged', syncDocumentLanguage)

if (typeof window !== 'undefined' && typeof window.addEventListener === 'function') {
  // Follow OS language changes while the app is running.
  window.addEventListener('languagechange', () => {
    if (getLanguagePreference() === 'system') {
      void i18n.changeLanguage(resolveSystemLanguage())
    }
  })

  // Keep other windows (e.g. the update progress window) in step.
  window.addEventListener('storage', (event) => {
    if (event.key === PREFERENCE_KEY && isLanguagePreference(event.newValue)) {
      void i18n.changeLanguage(resolveLanguage(event.newValue))
    }
  })
}

export default i18n

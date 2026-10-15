import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import { DEFAULT_LANGUAGE, isSupportedLanguage, matchLanguage, type AppLanguage } from './languages'
import deDE from './locales/de-DE'
import enUS from './locales/en-US'
import esES from './locales/es-ES'
import frFR from './locales/fr-FR'
import jaJP from './locales/ja-JP'
import koKR from './locales/ko-KR'
import ptBR from './locales/pt-BR'
import ruRU from './locales/ru-RU'
import zhCN from './locales/zh-CN'
import zhTW from './locales/zh-TW'

const LANGUAGE_KEY = 'ui-language'

function resolveInitialLanguage(): AppLanguage {
  if (typeof window === 'undefined') {
    return DEFAULT_LANGUAGE
  }

  const stored = window.localStorage.getItem(LANGUAGE_KEY)
  if (isSupportedLanguage(stored)) {
    return stored
  }

  const preferred = window.navigator.languages?.length ? window.navigator.languages : [window.navigator.language]
  for (const tag of preferred) {
    const match = matchLanguage(tag)
    if (match) {
      return match
    }
  }

  return DEFAULT_LANGUAGE
}

export const resources = {
  'en-US': { translation: enUS },
  'zh-CN': { translation: zhCN },
  'zh-TW': { translation: zhTW },
  'ja-JP': { translation: jaJP },
  'ko-KR': { translation: koKR },
  'de-DE': { translation: deDE },
  'fr-FR': { translation: frFR },
  'es-ES': { translation: esES },
  'pt-BR': { translation: ptBR },
  'ru-RU': { translation: ruRU },
} as const satisfies Record<AppLanguage, unknown>

function syncDocumentLanguage(language: string) {
  if (typeof document !== 'undefined') {
    document.documentElement.lang = language
  }
}

i18n.use(initReactI18next).init({
  resources,
  lng: resolveInitialLanguage(),
  fallbackLng: DEFAULT_LANGUAGE,
  interpolation: { escapeValue: false },
})

syncDocumentLanguage(i18n.language)

i18n.on('languageChanged', (language) => {
  syncDocumentLanguage(language)
  if (typeof window !== 'undefined' && isSupportedLanguage(language)) {
    window.localStorage.setItem(LANGUAGE_KEY, language)
  }
})

export default i18n

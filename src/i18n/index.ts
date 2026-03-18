import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import enUS from './locales/en-US'
import zhCN from './locales/zh-CN'

const LANGUAGE_KEY = 'ui-language'

function resolveInitialLanguage(): 'zh-CN' | 'en-US' {
  if (typeof window === 'undefined') {
    return 'zh-CN'
  }

  const stored = window.localStorage.getItem(LANGUAGE_KEY)
  if (stored === 'zh-CN' || stored === 'en-US') {
    return stored
  }

  const browserLanguage = window.navigator.language
  return browserLanguage.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US'
}

const resources = {
  'en-US': { translation: enUS },
  'zh-CN': { translation: zhCN },
} as const

i18n.use(initReactI18next).init({
  resources,
  lng: resolveInitialLanguage(),
  fallbackLng: 'en-US',
  interpolation: { escapeValue: false },
})

i18n.on('languageChanged', (language) => {
  if (typeof window !== 'undefined' && (language === 'zh-CN' || language === 'en-US')) {
    window.localStorage.setItem(LANGUAGE_KEY, language)
  }
})

export default i18n

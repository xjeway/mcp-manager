export const SUPPORTED_LANGUAGES = [
  { code: 'en-US', label: 'English' },
  { code: 'zh-CN', label: '简体中文' },
  { code: 'zh-TW', label: '繁體中文' },
  { code: 'ja-JP', label: '日本語' },
  { code: 'ko-KR', label: '한국어' },
  { code: 'de-DE', label: 'Deutsch' },
  { code: 'fr-FR', label: 'Français' },
  { code: 'es-ES', label: 'Español' },
  { code: 'pt-BR', label: 'Português (Brasil)' },
  { code: 'ru-RU', label: 'Русский' },
] as const

export type AppLanguage = (typeof SUPPORTED_LANGUAGES)[number]['code']

export type LanguagePreference = AppLanguage | 'system'

export const DEFAULT_LANGUAGE: AppLanguage = 'en-US'

export function isSupportedLanguage(value: unknown): value is AppLanguage {
  return SUPPORTED_LANGUAGES.some((language) => language.code === value)
}

export function isLanguagePreference(value: unknown): value is LanguagePreference {
  return value === 'system' || isSupportedLanguage(value)
}

// Traditional Chinese is used in Taiwan, Hong Kong and Macau; every other
// Chinese variant falls back to Simplified Chinese.
const TRADITIONAL_CHINESE = /^zh-(tw|hk|mo|hant)\b/

/**
 * Maps a BCP 47 tag such as `navigator.language` to the closest supported
 * language: an exact match first, then a match on the primary subtag.
 */
export function matchLanguage(tag: string | undefined | null): AppLanguage | undefined {
  if (!tag) {
    return undefined
  }

  const normalized = tag.trim().replace(/_/g, '-').toLowerCase()
  const exact = SUPPORTED_LANGUAGES.find((language) => language.code.toLowerCase() === normalized)
  if (exact) {
    return exact.code
  }

  if (normalized.startsWith('zh')) {
    return TRADITIONAL_CHINESE.test(normalized) ? 'zh-TW' : 'zh-CN'
  }

  const primary = normalized.split('-')[0]
  return SUPPORTED_LANGUAGES.find((language) => language.code.toLowerCase().split('-')[0] === primary)?.code
}

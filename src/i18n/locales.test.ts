import { describe, expect, it } from 'vitest'
import { resources } from './index'
import { SUPPORTED_LANGUAGES, matchLanguage } from './languages'

type Messages = { [key: string]: string | Messages }

function flatten(messages: Messages, prefix = ''): Record<string, string> {
  return Object.entries(messages).reduce<Record<string, string>>((result, [key, value]) => {
    const path = `${prefix}${key}`
    if (typeof value === 'string') {
      result[path] = value
    } else {
      Object.assign(result, flatten(value, `${path}.`))
    }
    return result
  }, {})
}

function placeholders(value: string): string[] {
  return [...value.matchAll(/\{\{\s*(\w+)\s*\}\}/g)].map((match) => match[1]).sort()
}

const reference = flatten(resources['en-US'].translation)

describe('locales', () => {
  it('registers a resource bundle for every supported language', () => {
    expect(Object.keys(resources).sort()).toEqual(SUPPORTED_LANGUAGES.map((language) => language.code).sort())
  })

  describe.each(SUPPORTED_LANGUAGES.map((language) => language.code))('%s', (code) => {
    const messages = flatten(resources[code].translation)

    it('has exactly the same keys as en-US', () => {
      expect(Object.keys(messages).sort()).toEqual(Object.keys(reference).sort())
    })

    it('keeps the same interpolation placeholders as en-US', () => {
      for (const [key, value] of Object.entries(reference)) {
        expect({ key, placeholders: placeholders(messages[key]) }).toEqual({ key, placeholders: placeholders(value) })
      }
    })

    it('translates every non-empty message', () => {
      for (const [key, value] of Object.entries(reference)) {
        if (value !== '') {
          expect({ key, empty: messages[key] === '' }).toEqual({ key, empty: false })
        }
      }
    })
  })
})

describe('matchLanguage', () => {
  it.each([
    ['en-US', 'en-US'],
    ['en-GB', 'en-US'],
    ['zh', 'zh-CN'],
    ['zh-CN', 'zh-CN'],
    ['zh-SG', 'zh-CN'],
    ['zh-Hans-CN', 'zh-CN'],
    ['zh-TW', 'zh-TW'],
    ['zh-HK', 'zh-TW'],
    ['zh-Hant', 'zh-TW'],
    ['zh_TW', 'zh-TW'],
    ['ja', 'ja-JP'],
    ['ko-KR', 'ko-KR'],
    ['de-AT', 'de-DE'],
    ['fr-CA', 'fr-FR'],
    ['es-MX', 'es-ES'],
    ['it-CH', 'it-IT'],
    ['pt-PT', 'pt-BR'],
    ['ru', 'ru-RU'],
  ])('maps %s to %s', (tag, expected) => {
    expect(matchLanguage(tag)).toBe(expected)
  })

  it('returns undefined for unsupported or empty tags', () => {
    expect(matchLanguage('nl-NL')).toBeUndefined()
    expect(matchLanguage('')).toBeUndefined()
    expect(matchLanguage(undefined)).toBeUndefined()
  })
})

import { useTranslation } from 'react-i18next'
import { SUPPORTED_LANGUAGES, type LanguagePreference } from '../i18n/languages'

interface LanguageMenuProps {
  language: LanguagePreference
  onChange: (language: LanguagePreference) => void
  selectClassName?: string
}

export function LanguageMenu({ language, onChange, selectClassName }: LanguageMenuProps) {
  const { t } = useTranslation()

  return (
    <label className="settings-select-wrap" aria-label={t('language')}>
      <select
        className={selectClassName ? `settings-select ${selectClassName}` : 'settings-select'}
        value={language}
        onChange={(event) => onChange(event.target.value as LanguagePreference)}
      >
        <option value="system">{t('languageSystem')}</option>
        {SUPPORTED_LANGUAGES.map(({ code, label }) => (
          <option key={code} value={code} lang={code}>
            {label}
          </option>
        ))}
      </select>
    </label>
  )
}

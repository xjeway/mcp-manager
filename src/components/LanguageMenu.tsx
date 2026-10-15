import { useTranslation } from 'react-i18next'
import { SUPPORTED_LANGUAGES, type AppLanguage } from '../i18n/languages'

interface LanguageMenuProps {
  language: string
  onChange: (language: AppLanguage) => void
  selectClassName?: string
}

export function LanguageMenu({ language, onChange, selectClassName }: LanguageMenuProps) {
  const { t } = useTranslation()

  return (
    <label className="settings-select-wrap" aria-label={t('language')}>
      <select
        className={selectClassName ? `settings-select ${selectClassName}` : 'settings-select'}
        value={language}
        onChange={(event) => onChange(event.target.value as AppLanguage)}
      >
        {SUPPORTED_LANGUAGES.map(({ code, label }) => (
          <option key={code} value={code} lang={code}>
            {label}
          </option>
        ))}
      </select>
    </label>
  )
}

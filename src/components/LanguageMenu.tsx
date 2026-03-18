import { useTranslation } from 'react-i18next'

interface LanguageMenuProps {
  language: string
  onChange: (language: 'zh-CN' | 'en-US') => void
}

export function LanguageMenu({ language, onChange }: LanguageMenuProps) {
  const { t } = useTranslation()

  return (
    <label className="settings-select-wrap" aria-label={t('language')}>
      <select
        className="settings-select"
        value={language}
        onChange={(event) => onChange(event.target.value as 'zh-CN' | 'en-US')}
      >
        <option value="zh-CN">中文</option>
        <option value="en-US">English</option>
      </select>
    </label>
  )
}

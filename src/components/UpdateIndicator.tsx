import { Download, LoaderCircle, RotateCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { UpdateState } from '../services/updater'
import { Tooltip } from './Tooltip'

interface UpdateIndicatorProps {
  state: Pick<UpdateState, 'phase' | 'percent' | 'targetVersion'>
  onInstall: () => void
  onShowProgress: () => void
}

/**
 * Keeps an update the user sent to the background visible in the header until
 * it is installed, so a missed toast never leaves it unnoticed.
 */
export function UpdateIndicator({ state, onInstall, onShowProgress }: UpdateIndicatorProps) {
  const { t } = useTranslation()

  if (state.phase === 'downloaded') {
    return (
      <Tooltip content={t('updateReadyBackground', { version: state.targetVersion ?? '' })}>
        <button type="button" className="update-indicator update-indicator-ready" onClick={onInstall}>
          <RotateCw size={14} />
          <span>{t('updateRestartToUpdate')}</span>
        </button>
      </Tooltip>
    )
  }

  if (state.phase === 'installing') {
    return (
      <button type="button" className="update-indicator" disabled>
        <LoaderCircle size={14} className="spin" />
        <span>{t('updateWindowInstallingTitle')}</span>
      </button>
    )
  }

  if (state.phase === 'downloading') {
    return (
      <button type="button" className="update-indicator" onClick={onShowProgress}>
        <Download size={14} />
        <span>
          {t('updateWindowDownloadingTitle')}
          {state.percent === null ? '' : ` ${state.percent}%`}
        </span>
      </button>
    )
  }

  return null
}

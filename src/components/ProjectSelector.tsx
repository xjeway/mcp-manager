import { FolderOpen } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { WorkspaceContext } from '../types/config'

interface Props {
  workspace: WorkspaceContext
  recent: string[]
  disabled: boolean
  problem: string | null
  onSelect: (root?: string) => void
}

/** Selection changes only context; importing and writing remain explicit actions. */
export function ProjectSelector({ workspace, recent, disabled, problem, onSelect }: Props) {
  const { t } = useTranslation()
  const paths = [...new Set([workspace.root, ...recent].filter(Boolean))]
  const basename = (path: string) => path.split(/[\\/]/).filter(Boolean).pop() ?? path
  const label = (path: string) => paths.filter((other) => basename(other) === basename(path)).length > 1 ? path : basename(path)
  return (
    <div className="project-selector" title={problem ?? workspace.root}>
      <FolderOpen size={14} aria-hidden="true" />
      <select
        className="settings-select project-select"
        aria-label={t('projectSelection')}
        disabled={disabled}
        value={workspace.root}
        onChange={(event) => onSelect(event.target.value === '__choose_project__' ? undefined : event.target.value)}
      >
        <option value="">{t('projectComputer')}</option>
        {paths.length > 0 ? <optgroup label={t('projectRecent')}>
          {paths.map((path) => <option key={path} value={path}>{label(path)}</option>)}
        </optgroup> : null}
        <option value="__choose_project__">{t('projectChoose')}</option>
      </select>
      {problem ? <span className="project-selection-error" role="alert">{t('projectUnavailable')}</span> : null}
    </div>
  )
}

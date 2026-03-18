import type { ReactNode } from 'react'
import type { SupportedApp } from '../types/config'

function ClientLogo({ src, alt }: { alt: string; src: string }) {
  return <img src={src} alt={alt} className="client-logo-image" loading="eager" decoding="async" />
}

export interface ClientMeta {
  accent: string
  icon: ReactNode
  id: SupportedApp
  label: string
}

export const CLIENTS: ClientMeta[] = [
  {
    id: 'vscode',
    label: 'VS Code',
    accent: 'client-vscode',
    icon: <ClientLogo src="/logos/apps/vscode.svg" alt="VS Code" />,
  },
  {
    id: 'cursor',
    label: 'Cursor',
    accent: 'client-cursor',
    icon: <ClientLogo src="/logos/apps/cursor.ico" alt="Cursor" />,
  },
  {
    id: 'claudeCode',
    label: 'Claude Code',
    accent: 'client-claude',
    icon: <ClientLogo src="/logos/apps/claude.svg" alt="Claude Code" />,
  },
  {
    id: 'codex',
    label: 'Codex',
    accent: 'client-codex',
    icon: <ClientLogo src="/logos/apps/openai.svg" alt="Codex" />,
  },
]

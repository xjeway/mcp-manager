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
    icon: <ClientLogo src="/logos/apps/cursor.svg" alt="Cursor" />,
  },
  {
    id: 'claudeCode',
    label: 'Claude Code',
    accent: 'client-claude',
    icon: <ClientLogo src="/logos/apps/claudecode.svg" alt="Claude Code" />,
  },
  {
    id: 'claudeDesktop',
    label: 'Claude Desktop',
    accent: 'client-claude',
    icon: <ClientLogo src="/logos/apps/claude.svg" alt="Claude Desktop" />,
  },
  {
    id: 'codex',
    label: 'Codex',
    accent: 'client-codex',
    icon: <ClientLogo src="/logos/apps/codex.svg" alt="Codex" />,
  },
  {
    id: 'openCode',
    label: 'OpenCode',
    accent: 'client-cursor',
    icon: <ClientLogo src="/logos/apps/opencode.svg" alt="OpenCode" />,
  },
  {
    id: 'githubCopilot',
    label: 'GitHub Copilot',
    accent: 'client-vscode',
    icon: <ClientLogo src="/logos/apps/githubcopilot.svg" alt="GitHub Copilot" />,
  },
  {
    id: 'geminiCli',
    label: 'Gemini CLI',
    accent: 'client-codex',
    icon: <ClientLogo src="/logos/apps/gemini.svg" alt="Gemini CLI" />,
  },
  {
    id: 'antigravity',
    label: 'Antigravity',
    accent: 'client-cursor',
    icon: <ClientLogo src="/logos/apps/antigravity.svg" alt="Antigravity" />,
  },
  {
    id: 'iFlow',
    label: 'iFlow',
    accent: 'client-vscode',
    icon: <ClientLogo src="/logos/apps/iflow.svg" alt="iFlow" />,
  },
]

export const PLANNED_CLIENTS = [
  'Cline',
  'RooCode',
  'Kilo Code',
  'Amazon Q',
  'Qoder',
  'Auggie CLI',
  'Qwen Code',
  'CodeBuddy',
  'CoStrict',
  'Crush',
  'Factory Droid',
] as const

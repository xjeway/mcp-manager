import appLogo from '../../src-tauri/icons/app-icon-rounded.png'

interface AppLogoProps {
  className?: string
  alt?: string
}

export function AppLogo({ className = '', alt = 'MCP Manager logo' }: AppLogoProps) {
  const classes = ['app-logo', className].filter(Boolean).join(' ')

  return <img src={appLogo} alt={alt} className={classes} loading="eager" decoding="async" />
}

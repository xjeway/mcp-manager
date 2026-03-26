import { invoke } from '@tauri-apps/api/core'
import { isDesktopRuntime } from './runtime'

type DialogKind = 'info' | 'warning' | 'error'

interface ConfirmDialogOptions {
  cancelLabel?: string
  kind?: DialogKind
  message: string
  okLabel?: string
  title?: string
}

async function confirmWithBrowserFallback(message: string): Promise<boolean> {
  try {
    return Boolean(await Promise.resolve(window.confirm(message) as unknown as boolean | Promise<boolean>))
  } catch {
    return false
  }
}

export async function confirmDialog({
  cancelLabel,
  kind,
  message,
  okLabel,
  title,
}: ConfirmDialogOptions): Promise<boolean> {
  if (!isDesktopRuntime()) {
    return confirmWithBrowserFallback(message)
  }

  try {
    return await invoke<boolean>('plugin:dialog|confirm', {
      cancelButtonLabel: cancelLabel,
      kind,
      message,
      okButtonLabel: okLabel,
      title,
    })
  } catch {
    return confirmWithBrowserFallback(message)
  }
}

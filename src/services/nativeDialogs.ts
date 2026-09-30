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

  // tauri-plugin-dialog dropped its `confirm` command; `message` with two buttons replaces it.
  const okText = okLabel ?? 'OK'
  const cancelText = cancelLabel ?? 'Cancel'

  try {
    const result = await invoke<unknown>('plugin:dialog|message', {
      buttons: { OkCancelCustom: [okText, cancelText] },
      kind,
      message,
      title,
    })
    return isAccepted(result, okText)
  } catch {
    return confirmWithBrowserFallback(message)
  }
}

// The plugin answers "Ok"/"Yes", or the pressed button's label (bare or as `{ Custom: label }`).
function isAccepted(result: unknown, okText: string): boolean {
  const label =
    typeof result === 'object' && result !== null && 'Custom' in result
      ? (result as { Custom: unknown }).Custom
      : result
  return label === true || label === 'Ok' || label === 'Yes' || label === okText
}

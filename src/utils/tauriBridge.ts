import type { UnlistenFn } from '@tauri-apps/api/event'

export async function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(cmd, args)
}

export async function tauriListen<T>(
  event: string,
  handler: (payload: T) => void
): Promise<UnlistenFn> {
  const { listen } = await import('@tauri-apps/api/event')
  return listen<T>(event, (e) => handler(e.payload))
}

export async function tauriReadTextFile(path: string): Promise<string> {
  const { readTextFile } = await import('@tauri-apps/plugin-fs')
  return readTextFile(path)
}

export async function tauriWriteTextFile(path: string, content: string): Promise<void> {
  const { writeTextFile } = await import('@tauri-apps/plugin-fs')
  return writeTextFile(path, content)
}

export async function tauriOpenFile(options?: Parameters<
  Awaited<typeof import('@tauri-apps/plugin-dialog')>['open']
>[0]) {
  const { open } = await import('@tauri-apps/plugin-dialog')
  return open(options)
}

export async function tauriSaveFile(options?: Parameters<
  Awaited<typeof import('@tauri-apps/plugin-dialog')>['save']
>[0]) {
  const { save } = await import('@tauri-apps/plugin-dialog')
  return save(options)
}

export async function tauriAsk(
  message: string,
  options?: Parameters<Awaited<typeof import('@tauri-apps/plugin-dialog')>['ask']>[1]
) {
  const { ask } = await import('@tauri-apps/plugin-dialog')
  return ask(message, options)
}

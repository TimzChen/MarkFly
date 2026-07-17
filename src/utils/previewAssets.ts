import { convertFileSrc } from '@tauri-apps/api/core'
import { tauriInvoke } from './tauriBridge'

export function isDiskFilePath(filePath: string): boolean {
  return /[/\\]/.test(filePath) || /^[A-Za-z]:/.test(filePath)
}

export function isExternalAssetUrl(src: string): boolean {
  const trimmed = src.trim()
  return (
    /^(https?:|data:|blob:|asset:|javascript:|mailto:|#)/i.test(trimmed) ||
    trimmed.startsWith('//')
  )
}

/** 将 Markdown 中的相对资源路径解析为绝对磁盘路径 */
export function resolveMarkdownAssetPath(filePath: string, src: string): string {
  const trimmed = src.trim()
  if (!trimmed || isExternalAssetUrl(trimmed)) return trimmed
  if (/^[A-Za-z]:[/\\]/.test(trimmed) || trimmed.startsWith('\\\\')) return trimmed

  const normalizedFile = filePath.replace(/\\/g, '/')
  const dir = normalizedFile.includes('/')
    ? normalizedFile.slice(0, normalizedFile.lastIndexOf('/') + 1)
    : ''

  let absolute = decodeURIComponent(new URL(trimmed, `file:///${dir}`).pathname)
  if (/^\/[A-Za-z]:/.test(absolute)) {
    absolute = absolute.slice(1)
  }
  return absolute
}

const allowedAssets = new Set<string>()

export async function ensurePreviewAssetAllowed(resolvedPath: string): Promise<void> {
  if (!resolvedPath || allowedAssets.has(resolvedPath)) return
  allowedAssets.add(resolvedPath)
  try {
    await tauriInvoke('allow_preview_asset', { path: resolvedPath })
  } catch {
    // 非 Tauri 环境或文件不存在时忽略
  }
}

export function toPreviewAssetUrl(filePath: string, src: string): string {
  if (!filePath || !isDiskFilePath(filePath)) return src
  const resolved = resolveMarkdownAssetPath(filePath, src)
  if (isExternalAssetUrl(resolved)) return resolved
  try {
    return convertFileSrc(resolved)
  } catch {
    return resolved
  }
}

export function extractMarkdownImageSources(content: string): string[] {
  const sources = new Set<string>()
  const imagePattern = /!\[[^\]]*\]\(([^)\s]+(?:\s+"[^"]*")?)\)/g
  let match: RegExpExecArray | null
  while ((match = imagePattern.exec(content)) !== null) {
    sources.add(match[1].trim().split(/\s+/)[0])
  }
  return [...sources]
}

export async function allowMarkdownAssets(filePath: string, content: string): Promise<void> {
  if (!filePath || !isDiskFilePath(filePath)) return
  const tasks = extractMarkdownImageSources(content)
    .filter((src) => !isExternalAssetUrl(src))
    .map((src) => ensurePreviewAssetAllowed(resolveMarkdownAssetPath(filePath, src)))
  await Promise.all(tasks)
}

export function rewritePreviewAssetUrls(html: string, filePath: string): string {
  if (!filePath || !isDiskFilePath(filePath)) return html
  return html.replace(
    /(<img\b[^>]*?\bsrc=)(["'])([^"']+)\2/gi,
    (_match, prefix, quote, src) => `${prefix}${quote}${toPreviewAssetUrl(filePath, src)}${quote}`
  )
}

export function fixPreviewImages(container: HTMLElement | null, filePath: string): void {
  if (!container || !filePath || !isDiskFilePath(filePath)) return
  container.querySelectorAll('img[src]').forEach((node) => {
    const src = node.getAttribute('src')
    if (!src) return
    const next = toPreviewAssetUrl(filePath, src)
    if (next !== src) node.setAttribute('src', next)
  })
}

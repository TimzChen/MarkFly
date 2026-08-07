import { resolveMarkdownAssetPath } from './previewAssets'
import {
  fromMarkflyFileHref,
  isExternalLinkUrl,
  isSystemBrowserUrl,
  MARKFLY_FILE_PROTOCOL,
  normalizeExternalUrl,
} from './previewLinkConfig'

export type PreviewLinkNavigationOptions = {
  getCurrentFilePath: () => string
  onOpenFile: (path: string) => void | Promise<void>
  /** http(s) / mailto 外链：系统浏览器打开 */
  onOpenExternal?: (url: string) => void | Promise<void>
  /** 本地路径解析失败时的提示（可选） */
  onResolveError?: (href: string) => void
  /** 仅处理落在这些选择器内的链接（默认预览区） */
  previewSelector?: string
}

type LinkBinding = { detach: () => void }
const bindingsByContainer = new WeakMap<HTMLElement, LinkBinding>()

const DEFAULT_PREVIEW_SELECTOR = '.markfly-preview-viewer, .bytemd-preview'

function splitHref(href: string): { pathPart: string; hash: string } {
  const hashIndex = href.indexOf('#')
  if (hashIndex === -1) return { pathPart: href, hash: '' }
  return { pathPart: href.slice(0, hashIndex), hash: href.slice(hashIndex + 1) }
}

function resolveLocalLinkPath(pathPart: string, currentFilePath: string): string | null {
  const trimmed = pathPart.trim()
  if (!trimmed) return null

  if (trimmed.toLowerCase().startsWith(`${MARKFLY_FILE_PROTOCOL}:`)) {
    return fromMarkflyFileHref(trimmed)
  }

  if (/^file:\/\//i.test(trimmed)) {
    try {
      const parsed = new URL(trimmed)
      let path = decodeURIComponent(parsed.pathname)
      if (/^\/[A-Za-z]:/.test(path)) path = path.slice(1)
      return path.replace(/\//g, '\\')
    } catch {
      return null
    }
  }

  if (isExternalLinkUrl(trimmed)) return null
  if (!currentFilePath) return null

  return resolveMarkdownAssetPath(currentFilePath, trimmed)
}

function scrollToAnchor(previewRoot: HTMLElement, hash: string) {
  if (!hash) return
  const target =
    previewRoot.querySelector(`#${CSS.escape(hash)}`) ??
    previewRoot.querySelector(`[id="${hash}"]`) ??
    previewRoot.querySelector(`#user-content-${CSS.escape(hash)}`)
  if (!target) return

  const scrollRoot =
    (previewRoot.closest('.markfly-preview-viewer, .bytemd-preview') as HTMLElement | null) ??
    previewRoot
  const rootRect = scrollRoot.getBoundingClientRect()
  const targetRect = (target as HTMLElement).getBoundingClientRect()
  const nextTop = targetRect.top - rootRect.top + scrollRoot.scrollTop
  const maxScroll = Math.max(0, scrollRoot.scrollHeight - scrollRoot.clientHeight)
  scrollRoot.scrollTo({ top: Math.min(Math.max(0, nextTop), maxScroll), behavior: 'smooth' })
}

/**
 * 在稳定容器上做捕获阶段委托（不依赖 .markdown-body 是否被重建）。
 * 用于仅预览根节点 / editor-content / bytemd-preview。
 */
export function bindPreviewLinkNavigation(
  container: HTMLElement | null,
  options: PreviewLinkNavigationOptions
): void {
  if (!container) return

  bindingsByContainer.get(container)?.detach()

  const previewSelector = options.previewSelector ?? DEFAULT_PREVIEW_SELECTOR

  const handler = async (event: MouseEvent) => {
    if (event.defaultPrevented || event.button !== 0) return
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return

    const target = event.target as Element | null
    if (!target) return

    const previewRoot = target.closest(previewSelector) as HTMLElement | null
    if (!previewRoot || !container.contains(previewRoot)) return

    const anchor = target.closest('a')
    if (!anchor || !previewRoot.contains(anchor)) return

    const href = anchor.getAttribute('href')
    if (!href) return

    const trimmed = href.trim()

    // javascript: 等危险协议：拦截但不打开
    if (/^javascript:/i.test(trimmed)) {
      event.preventDefault()
      event.stopPropagation()
      return
    }

    // http(s) / mailto / 协议相对 → 系统浏览器
    if (isSystemBrowserUrl(trimmed)) {
      event.preventDefault()
      event.stopPropagation()
      const externalUrl = normalizeExternalUrl(trimmed)
      if (options.onOpenExternal) {
        await options.onOpenExternal(externalUrl)
      } else {
        // 未接线时兜底，避免 preventDefault 后点击无响应
        window.open(externalUrl, '_blank', 'noopener,noreferrer')
      }
      return
    }

    // 其余外链形态（若有）仍不拦截；# 与本地路径继续处理
    if (isExternalLinkUrl(trimmed) && !trimmed.startsWith('#')) return

    event.preventDefault()
    event.stopPropagation()

    const { pathPart, hash } = splitHref(trimmed)

    if (!pathPart || pathPart === '.') {
      scrollToAnchor(previewRoot, hash)
      return
    }

    const resolved = resolveLocalLinkPath(pathPart, options.getCurrentFilePath())
    if (!resolved) {
      console.warn('无法解析本地链接:', trimmed)
      options.onResolveError?.(trimmed)
      return
    }

    await options.onOpenFile(resolved)
  }

  // 捕获阶段：避免 WebView 先导航自定义协议导致点击失效
  container.addEventListener('click', handler, true)
  bindingsByContainer.set(container, {
    detach: () => container.removeEventListener('click', handler, true),
  })
}

export function unbindPreviewLinkNavigation(container: HTMLElement | null): void {
  if (!container) return
  bindingsByContainer.get(container)?.detach()
  bindingsByContainer.delete(container)
}

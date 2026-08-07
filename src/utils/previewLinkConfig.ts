import type { BytemdPlugin } from 'bytemd'
import type { Schema } from 'hast-util-sanitize'
import type { Link } from 'mdast'
import { visit } from 'unist-util-visit'

/** 自定义协议：sanitize 白名单内保留本地路径 href */
export const MARKFLY_FILE_PROTOCOL = 'markfly-file'

export function isExternalLinkUrl(url: string): boolean {
  const trimmed = url.trim()
  return (
    /^(https?:|mailto:|javascript:|#)/i.test(trimmed) ||
    trimmed.startsWith('//')
  )
}

/** 应用内应交给系统浏览器打开的外链（不含 # 锚点 / javascript） */
export function isSystemBrowserUrl(url: string): boolean {
  const trimmed = url.trim()
  if (!trimmed || trimmed.startsWith('#') || /^javascript:/i.test(trimmed)) {
    return false
  }
  return /^(https?:|mailto:)/i.test(trimmed) || trimmed.startsWith('//')
}

/** 规范化协议相对 URL：//example.com → https://example.com */
export function normalizeExternalUrl(url: string): string {
  const trimmed = url.trim()
  if (trimmed.startsWith('//')) return `https:${trimmed}`
  return trimmed
}

function normalizeDiskPath(path: string): string {
  return path.replace(/\\/g, '/').replace(/^([A-Za-z]):/, (_, drive) => `${drive.toLowerCase()}:`)
}

/** 磁盘路径 → sanitize 安全的 markfly-file href */
export function toMarkflyFileHref(diskPath: string): string {
  return `${MARKFLY_FILE_PROTOCOL}:///${normalizeDiskPath(diskPath)}`
}

/** markfly-file href → 磁盘路径 */
export function fromMarkflyFileHref(href: string): string {
  const trimmed = href.trim()
  const lower = trimmed.toLowerCase()
  const protocol = `${MARKFLY_FILE_PROTOCOL}:`
  if (!lower.startsWith(protocol)) return trimmed

  // 兼容 markfly-file:///d:/a.md 与 markfly-file://d:/a.md
  let rest = trimmed.slice(protocol.length).replace(/^\/\//, '')
  rest = rest.replace(/^\/+/, '')

  let path = decodeURIComponent(rest)
  // 个别渲染会把反斜杠编成 %5C，解码后已是 \
  if (/^[A-Za-z]:/.test(path)) {
    path = path.replace(/\//g, '\\')
  } else if (path.startsWith('\\\\') || path.startsWith('//')) {
    path = path.replace(/\//g, '\\')
  }
  return path
}

function fileUrlToDiskPath(fileUrl: string): string | null {
  try {
    const parsed = new URL(fileUrl)
    let path = decodeURIComponent(parsed.pathname)
    if (/^\/[A-Za-z]:/.test(path)) path = path.slice(1)
    return path
  } catch {
    return null
  }
}

/** remark 阶段：绝对 / file:// 本地路径改写为 markfly-file，避免 sanitize 剥掉 href */
function normalizeLinkToMarkflyFile(url: string): string | null {
  const trimmed = url.trim()
  if (!trimmed || isExternalLinkUrl(trimmed)) return null
  if (trimmed.toLowerCase().startsWith(`${MARKFLY_FILE_PROTOCOL}:`)) return trimmed

  if (/^file:\/\//i.test(trimmed)) {
    const path = fileUrlToDiskPath(trimmed)
    return path ? toMarkflyFileHref(path) : null
  }

  if (/^[A-Za-z]:[/\\]/.test(trimmed) || trimmed.startsWith('\\\\')) {
    return toMarkflyFileHref(trimmed)
  }

  return null
}

export function createLocalFileLinksPlugin(): BytemdPlugin {
  return {
    remark: (processor) =>
      processor.use(() => (tree) => {
        visit(tree, 'link', (node: Link) => {
          if (!node.url) return
          const next = normalizeLinkToMarkflyFile(node.url)
          if (next) node.url = next
        })
      }),
  }
}

export function createPreviewSanitizeSchema(baseSchema: Schema): Schema {
  return {
    ...baseSchema,
    protocols: {
      ...baseSchema.protocols,
      href: [...(baseSchema.protocols?.href ?? []), MARKFLY_FILE_PROTOCOL],
    },
  }
}

export const previewSanitize = (schema: Schema) => createPreviewSanitizeSchema(schema)

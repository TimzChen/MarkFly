/**
 * 飞书 / 部分导出习惯会给普通标点加反斜杠（看板\-、1\.、\(Coda\)）。
 * 这些解开后不会变成强调、链接或代码；\ * \_ \` 等会改变语法，保持原样。
 */
const SAFE_ESCAPED_PUNCT = /\\([-.()+])/g
const FENCE_RE = /^(\s*)(```|~~~)/

export function unescapeSafeMarkdownPunctuation(text: string): string {
  return text.replace(SAFE_ESCAPED_PUNCT, '$1')
}

const collapseExtraEmphasis = (text: string): string => {
  let prev = ''
  let next = text
  while (next !== prev) {
    prev = next
    next = next.replace(/\*{4}/g, '**')
  }
  return next
}

const transformUnfencedLine = (line: string): string => {
  const parts = line.split(/(`+)/)
  let ticks = ''
  return parts
    .map((part) => {
      if (!ticks && /^(`+)$/.test(part)) {
        ticks = part
        return part
      }
      if (ticks && part === ticks) {
        ticks = ''
        return part
      }
      if (ticks) return part
      return collapseExtraEmphasis(unescapeSafeMarkdownPunctuation(part))
    })
    .join('')
}

/** 预览和目录去掉导出残留转义，不碰代码块，也不改磁盘原文 */
export function normalizeImportedMarkdown(markdown: string): string {
  if (!markdown) return markdown
  const lines = markdown.replace(/\r\n/g, '\n').split('\n')
  let fence = ''
  return lines
    .map((line) => {
      const fenceMatch = line.match(FENCE_RE)
      if (fenceMatch) {
        const mark = fenceMatch[2]
        if (!fence) fence = mark
        else if (mark === fence) fence = ''
        return line
      }
      if (fence) return line
      return transformUnfencedLine(line)
    })
    .join('\n')
}

export function looksLikeInlineTex(inner: string): boolean {
  const text = inner.trim()
  if (!text) return false
  if (/\\[a-zA-Z]+/.test(text)) return true
  if (/[=^_]/.test(text)) return true
  if (!/[+\-*/]/.test(text)) return false
  if (/[A-Za-z]/.test(text)) return true
  return /\d/.test(text) && text.length < 80
}

/** 飞书文案里的 \(Coda\) 不是公式；只有内容像 TeX 才加载数学插件 */
export function hasRealInlineMath(content: string): boolean {
  const pattern = /\\\(([\s\S]+?)\\\)/g
  let match: RegExpExecArray | null
  while ((match = pattern.exec(content))) {
    if (looksLikeInlineTex(match[1])) return true
  }
  return false
}

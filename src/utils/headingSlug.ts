import { unescapeSafeMarkdownPunctuation } from './markdownEscapes'

/**
 * 标题锚点按 VS Code Markdown 预览规则生成：
 * 去首尾空白、英文小写、空白变成 `-`、去掉中英文标点。
 * 中文标题（如「状态、补丁」）结果通常也与 GitHub 一致。
 */
const HEADING_PUNCT_RE =
  /[\][!'#$%&()*+,./:;<=>?@\\^_{|}~`。，、；：？！…—·ˉ¨‘’“”々～‖∶＂＇｀｜〃〔〕〈〉《》「」『』．〖〗【】（）［］｛｝]/g

export function slugifyHeading(text: string): string {
  return text
    .trim()
    .toLowerCase()
    .replace(/\s+/g, '-')
    .replace(HEADING_PUNCT_RE, '')
    .replace(/^-+/, '')
    .replace(/-+$/, '')
}

/** 去掉标题里的 Markdown 标记，只留纯文本再生成锚点 */
export function headingPlainText(raw: string): string {
  return unescapeSafeMarkdownPunctuation(raw)
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    .replace(/__([^_]+)__/g, '$1')
    .replace(/\*([^*]+)\*/g, '$1')
    .replace(/_([^_]+)_/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/\*{2,}/g, '')
    .trim()
}

/**
 * 同一篇文档内重复标题加 -1、-2。
 * 已发出的 id 也登记，避免 Foo / Foo-1 / Foo 撞成两个 foo-1。
 */
export function createHeadingSlugger(): (text: string) => string {
  const used = new Set<string>()
  return (text: string) => {
    const base = slugifyHeading(text) || 'section'
    let slug = base
    let n = 1
    while (used.has(slug)) {
      slug = `${base}-${n}`
      n += 1
    }
    used.add(slug)
    return slug
  }
}

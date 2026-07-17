/** 去掉 YAML frontmatter，避免影响预览解析 */
export const stripFrontmatter = (markdown: string): string => {
  if (!/^\s*---\r?\n/.test(markdown)) {
    return markdown
  }
  const match = markdown.match(/^\s*---\r?\n[\s\S]*?\r?\n---\r?\n?/)
  if (!match) {
    return markdown
  }
  return markdown.slice(match[0].length)
}

/** 是否必须用 ByteMD + GFM 完整管道（表格、frontmatter 等） */
export const needsFullPreview = (content: string): boolean => {
  if (!content) return false
  if (/^\s*---\r?\n[\s\S]*?\r?\n---/.test(content)) return true
  if (/^\|.+\|\s*$/m.test(content)) return true
  if (/^>\s/m.test(content)) return true
  if (/```/.test(content)) return true
  return false
}

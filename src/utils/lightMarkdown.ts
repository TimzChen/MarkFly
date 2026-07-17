const escapeHtml = (text: string): string =>
  text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')

const renderInline = (text: string): string =>
  escapeHtml(text)
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>')

/** 零依赖轻量 Markdown 渲染，与 boot 预览层逻辑一致 */
export const lightMarkdownToHtml = (markdown: string): string => {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n')
  const html: string[] = []
  let inCode = false
  let codeLines: string[] = []
  let listType = ''

  const flushList = () => {
    if (listType) {
      html.push(listType === 'ol' ? '</ol>' : '</ul>')
      listType = ''
    }
  }

  for (const line of lines) {
    if (line.startsWith('```')) {
      flushList()
      if (!inCode) {
        inCode = true
        codeLines = []
      } else {
        html.push(`<pre><code>${escapeHtml(codeLines.join('\n'))}</code></pre>`)
        inCode = false
      }
      continue
    }

    if (inCode) {
      codeLines.push(line)
      continue
    }

    if (/^#{1,6}\s/.test(line)) {
      flushList()
      const level = line.match(/^#+/)![0].length
      html.push(
        `<h${level}>${renderInline(line.replace(/^#{1,6}\s/, ''))}</h${level}>`
      )
      continue
    }

    if (/^!\[([^\]]*)\]\(([^)]+)\)\s*$/.test(line)) {
      flushList()
      const imageMatch = line.match(/^!\[([^\]]*)\]\(([^)]+)\)\s*$/)!
      html.push(`<p><img alt="${escapeHtml(imageMatch[1])}" src="${escapeHtml(imageMatch[2])}"></p>`)
      continue
    }

    if (/^(\*\s|-\s|\d+\.\s)/.test(line)) {
      const ordered = /^\d+\.\s/.test(line)
      const nextType = ordered ? 'ol' : 'ul'
      if (listType !== nextType) {
        flushList()
        listType = nextType
        html.push(nextType === 'ol' ? '<ol>' : '<ul>')
      }
      html.push(
        `<li>${renderInline(line.replace(/^(\*\s|-\s|\d+\.\s)/, ''))}</li>`
      )
      continue
    }

    if (!line.trim()) {
      flushList()
      continue
    }

    flushList()
    html.push(`<p>${renderInline(line)}</p>`)
  }

  flushList()
  return html.join('')
}

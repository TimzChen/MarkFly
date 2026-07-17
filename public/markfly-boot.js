(function () {
  function escapeHtml(text) {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
  }

  function renderInline(text) {
    return escapeHtml(text)
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
      .replace(/\*([^*]+)\*/g, '<em>$1</em>')
      .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>')
  }

  function stripFrontmatter(markdown) {
    if (!/^\s*---\r?\n/.test(markdown)) return markdown
    const match = markdown.match(/^\s*---\r?\n[\s\S]*?\r?\n---\r?\n?/)
    if (!match) return markdown
    return markdown.slice(match[0].length)
  }

  function renderMarkdown(markdown) {
    const lines = stripFrontmatter(markdown).replace(/\r\n/g, '\n').split('\n')
    const html = []
    let inCode = false
    let codeLines = []
    let listType = ''
    let tablePlaceholderShown = false

    const flushList = () => {
      if (listType) {
        html.push(listType === 'ol' ? '</ol>' : '</ul>')
        listType = ''
      }
    }

    const isTableLine = (line) => {
      const trimmed = line.trim()
      return trimmed.startsWith('|') && trimmed.endsWith('|')
    }

    for (const line of lines) {
      if (isTableLine(line)) {
        flushList()
        if (!tablePlaceholderShown) {
          html.push('<p class="markfly-boot-truncated"><em>表格正在加载完整预览…</em></p>')
          tablePlaceholderShown = true
        }
        continue
      }
      if (line.startsWith('```')) {
        flushList()
        if (!inCode) {
          inCode = true
          codeLines = []
        } else {
          html.push('<pre><code>' + escapeHtml(codeLines.join('\n')) + '</code></pre>')
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
        const level = line.match(/^#+/)[0].length
        html.push('<h' + level + '>' + renderInline(line.replace(/^#{1,6}\s/, '')) + '</h' + level + '>')
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
        html.push('<li>' + renderInline(line.replace(/^(\*\s|-\s|\d+\.\s)/, '')) + '</li>')
        continue
      }

      if (!line.trim()) {
        flushList()
        continue
      }

      flushList()
      html.push('<p>' + renderInline(line) + '</p>')
    }

    flushList()
    return html.join('')
  }

  function fileNameFromPath(path) {
    const parts = path.split(/[/\\]/)
    return parts[parts.length - 1] || path
  }

  function resolveBootDark() {
    const mode = localStorage.getItem('theme-mode')
    if (mode === 'dark') return true
    if (mode === 'light') return false
    return window.matchMedia('(prefers-color-scheme: dark)').matches
  }

  function applyBootTheme(layer) {
    const isDark = resolveBootDark()
    document.documentElement.classList.toggle('boot-dark', isDark)
    if (!layer) return
    layer.style.background = isDark ? '#1e1e1e' : '#ffffff'
    layer.style.color = isDark ? '#cccccc' : '#333333'
    const header = layer.querySelector('.markfly-boot-header')
    if (header) {
      header.style.background = isDark ? '#252526' : '#ececec'
      header.style.borderBottomColor = isDark ? '#3e3e42' : '#e5e5e5'
      header.style.color = isDark ? '#cccccc' : '#333333'
    }
  }

  function ensureLayer() {
    let layer = document.getElementById('markfly-boot-layer')
    if (layer) return layer

    layer = document.createElement('div')
    layer.id = 'markfly-boot-layer'
    layer.innerHTML =
      '<div class="markfly-boot-header"><span id="markfly-boot-tab"></span></div>' +
      '<div id="markfly-boot-content" class="markfly-boot-content markdown-body"></div>'
    document.body.appendChild(layer)
    applyBootTheme(layer)
    return layer
  }

  function renderBoot(files) {
    if (!files || !files.length) return

    const first = files[0]
    ensureLayer()
    document.getElementById('markfly-boot-tab').textContent = fileNameFromPath(first.path)
    let html = renderMarkdown(first.content || '')
    if (first.truncated) {
      html += '<p class="markfly-boot-truncated"><em>文档较长，正在加载完整内容…</em></p>'
    }
    document.getElementById('markfly-boot-content').innerHTML = html
  }

  window.__markflyApplyBoot = function (files) {
    window.__MARKFLY_BOOT__ = files
    renderBoot(files)
  }

  window.__markflyHideBoot = function () {
    const layer = document.getElementById('markfly-boot-layer')
    if (layer) layer.remove()
    document.documentElement.classList.remove('boot-dark')
  }

  applyBootTheme(null)

  if (window.__MARKFLY_BOOT__ && window.__MARKFLY_BOOT__.length) {
    renderBoot(window.__MARKFLY_BOOT__)
  }
})()

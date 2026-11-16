import type { BytemdPlugin } from 'bytemd'
import { alignFlowchartLayout } from './mermaidFlowchart'

type MermaidApi = typeof import('mermaid').default

const LIGHT_THEME_VARS = {
  primaryColor: '#ffffff',
  primaryTextColor: '#222222',
  primaryBorderColor: '#c8c8c8',
  lineColor: '#666666',
  secondaryColor: '#f4f4f4',
  tertiaryColor: '#f4f4f4',
  background: '#ffffff',
  mainBkg: '#ffffff',
  nodeBorder: '#c8c8c8',
  clusterBkg: '#f4f4f4',
  clusterBorder: '#d4d4d4',
  titleColor: '#333333',
  edgeLabelBackground: '#ffffff',
}

const DARK_THEME_VARS = {
  primaryColor: '#2d2d30',
  primaryTextColor: '#cccccc',
  primaryBorderColor: '#5a5a5a',
  lineColor: '#9a9a9a',
  secondaryColor: '#252526',
  tertiaryColor: '#252526',
  background: '#1e1e1e',
  mainBkg: '#2d2d30',
  nodeBorder: '#5a5a5a',
  clusterBkg: '#252526',
  clusterBorder: '#3e3e42',
  titleColor: '#cccccc',
  edgeLabelBackground: '#1e1e1e',
}

const isDarkTheme = () => document.documentElement.classList.contains('dark')

const applyMermaidConfig = (mermaid: MermaidApi) => {
  const dark = isDarkTheme()
  mermaid.initialize({
    startOnLoad: false,
    securityLevel: 'strict',
    theme: dark ? 'dark' : 'base',
    themeVariables: dark ? DARK_THEME_VARS : LIGHT_THEME_VARS,
    flowchart: {
      htmlLabels: true,
      useMaxWidth: true,
      padding: 12,
      nodeSpacing: 40,
      rankSpacing: 50,
      curve: 'basis',
    },
  })
}

const renderError = (container: HTMLElement, message: string) => {
  const pre = document.createElement('pre')
  pre.className = 'markfly-mermaid-error'
  pre.textContent = `Mermaid 渲染失败：${message}`
  container.replaceChildren(pre)
}

/**
 * 自管 Mermaid 渲染：布局对齐 Cursor，配色走 base + 主题变量。
 * 工具栏「插入图表」仍复用 @bytemd/plugin-mermaid 的 actions。
 */
export const createMermaidPlugin = async (): Promise<BytemdPlugin> => {
  const [{ default: mermaidByteMd }, { default: mermaid }] = await Promise.all([
    import('@bytemd/plugin-mermaid'),
    import('mermaid'),
  ])
  const toolbar = mermaidByteMd()
  let seq = 0

  return {
    actions: toolbar.actions,
    viewerEffect({ markdownBody }) {
      const blocks = markdownBody.querySelectorAll('pre > code.language-mermaid')
      if (blocks.length === 0) return

      applyMermaidConfig(mermaid)

      blocks.forEach((code, index) => {
        const pre = code.parentElement
        if (!pre) return

        const source = code.textContent || ''
        const container = document.createElement('div')
        container.className = 'markfly-mermaid bytemd-mermaid'
        pre.replaceWith(container)

        const id = `markfly-mermaid-${Date.now()}-${seq++}-${index}`
        const aligned = alignFlowchartLayout(source)

        mermaid
          .render(id, aligned)
          .then(({ svg, bindFunctions }) => {
            container.innerHTML = svg
            bindFunctions?.(container)
          })
          .catch((error: unknown) => {
            const message = error instanceof Error ? error.message : String(error)
            renderError(container, message)
            console.error('Mermaid 渲染失败:', error)
          })
      })
    },
  }
}

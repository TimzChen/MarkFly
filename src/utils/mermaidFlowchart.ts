/** Mermaid 流程图方向 */
export type FlowDir = 'TB' | 'BT' | 'LR' | 'RL'

const HEADER_RE = /^\s*(flowchart(?:-elk)?|graph)(?:\s+(TD|TB|BT|RL|LR))?\b/i

const normalizeDir = (raw?: string): FlowDir => {
  const dir = (raw || 'TB').trim().toUpperCase()
  if (dir === 'TD') return 'TB'
  if (dir === 'BT' || dir === 'LR' || dir === 'RL') return dir
  return 'TB'
}

/**
 * Cursor 预览用 flowchart-elk（ELK 布局）。
 * 普通 flowchart/graph 走 dagre，嵌套 subgraph 会挤在一起、同级顺序也和 Cursor 不一致。
 */
export const alignFlowchartLayout = (source: string): string => {
  const text = source.replace(/\r\n/g, '\n').replace(/^\uFEFF/, '')
  const headerMatch = text.match(HEADER_RE)
  if (!headerMatch) return source

  const parentDir = normalizeDir(headerMatch[2])
  const rest = text.slice(headerMatch[0].length)
  return `flowchart-elk ${parentDir}${rest}`
}

/** Mermaid 流程图方向 */
export type FlowDir = 'TB' | 'BT' | 'LR' | 'RL'

type Piece = { type: 'text' | 'subgraph'; content: string }

const HEADER_RE = /^\s*(flowchart|graph)(?:\s+(TD|TB|BT|RL|LR))?\b/i
const SUBGRAPH_START_RE = /^\s*subgraph\b/
const SUBGRAPH_END_RE = /^\s*end\s*$/
const DIRECTION_RE = /^\s*direction\s+(TD|TB|BT|RL|LR)\s*$/i

const normalizeDir = (raw?: string): FlowDir => {
  const dir = (raw || 'TB').trim().toUpperCase()
  if (dir === 'TD') return 'TB'
  if (dir === 'BT' || dir === 'LR' || dir === 'RL') return dir
  return 'TB'
}

/**
 * 把源码拆成「普通文本」和「顶层 subgraph」交替片段。
 * 嵌套 subgraph 留在父块内部，不单独抽出。
 */
const splitTopLevelSubgraphs = (source: string): Piece[] => {
  const lines = source.replace(/\r\n/g, '\n').split('\n')
  const pieces: Piece[] = []
  let depth = 0
  let buffer: string[] = []
  let subgraph: string[] = []

  const flushText = () => {
    if (buffer.length === 0) return
    pieces.push({ type: 'text', content: buffer.join('\n') })
    buffer = []
  }

  for (const line of lines) {
    if (depth === 0 && SUBGRAPH_START_RE.test(line)) {
      flushText()
      depth = 1
      subgraph = [line]
      continue
    }

    if (depth > 0) {
      subgraph.push(line)
      if (SUBGRAPH_START_RE.test(line)) depth += 1
      else if (SUBGRAPH_END_RE.test(line)) {
        depth -= 1
        if (depth === 0) {
          pieces.push({ type: 'subgraph', content: subgraph.join('\n') })
          subgraph = []
        }
      }
      continue
    }

    buffer.push(line)
  }

  if (subgraph.length > 0) buffer.push(...subgraph)
  flushText()
  return pieces
}

const ensureSubgraphDirection = (block: string, dir: FlowDir): string => {
  const lines = block.split('\n')
  const body = lines.slice(1)
  if (body.some((line) => DIRECTION_RE.test(line))) return block
  return [lines[0], `  direction ${dir}`, ...body].join('\n')
}

/**
 * 对齐 Cursor 的 flowchart 布局：
 * 1. subgraph 未写 direction 时继承父图方向（Mermaid 10 默认当成 TB）
 * 2. 同级 subgraph 按源码声明顺序排列（Mermaid dagre 会把它们倒过来）
 */
export const alignFlowchartLayout = (source: string): string => {
  const text = source.replace(/\r\n/g, '\n').replace(/^\uFEFF/, '')
  const headerMatch = text.match(HEADER_RE)
  if (!headerMatch) return source

  const parentDir = normalizeDir(headerMatch[2])
  const pieces = splitTopLevelSubgraphs(text)
  const subgraphIndexes = pieces
    .map((piece, index) => (piece.type === 'subgraph' ? index : -1))
    .filter((index) => index >= 0)

  if (subgraphIndexes.length === 0) return source

  for (const index of subgraphIndexes) {
    pieces[index].content = ensureSubgraphDirection(pieces[index].content, parentDir)
  }

  // dagre 把同级 subgraph 画成声明顺序的反序；先倒一次，渲染后才是源码顺序
  if (subgraphIndexes.length > 1) {
    const reversed = subgraphIndexes.map((index) => pieces[index].content).reverse()
    subgraphIndexes.forEach((index, offset) => {
      pieces[index].content = reversed[offset]
    })
  }

  return pieces.map((piece) => piece.content).join('\n')
}

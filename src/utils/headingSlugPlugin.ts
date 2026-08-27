import type { BytemdPlugin } from 'bytemd'
import { visit } from 'unist-util-visit'
import { createHeadingSlugger } from './headingSlug'

type HastLike = {
  type: string
  tagName?: string
  value?: string
  properties?: Record<string, unknown>
  children?: HastLike[]
}

function collectText(node: HastLike): string {
  if (node.type === 'text') return node.value ?? ''
  if (!node.children?.length) return ''
  return node.children.map(collectText).join('')
}

/** ByteMD 在 sanitize 之后跑 rehype，这里补上的 id 不会被清掉 */
export function createHeadingSlugPlugin(): BytemdPlugin {
  return {
    rehype: (processor) =>
      processor.use(() => (tree: HastLike) => {
        const slugger = createHeadingSlugger()
        visit(tree, 'element', (node: HastLike) => {
          if (!node.tagName || !/^h[1-6]$/.test(node.tagName)) return
          const props = node.properties ?? (node.properties = {})
          if (typeof props.id === 'string' && props.id) return
          props.id = slugger(collectText(node))
        })
      }),
  }
}

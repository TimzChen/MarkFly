<template>
  <aside class="toc-panel" :class="{ collapsed }" :style="panelStyle" :aria-hidden="collapsed">
    <button type="button" class="toc-close-btn" title="关闭目录" @click="emit('close')">
      <svg width="14" height="14" viewBox="0 0 48 48" fill="none">
        <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="4" d="m8 8 32 32M8 40 40 8"/>
      </svg>
    </button>
    <h2 class="toc-title">目录</h2>
    <nav class="toc-content" aria-label="文档目录">
      <div v-if="headings.length === 0" class="toc-empty">暂无标题</div>
      <button
        v-for="(heading, index) in headings"
        :key="`${index}-${heading.text}`"
        type="button"
        class="toc-item"
        :class="{
          'is-active': index === activeIndex,
          'is-first': heading.level === minLevel,
        }"
        :style="{ paddingLeft: `${(heading.level - minLevel) * 16 + 8}px` }"
        @click="scrollToHeading(index)"
      >
        <span class="toc-text">{{ heading.text }}</span>
      </button>
    </nav>
  </aside>
</template>

<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'

type Heading = {
  text: string
  level: number
}

type TocScrollRoot = HTMLElement | null | (() => HTMLElement | null)

const props = withDefaults(defineProps<{
  content: string
  scrollRoot?: TocScrollRoot
  /** 预览容器切换时（分屏 / 仅预览）重新绑定滚动高亮 */
  syncKey?: string
  collapsed?: boolean
  width?: number
}>(), {
  collapsed: false,
  width: 280,
  syncKey: '',
})

const panelStyle = computed(() => {
  if (props.collapsed) {
    return { width: '0px', minWidth: '0px', flexBasis: '0px' }
  }
  return {
    width: `${props.width}px`,
    minWidth: `${props.width}px`,
    flexBasis: `${props.width}px`,
  }
})

const emit = defineEmits<{
  close: []
}>()

/** 从 markdown 提取标题；跳过 fenced code，避免把 `# 注释` 当成标题导致目录索引错位 */
const extractHeadings = (markdown: string): Heading[] => {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n')
  const matches: Heading[] = []
  let inCodeFence = false

  for (const line of lines) {
    if (/^\s*(```|~~~)/.test(line)) {
      inCodeFence = !inCodeFence
      continue
    }
    if (inCodeFence) continue
    const match = line.match(/^(#{1,6})\s+(.+)$/)
    if (!match) continue
    matches.push({
      level: match[1].length,
      text: match[2].trim(),
    })
  }
  return matches
}

const headings = computed(() => extractHeadings(props.content))
const minLevel = computed(() => {
  if (headings.value.length === 0) return 1
  return Math.min(...headings.value.map((item) => item.level))
})
const activeIndex = ref(0)

const resolveScrollRoot = (): HTMLElement | null => {
  const root = props.scrollRoot
  return typeof root === 'function' ? root() : root ?? null
}

const headingNodes = (root: HTMLElement) =>
  root.querySelectorAll<HTMLElement>('h1,h2,h3,h4,h5,h6')

const headingOffsetInRoot = (root: HTMLElement, target: HTMLElement) => {
  const rootRect = root.getBoundingClientRect()
  const targetRect = target.getBoundingClientRect()
  return targetRect.top - rootRect.top + root.scrollTop
}

const updateActiveFromScroll = () => {
  const root = boundRoot
  if (!root || headings.value.length === 0) return
  const nodes = headingNodes(root)
  if (nodes.length === 0) return
  const rootTop = root.getBoundingClientRect().top
  let current = 0
  for (let index = 0; index < nodes.length; index += 1) {
    if (nodes[index].getBoundingClientRect().top - rootTop <= 12) current = index
    else break
  }
  activeIndex.value = Math.min(current, headings.value.length - 1)
}

const scrollToHeading = (index: number) => {
  const root = resolveScrollRoot()
  if (!root) return
  const target = headingNodes(root)[index]
  if (!target) return
  activeIndex.value = index
  const nextTop = headingOffsetInRoot(root, target)
  const maxScroll = Math.max(0, root.scrollHeight - root.clientHeight)
  root.scrollTo({ top: Math.min(Math.max(0, nextTop), maxScroll), behavior: 'smooth' })
}

let boundRoot: HTMLElement | null = null
let retryTimer = 0
let retryCount = 0

const unbindScroll = () => {
  window.clearTimeout(retryTimer)
  boundRoot?.removeEventListener('scroll', updateActiveFromScroll)
  boundRoot = null
}

const bindScroll = () => {
  unbindScroll()
  const root = resolveScrollRoot()
  const hasHeadings = headings.value.length > 0
  const nodesReady = !!root && (!hasHeadings || headingNodes(root).length > 0)
  if (!root || !nodesReady) {
    if (retryCount < 20) {
      retryCount += 1
      retryTimer = window.setTimeout(bindScroll, 150)
    }
    return
  }
  retryCount = 0
  boundRoot = root
  root.addEventListener('scroll', updateActiveFromScroll, { passive: true })
  updateActiveFromScroll()
}

watch(
  () => [props.content, props.collapsed, props.syncKey] as const,
  async () => {
    retryCount = 0
    if (props.collapsed) {
      unbindScroll()
      return
    }
    await nextTick()
    bindScroll()
  },
  { immediate: true }
)

onUnmounted(unbindScroll)
</script>

<style scoped>
.toc-panel {
  position: relative;
  flex: 0 0 auto;
  background: var(--bg-primary);
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color);
  overflow: hidden;
  transition:
    width 0.2s ease,
    min-width 0.2s ease,
    flex-basis 0.2s ease,
    opacity 0.2s ease,
    border-color 0.2s ease,
    background-color 0.3s ease,
    color 0.3s ease;
}

.toc-panel.collapsed {
  opacity: 0;
  border-right-color: transparent;
  pointer-events: none;
}

.toc-title {
  margin: 24px 16px 8px;
  font-size: 16px;
  font-weight: 600;
  line-height: 1.3;
  color: var(--text-primary);
}

.toc-close-btn {
  position: absolute;
  top: 10px;
  right: 8px;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.toc-close-btn:hover {
  color: var(--accent-color);
}

.toc-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0 8px 16px;
}

.toc-empty {
  padding: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}

.toc-item {
  display: block;
  width: 100%;
  border: none;
  background: transparent;
  text-align: left;
  padding-top: 0;
  padding-right: 8px;
  padding-bottom: 0;
  margin-bottom: 4px;
  cursor: pointer;
  font-size: 14px;
  line-height: 2;
  color: var(--text-secondary);
  border-radius: 4px;
}

.toc-item:hover {
  background: var(--hover-bg);
}

.toc-item.is-first {
  font-weight: 500;
  color: var(--text-primary);
}

.toc-item.is-active {
  color: var(--accent-color);
  background: var(--hover-bg);
}

.toc-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}
</style>

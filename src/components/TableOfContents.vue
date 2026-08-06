<template>
  <aside class="toc-panel">
    <div class="toc-header">
      <div class="header-title">
        <span class="header-icon" v-html="toolbarIcons.toc" />
        <span>目录</span>
      </div>
      <button type="button" class="toc-close-btn" title="关闭目录" @click="emit('close')">
        <svg width="12" height="12" viewBox="0 0 48 48" fill="none">
          <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="4" d="m8 8 32 32M8 40 40 8"/>
        </svg>
      </button>
    </div>

    <div class="toc-content">
      <div v-if="headings.length === 0" class="toc-empty">暂无标题</div>
      <button
        v-for="(heading, index) in headings"
        :key="`${index}-${heading.text}`"
        type="button"
        class="toc-item"
        :class="`level-${heading.level}`"
        @click="scrollToHeading(index)"
      >
        <span class="toc-text">{{ heading.text }}</span>
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { toolbarIcons } from '../utils/toolbarIcons'

interface Heading {
  text: string
  level: number
}

const props = defineProps<{
  content: string
  scrollRoot?: HTMLElement | null
}>()

const emit = defineEmits<{
  close: []
}>()

/** 从 markdown 提取标题；跳过 fenced code，避免把 `# 注释` 当成标题导致目录索引错位 */
const extractHeadings = (markdown: string): Heading[] => {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n')
  const matches: Heading[] = []
  let inCodeFence = false

  for (const line of lines) {
    // 围栏代码块（``` / ~~~），与缩进无关
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

const scrollToHeading = (index: number) => {
  const root = props.scrollRoot
  if (!root) return

  const target = root.querySelectorAll('h1,h2,h3,h4,h5,h6')[index] as HTMLElement | undefined
  if (!target) return

  // 只滚动预览容器，避免 scrollIntoView 连带外层滚动；文末标题 clamp 到 maxScroll
  const rootRect = root.getBoundingClientRect()
  const targetRect = target.getBoundingClientRect()
  const nextTop = targetRect.top - rootRect.top + root.scrollTop
  const maxScroll = Math.max(0, root.scrollHeight - root.clientHeight)
  root.scrollTo({ top: Math.min(Math.max(0, nextTop), maxScroll), behavior: 'smooth' })
}
</script>

<style scoped>
.toc-panel {
  width: 250px;
  flex: 0 0 250px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--border-color);
  transition: background-color 0.3s ease, color 0.3s ease;
}

.toc-header {
  height: 35px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px 0 12px;
  border-bottom: 1px solid var(--border-color);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  flex-shrink: 0;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-primary);
}

.header-icon :deep(svg) {
  width: 14px;
  height: 14px;
  display: block;
}

.toc-close-btn {
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.toc-close-btn:hover {
  background: var(--hover-bg);
  color: var(--text-primary);
}

.toc-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 8px 0;
}

.toc-empty {
  padding: 12px;
  font-size: 12px;
  color: var(--text-secondary);
}

.toc-item {
  width: 100%;
  border: none;
  background: transparent;
  text-align: left;
  padding: 3px 12px;
  cursor: pointer;
  font-size: 13px;
  line-height: 1.4;
  color: var(--text-primary);
  transition: background-color 0.2s;
  border-left: 2px solid transparent;
}

.toc-item:hover {
  background: var(--hover-bg);
  border-left-color: var(--accent-color);
}

.toc-item.level-1 {
  padding-left: 12px;
  font-weight: 600;
}

.toc-item.level-2 {
  padding-left: 24px;
}

.toc-item.level-3 {
  padding-left: 36px;
  font-size: 12px;
}

.toc-item.level-4 {
  padding-left: 48px;
  font-size: 12px;
}

.toc-item.level-5 {
  padding-left: 60px;
  font-size: 11px;
}

.toc-item.level-6 {
  padding-left: 72px;
  font-size: 11px;
}

.toc-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}
</style>

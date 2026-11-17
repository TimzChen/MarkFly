<template>
  <div class="file-tree" :class="{ collapsed }" :style="panelStyle" :aria-hidden="collapsed">
    <div class="kb-head">
      <h2 class="kb-title">知识库</h2>
      <div class="header-actions">
        <button class="action-btn" title="新建文件" @click="$emit('newFile')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path d="M12 5v14m-7-7h14" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
        <button class="action-btn" title="打开文件夹" @click="$emit('openFolder')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path d="M3 3h7l2 3h9v12H3V3z" stroke="currentColor" stroke-width="2" fill="none"/>
          </svg>
        </button>
        <button
          v-if="kbPane === 'folder' && folderName"
          class="action-btn"
          :class="{ 'is-busy': folderRefreshing }"
          title="刷新目录"
          :aria-busy="folderRefreshing"
          @click="$emit('refreshFolder')"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path d="M3 12a9 9 0 0 1 15.5-6.4M21 12a9 9 0 0 1-15.5 6.4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            <path d="M18.5 5.6V9h-3.4M5.5 18.4V15h3.4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        <button
          class="action-btn danger"
          :disabled="clearDisabled"
          :title="clearTitle"
          @click="onClearCurrentPane"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path d="M4 7h16M9 7V5h6v2m-7 0l.7 12h7.6L17 7" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>
          </svg>
        </button>
        <button type="button" class="action-btn kb-close-btn" title="关闭知识库" @click="emit('close')">
          <svg width="14" height="14" viewBox="0 0 48 48" fill="none">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="4" d="m8 8 32 32M8 40 40 8"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="kb-seg" role="tablist" aria-label="知识库页签">
      <button
        type="button"
        role="tab"
        class="kb-seg-btn"
        :class="{ 'is-active': kbPane === 'opened' }"
        :aria-selected="kbPane === 'opened'"
        @click="$emit('changePane', 'opened')"
      >
        已打开
      </button>
      <button
        type="button"
        role="tab"
        class="kb-seg-btn"
        :class="{ 'is-active': kbPane === 'recents' }"
        :aria-selected="kbPane === 'recents'"
        @click="$emit('changePane', 'recents')"
      >
        最近打开
      </button>
      <button
        type="button"
        role="tab"
        class="kb-seg-btn"
        :class="{ 'is-active': kbPane === 'folder' }"
        :aria-selected="kbPane === 'folder'"
        @click="$emit('changePane', 'folder')"
      >
        目录
      </button>
    </div>

    <div v-if="kbPane === 'opened'" class="file-list" @contextmenu.prevent="openBlankMenu">
      <div v-if="files.length === 0" class="kb-empty">
        <p>还没有打开的文件</p>
        <span>从「最近打开」挑一个，或新建 / 打开文件</span>
      </div>
      <div
        v-for="file in files"
        :key="file.path"
        class="file-item"
        :class="{ active: file.path === activeFile }"
        @click="$emit('selectFile', file)"
        @contextmenu.prevent.stop="openOpenedMenu($event, file)"
      >
        <div class="file-icon">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" stroke="currentColor" stroke-width="2" fill="none"/>
            <polyline points="14,2 14,8 20,8" stroke="currentColor" stroke-width="2"/>
          </svg>
        </div>
        <div class="file-meta">
          <input
            v-if="renamingPath === file.path"
            :ref="bindRenameInput"
            class="file-rename-input"
            v-model="renameDraft"
            @click.stop
            @pointerdown.stop
            @keydown="onRenameKeydown"
            @blur="commitRename"
          />
          <span v-else class="file-name">{{ file.name }}</span>
        </div>
        <button
          class="row-close"
          title="关闭文件"
          @click.stop="$emit('closeFile', file)"
        >
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>
    </div>

    <div v-else-if="kbPane === 'recents'" class="file-list" @contextmenu.prevent="openBlankMenu">
      <div v-if="recents.length === 0" class="kb-empty">
        <p>还没有最近打开</p>
        <span>打开过的磁盘文件会留在这里</span>
      </div>
      <div
        v-for="item in recents"
        :key="item.path"
        class="file-item is-recent"
        :class="{ active: item.path === activeFile }"
        :title="item.path"
        @click="$emit('selectRecent', item.path)"
        @contextmenu.prevent.stop="openRecentMenu($event, item)"
      >
        <div class="file-icon">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" stroke="currentColor" stroke-width="2" fill="none"/>
            <polyline points="14,2 14,8 20,8" stroke="currentColor" stroke-width="2"/>
          </svg>
        </div>
        <div class="file-meta">
          <input
            v-if="renamingPath === item.path"
            :ref="bindRenameInput"
            class="file-rename-input"
            v-model="renameDraft"
            @click.stop
            @pointerdown.stop
            @keydown="onRenameKeydown"
            @blur="commitRename"
          />
          <template v-else>
            <span class="file-name">{{ item.name }}</span>
            <span v-if="item.parentDir" class="file-parent">{{ item.parentDir }}</span>
          </template>
        </div>
        <button
          class="row-close"
          title="从最近打开移除"
          @click.stop="$emit('removeRecent', item.path)"
        >
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>
    </div>

    <div v-else class="file-list" @contextmenu.prevent="openBlankMenu">
      <div v-if="!folderName" class="kb-empty">
        <p>还没有打开目录</p>
        <span>仅本次有效，点上方按钮选择</span>
      </div>
      <div v-else-if="folderFiles.length === 0" class="kb-empty">
        <p>{{ folderName }}</p>
        <span>这个文件夹及其子目录里没有 Markdown / 文本文件</span>
      </div>
      <template v-for="row in folderRows" :key="row.kind === 'dir' ? `dir:${row.key}` : row.item.path">
        <button
          v-if="row.kind === 'dir'"
          type="button"
          class="file-item is-dir"
          :class="{ 'is-collapsed': row.collapsed }"
          :style="treePad(row.depth)"
          :title="row.key"
          :aria-expanded="!row.collapsed"
          @click="toggleDir(row.key)"
        >
          <span class="tree-twist" :class="{ 'is-open': !row.collapsed }">
            <svg width="10" height="10" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path d="M6 3.5 12 8 6 12.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </span>
          <span class="file-icon is-folder">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M3 7h6l2 2h10v10H3V7z" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>
            </svg>
          </span>
          <span class="file-name">{{ row.name }}</span>
        </button>
        <div
          v-else
          class="file-item is-recent"
          :class="{ active: row.item.path === activeFile }"
          :style="treePad(row.depth)"
          :title="row.item.path"
          @click="$emit('selectFolderFile', row.item.path)"
          @contextmenu.prevent.stop="openFolderItemMenu($event, row.item)"
        >
          <span class="tree-twist is-leaf" aria-hidden="true" />
          <div class="file-icon">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" stroke="currentColor" stroke-width="2" fill="none"/>
              <polyline points="14,2 14,8 20,8" stroke="currentColor" stroke-width="2"/>
            </svg>
          </div>
          <div class="file-meta">
            <input
              v-if="renamingPath === row.item.path"
              :ref="bindRenameInput"
              class="file-rename-input"
              v-model="renameDraft"
              @click.stop
              @pointerdown.stop
              @keydown="onRenameKeydown"
              @blur="commitRename"
            />
            <span v-else class="file-name">{{ row.item.name }}</span>
          </div>
        </div>
      </template>
    </div>

    <Teleport to="body">
      <div
        v-if="menu"
        ref="menuEl"
        class="kb-menu"
        :style="{ left: `${menu.x}px`, top: `${menu.y}px` }"
        @pointerdown.stop
        @click.stop
        @contextmenu.prevent
      >
        <template v-if="menu.kind === 'opened'">
          <button type="button" class="kb-menu-item" @click="run(() => emit('closeFile', menu.file))">
            关闭
          </button>
          <button
            type="button"
            class="kb-menu-item"
            :disabled="files.length < 2"
            @click="run(() => emit('closeOthers', menu.file))"
          >
            关闭其他
          </button>
          <button type="button" class="kb-menu-item is-danger" @click="run(() => emit('closeAllFiles'))">
            关闭全部
          </button>
          <div class="kb-menu-sep" />
          <button type="button" class="kb-menu-item" @click="run(() => beginRename(menu.file.path, menu.file.name))">
            重命名
          </button>
          <button type="button" class="kb-menu-item" @click="copyText(menu.file.name)">
            复制文件名
          </button>
          <button
            type="button"
            class="kb-menu-item"
            :disabled="!isDiskFilePath(menu.file.path)"
            @click="copyText(menu.file.path)"
          >
            复制文件路径
          </button>
          <div class="kb-menu-sep" />
          <button
            type="button"
            class="kb-menu-item"
            :disabled="!isDiskFilePath(menu.file.path)"
            @click="run(() => emit('revealPath', menu.file.path))"
          >
            打开文件目录
          </button>
          <button
            type="button"
            class="kb-menu-item"
            :disabled="!isDiskFilePath(menu.file.path)"
            @click="run(() => emit('refreshFile', menu.file))"
          >
            从磁盘刷新
          </button>
        </template>

        <template v-else-if="menu.kind === 'folder'">
          <button type="button" class="kb-menu-item" @click="run(() => emit('selectFolderFile', menu.item.path))">
            打开
          </button>
          <button type="button" class="kb-menu-item" @click="run(() => beginRename(menu.item.path, menu.item.name))">
            重命名
          </button>
          <div class="kb-menu-sep" />
          <button type="button" class="kb-menu-item" @click="copyText(menu.item.name)">
            复制文件名
          </button>
          <button type="button" class="kb-menu-item" @click="copyText(menu.item.path)">
            复制文件路径
          </button>
          <button type="button" class="kb-menu-item" @click="run(() => emit('revealPath', menu.item.path))">
            打开文件目录
          </button>
          <button type="button" class="kb-menu-item" @click="run(() => emit('refreshFolder'))">
            刷新目录
          </button>
        </template>

        <template v-else-if="menu.kind === 'recent'">
          <button type="button" class="kb-menu-item" @click="run(() => emit('selectRecent', menu.item.path))">
            打开
          </button>
          <button type="button" class="kb-menu-item" @click="run(() => beginRename(menu.item.path, menu.item.name))">
            重命名
          </button>
          <div class="kb-menu-sep" />
          <button type="button" class="kb-menu-item" @click="copyText(menu.item.name)">
            复制文件名
          </button>
          <button type="button" class="kb-menu-item" @click="copyText(menu.item.path)">
            复制文件路径
          </button>
          <button type="button" class="kb-menu-item" @click="run(() => emit('revealPath', menu.item.path))">
            打开文件目录
          </button>
          <div class="kb-menu-sep" />
          <button type="button" class="kb-menu-item" @click="run(() => emit('removeRecent', menu.item.path))">
            从最近打开移除
          </button>
          <button type="button" class="kb-menu-item is-danger" @click="run(() => emit('clearRecents'))">
            清空最近打开
          </button>
        </template>

        <template v-else>
          <button type="button" class="kb-menu-item" @click="run(() => emit('newFile'))">
            新建文件
          </button>
          <button type="button" class="kb-menu-item" @click="run(() => emit('openFile'))">
            打开文件...
          </button>
          <template v-if="kbPane === 'opened' && files.length > 0">
            <div class="kb-menu-sep" />
            <button type="button" class="kb-menu-item is-danger" @click="run(() => emit('closeAllFiles'))">
              关闭全部
            </button>
          </template>
          <template v-else-if="kbPane === 'recents' && recents.length > 0">
            <div class="kb-menu-sep" />
            <button type="button" class="kb-menu-item is-danger" @click="run(() => emit('clearRecents'))">
              清空最近打开
            </button>
          </template>
          <template v-else-if="kbPane === 'folder'">
            <div class="kb-menu-sep" />
            <button
              v-if="folderName"
              type="button"
              class="kb-menu-item"
              @click="run(() => emit('refreshFolder'))"
            >
              刷新目录
            </button>
            <button type="button" class="kb-menu-item" @click="run(() => emit('openFolder'))">
              选择文件夹...
            </button>
            <button
              v-if="folderName"
              type="button"
              class="kb-menu-item is-danger"
              @click="run(() => emit('closeFolder'))"
            >
              关闭此目录
            </button>
          </template>
        </template>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import type { FileItem } from '../data/sampleFiles'
import { flattenFolderTree, treePad } from '../utils/folderTree'
import { isDiskFilePath } from '../utils/previewAssets'
import { sameFilePath, type KnowledgePane, type RecentFile } from '../utils/recentFiles'

type KbMenu =
  | { kind: 'opened'; x: number; y: number; file: FileItem }
  | { kind: 'recent'; x: number; y: number; item: RecentFile }
  | { kind: 'folder'; x: number; y: number; item: RecentFile }
  | { kind: 'blank'; x: number; y: number }

const props = withDefaults(defineProps<{
  files: FileItem[]
  recents: RecentFile[]
  folderFiles: RecentFile[]
  folderName: string
  folderPath?: string
  startRenamePath: string
  activeFile: string
  kbPane: KnowledgePane
  folderRefreshing?: boolean
  collapsed?: boolean
  width?: number
}>(), {
  collapsed: false,
  folderRefreshing: false,
  width: 280,
  folderFiles: () => [],
  folderName: '',
  folderPath: '',
  startRenamePath: '',
})

const emit = defineEmits<{
  selectFile: [file: FileItem]
  selectRecent: [path: string]
  selectFolderFile: [path: string]
  newFile: []
  openFile: []
  openFolder: []
  closeFile: [file: FileItem]
  closeOthers: [file: FileItem]
  closeAllFiles: []
  removeRecent: [path: string]
  clearRecents: []
  closeFolder: []
  refreshFolder: []
  changePane: [pane: KnowledgePane]
  revealPath: [path: string]
  refreshFile: [file: FileItem]
  contextOpened: []
  renameFile: [payload: { path: string; newName: string }]
  renameStarted: []
  close: []
}>()

const menu = ref<KbMenu | null>(null)
const menuEl = ref<HTMLElement | null>(null)
const renameInput = ref<HTMLInputElement | null>(null)
const renamingPath = ref('')
const renameDraft = ref('')
let menuOpenedAt = 0
let renameLocked = false

const bindRenameInput = (el: unknown) => {
  renameInput.value = el instanceof HTMLInputElement ? el : null
}

const panelStyle = computed(() => {
  if (props.collapsed) {
    return { width: '0px', minWidth: '0px' }
  }
  return { width: `${props.width}px`, minWidth: `${props.width}px` }
})

const clearDisabled = computed(() => {
  if (props.kbPane === 'opened') return props.files.length === 0
  if (props.kbPane === 'recents') return props.recents.length === 0
  return !props.folderName
})

const collapsedDirs = ref<Record<string, boolean>>({})

const folderRows = computed(() => flattenFolderTree(props.folderFiles, collapsedDirs.value))

const toggleDir = (key: string) => {
  collapsedDirs.value = {
    ...collapsedDirs.value,
    [key]: !collapsedDirs.value[key],
  }
}

watch(
  () => props.folderPath || props.folderName,
  () => {
    collapsedDirs.value = {}
  }
)

const clearTitle = computed(() => {
  if (props.kbPane === 'opened') {
    return props.files.length === 0 ? '没有可关闭的文件' : '关闭全部'
  }
  if (props.kbPane === 'recents') {
    return props.recents.length === 0 ? '没有可清空的记录' : '清空最近打开'
  }
  return props.folderName ? '关闭此目录' : '还没有打开目录'
})

const onClearCurrentPane = () => {
  if (clearDisabled.value) return
  if (props.kbPane === 'opened') emit('closeAllFiles')
  else if (props.kbPane === 'recents') emit('clearRecents')
  else emit('closeFolder')
}

const closeMenu = (force = false) => {
  if (force !== true && Date.now() - menuOpenedAt < 120) return
  menu.value = null
}

const clampMenu = async () => {
  await nextTick()
  const el = menuEl.value
  const current = menu.value
  if (!el || !current) return
  const rect = el.getBoundingClientRect()
  const pad = 8
  let x = current.x
  let y = current.y
  if (x + rect.width > window.innerWidth - pad) x = window.innerWidth - pad - rect.width
  if (y + rect.height > window.innerHeight - pad) y = window.innerHeight - pad - rect.height
  menu.value = {
    ...current,
    x: Math.max(pad, x),
    y: Math.max(pad, y),
  }
}

const placeMenu = (next: KbMenu) => {
  menuOpenedAt = Date.now()
  menu.value = next
  emit('contextOpened')
  void clampMenu()
}

const openOpenedMenu = (event: MouseEvent, file: FileItem) => {
  placeMenu({ kind: 'opened', x: event.clientX, y: event.clientY, file })
}

const openRecentMenu = (event: MouseEvent, item: RecentFile) => {
  placeMenu({ kind: 'recent', x: event.clientX, y: event.clientY, item })
}

const openFolderItemMenu = (event: MouseEvent, item: RecentFile) => {
  placeMenu({ kind: 'folder', x: event.clientX, y: event.clientY, item })
}

const openBlankMenu = (event: MouseEvent) => {
  placeMenu({ kind: 'blank', x: event.clientX, y: event.clientY })
}

const focusRenameInput = async () => {
  await nextTick()
  const el =
    renameInput.value instanceof HTMLInputElement
      ? renameInput.value
      : document.querySelector<HTMLInputElement>('.file-rename-input')
  if (!el) return
  el.scrollIntoView({ block: 'nearest' })
  el.focus()
  const stem = renameDraft.value.replace(/\.[^.]+$/, '')
  el.setSelectionRange(0, stem.length)
}

const beginRename = (path: string, name: string) => {
  renamingPath.value = path
  renameDraft.value = name
  void focusRenameInput()
}

const cancelRename = () => {
  renamingPath.value = ''
  renameDraft.value = ''
}

const commitRename = () => {
  if (renameLocked) return
  const path = renamingPath.value
  const newName = renameDraft.value
  if (!path) return
  renameLocked = true
  renamingPath.value = ''
  emit('renameFile', { path, newName })
  queueMicrotask(() => {
    renameLocked = false
  })
}

const onRenameKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    event.preventDefault()
    cancelRename()
    return
  }
  if (event.key !== 'Enter' || event.isComposing || event.keyCode === 229) return
  event.preventDefault()
  commitRename()
}

const itemNameByPath = (path: string) => {
  const match = (item: { path: string; name: string }) =>
    item.path === path || sameFilePath(item.path, path)
  return (
    props.files.find(match)?.name ||
    props.recents.find(match)?.name ||
    props.folderFiles.find(match)?.name
  )
}

watch(
  () => props.startRenamePath,
  (path) => {
    if (!path) return
    const name = itemNameByPath(path)
    if (!name) return
    beginRename(path, name)
    emit('renameStarted')
  },
  { immediate: true }
)

const run = (action: () => void) => {
  action()
  closeMenu(true)
}

const copyText = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
  } catch (error) {
    console.error('复制失败:', error)
  } finally {
    closeMenu(true)
  }
}

const onWindowPointerDown = () => closeMenu(false)

const onWindowKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    cancelRename()
    closeMenu(true)
  }
}

watch(() => props.kbPane, () => closeMenu(true))
watch(() => props.collapsed, (collapsed) => {
  if (collapsed) closeMenu(true)
})

onMounted(() => {
  window.addEventListener('pointerdown', onWindowPointerDown)
  window.addEventListener('keydown', onWindowKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('pointerdown', onWindowPointerDown)
  window.removeEventListener('keydown', onWindowKeyDown)
})
</script>

<style scoped>
.file-tree {
  position: relative;
  background: var(--bg-primary);
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--border-color);
  transition:
    width 0.2s ease,
    min-width 0.2s ease,
    opacity 0.2s ease,
    border-color 0.2s ease,
    background-color 0.3s ease,
    color 0.3s ease;
  overflow: hidden;
  flex-shrink: 0;
}

.file-tree.collapsed {
  opacity: 0;
  border-left-color: transparent;
  pointer-events: none;
}

.kb-close-btn {
  position: relative;
  margin-left: 4px;
}

.kb-close-btn::before {
  content: '';
  position: absolute;
  left: -4px;
  top: 7px;
  width: 1px;
  height: 14px;
  background: var(--border-color);
}

.kb-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 16px 8px 8px 16px;
  flex-shrink: 0;
}

.kb-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  line-height: 1.3;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}

.action-btn {
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
  transition: background-color 0.15s, color 0.15s, transform 0.12s;
}

.action-btn:hover:not(:disabled) {
  color: var(--accent-color);
  background: var(--hover-bg);
}

.action-btn:active:not(:disabled) {
  transform: scale(0.9);
  color: var(--accent-color);
  background: var(--hover-bg);
}

.action-btn.danger:hover:not(:disabled) {
  background: color-mix(in srgb, var(--danger-color) 14%, transparent);
  color: var(--danger-color);
}

.action-btn.danger:active:not(:disabled) {
  background: color-mix(in srgb, var(--danger-color) 22%, transparent);
}

.action-btn.is-busy {
  color: var(--accent-color);
}

.action-btn.is-busy svg {
  animation: kb-refresh-spin 1.25s linear infinite;
}

@keyframes kb-refresh-spin {
  to {
    transform: rotate(360deg);
  }
}

.action-btn:disabled {
  opacity: 0.32;
  cursor: default;
}

.kb-seg {
  display: flex;
  gap: 0;
  margin: 0 8px 8px;
  padding: 0;
  background: transparent;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.kb-seg-btn {
  flex: 1;
  height: 28px;
  border: none;
  border-bottom: 2px solid transparent;
  border-radius: 0;
  font-size: 13px;
  line-height: 26px;
  white-space: nowrap;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0;
  margin-bottom: -1px;
  transition: color 0.15s, border-color 0.15s;
}

.kb-seg-btn:hover:not(.is-active) {
  color: var(--text-primary);
}

.kb-seg-btn.is-active {
  background: transparent;
  color: var(--accent-color);
  font-weight: 500;
  box-shadow: none;
  border-bottom-color: var(--accent-color);
}

.file-list {
  flex: 1;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: 4px 8px 12px;
  scrollbar-width: thin;
  background: transparent;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px 6px 8px;
  cursor: pointer;
  font-size: 14px;
  color: var(--text-secondary);
  border-radius: 4px;
  transition: background-color 0.15s, color 0.15s;
}

.file-item.is-recent {
  padding-top: 7px;
  padding-bottom: 7px;
}

.file-item:hover {
  background: var(--hover-bg);
}

.file-item.active {
  background: var(--hover-bg);
  color: var(--accent-color);
}

.file-item.active .file-icon {
  color: var(--accent-color);
}

.file-icon {
  display: flex;
  align-items: center;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.file-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.file-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-rename-input {
  width: 100%;
  min-width: 0;
  height: 22px;
  border: 1px solid var(--accent-color);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
  line-height: 20px;
  padding: 0 6px;
  border-radius: 4px;
  outline: none;
  box-sizing: border-box;
}

.file-parent {
  font-size: 11px;
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-item.is-dir {
  width: 100%;
  border: none;
  background: transparent;
  font: inherit;
  text-align: left;
}

.file-item.is-dir .file-name {
  font-weight: 500;
  color: var(--text-primary);
}

.file-icon.is-folder {
  color: var(--text-secondary);
}

.tree-twist {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
}

.tree-twist svg {
  display: block;
  transform: rotate(0deg);
  transition: transform 0.15s ease;
}

.tree-twist.is-open svg {
  transform: rotate(90deg);
}

.tree-twist.is-leaf {
  visibility: hidden;
}

.row-close {
  width: 18px;
  height: 18px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  opacity: 0;
  pointer-events: none;
  cursor: pointer;
  transition: background-color 0.15s, color 0.15s, opacity 0.15s;
}

.file-item:hover .row-close,
.file-item:focus-within .row-close {
  opacity: 1;
  pointer-events: auto;
}

.row-close:hover {
  background: var(--danger-color);
  color: #fff;
}

.kb-empty {
  padding: 28px 16px;
  text-align: center;
  color: var(--text-secondary);
}

.kb-empty p {
  margin: 0 0 6px;
  font-size: 13px;
  color: var(--text-primary);
}

.kb-empty span {
  font-size: 12px;
  line-height: 1.5;
}

.kb-menu {
  position: fixed;
  z-index: 1100;
  min-width: 196px;
  padding: 6px 0;
  background: var(--bg-primary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.16);
}

.kb-menu-item {
  display: block;
  width: 100%;
  border: none;
  background: transparent;
  color: inherit;
  text-align: left;
  padding: 7px 14px;
  font-size: 13px;
  line-height: 1.3;
  cursor: pointer;
}

.kb-menu-item:hover:not(:disabled) {
  background: var(--hover-bg);
}

.kb-menu-item:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.kb-menu-item.is-danger {
  color: var(--danger-color);
}

.kb-menu-item.is-danger:hover:not(:disabled) {
  background: color-mix(in srgb, var(--danger-color) 12%, transparent);
}

.kb-menu-sep {
  height: 1px;
  margin: 5px 8px;
  background: var(--border-color);
}
</style>

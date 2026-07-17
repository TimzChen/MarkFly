<template>
  <div class="markfly-app">
    <!-- 主编辑区域 -->
    <div class="app-body" ref="appBodyRef">
      <!-- 左侧文件树 -->
      <FileTree 
        :files="files"
        :activeFile="currentFilePath"
        :collapsed="sidebarCollapsed"
        @selectFile="selectFile"
        @newFile="createNewFile"
        @openFolder="openFolder"
        @closeFile="closeFile"
      />

      <button
        class="sidebar-toggle-btn"
        :class="{
          dragging: isDraggingSidebarBtn,
          'is-collapsed': sidebarCollapsed,
          'is-expanded': !sidebarCollapsed,
        }"
        :style="sidebarToggleBtnStyle"
        :title="sidebarCollapsed ? '显示侧边栏 (可拖动，Ctrl+B)' : '隐藏侧边栏 (可拖动，Ctrl+B)'"
        @pointerdown="onSidebarBtnPointerDown"
        @pointermove="onSidebarBtnPointerMove"
        @pointerup="onSidebarBtnPointerUp"
        @pointercancel="onSidebarBtnPointerUp"
        @click="onSidebarBtnClick"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
          <polyline
            :points="sidebarCollapsed ? '9,6 15,12 9,18' : '15,6 9,12 15,18'"
            stroke="currentColor"
            stroke-width="2"
          />
        </svg>
      </button>
      
      <!-- 中央编辑区域 -->
      <div class="editor-container">
        <div
          v-if="files.length > 0"
          class="editor-unified-header"
          :class="{ 'is-merged': showUnifiedHeader }"
          @click.capture="handleToolbarLayoutClick"
        >
          <div class="file-tabs">
            <div 
              v-for="file in files" 
              :key="file.path"
              class="file-tab"
              :class="{ active: file.path === currentFilePath }"
              @click="selectFile(file)"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" class="tab-file-icon">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" stroke="currentColor" stroke-width="2" fill="none"/>
                <polyline points="14,2 14,8 20,8" stroke="currentColor" stroke-width="2"/>
              </svg>
              <span class="tab-name">{{ file.name }}</span>
              <span
                v-if="externalChangePaths.includes(file.path)"
                class="tab-reload-dot"
                title="文件已在磁盘上被修改"
              />
              <button class="tab-close" @click.stop="closeFile(file)" title="关闭文件">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none">
                  <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
                  <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
                </svg>
              </button>
            </div>
          </div>
          <div
            v-if="showUnifiedHeader"
            ref="toolbarHostRef"
            class="toolbar-host"
            :class="{ collapsed: toolbarCollapsed }"
          >
            <button
              class="toolbar-toggle-btn"
              @click.stop="toggleToolbar"
              :title="toolbarCollapsed ? '显示工具栏' : '隐藏工具栏'"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                <polyline
                  :points="toolbarCollapsed ? '9,6 15,12 9,18' : '15,6 9,12 15,18'"
                  stroke="currentColor"
                  stroke-width="2"
                />
              </svg>
            </button>
          </div>
        </div>
        
        <div class="editor-content" v-if="currentFile" ref="editorContentRef">
          <Editor 
            :value="markdown" 
            :plugins="plugins"
            @change="handleChange"
            :locale="locale"
            mode="split"
            :class="['bytemd-editor-wrapper', `markfly-layout-${editorLayout}`]"
          />
        </div>
        
        <!-- 欢迎界面（启动加载待打开文件时不显示，避免闪屏） -->
        <div class="welcome-screen" v-else-if="!isBootstrapping">
          <div class="welcome-content">
            <div class="welcome-header">
              <svg width="64" height="64" viewBox="0 0 24 24" fill="none" class="welcome-icon">
                <path d="M3 3h18v18H3V3zm2 2v14h14V5H5z" fill="currentColor"/>
                <path d="M7 7h10v2H7V7zm0 4h10v2H7v-2zm0 4h7v2H7v-2z" fill="currentColor"/>
              </svg>
              <h1 class="welcome-title">欢迎使用 MarkFly</h1>
              <p class="welcome-subtitle">强大的 Markdown 编辑器，让创作更简单</p>
            </div>
            
            <div class="welcome-tips">
              <h3>快速开始</h3>
              <ul>
                <li>使用 <kbd>Ctrl/Cmd + N</kbd> 快速新建文件</li>
                <li>使用 <kbd>Ctrl/Cmd + O</kbd> 打开文件</li>
                <li>支持实时预览和语法高亮</li>
                <li>内置数学公式和图表支持</li>
                <li>使用 <kbd>Ctrl/Cmd + B</kbd> 或视图菜单显示/隐藏侧边栏</li>
                <li>点击左侧文件树或顶部标签栏切换文档</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
      

    </div>

    <!-- 状态栏（合并 ByteMD 内置状态，避免双行重复） -->
    <div class="app-footer">
      <div class="footer-left">
        <button class="settings-btn" @click="openSettings" title="设置">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
            <path d="M12 1v6m0 6v6m11-7h-6m-6 0H1m17-4a4 4 0 0 1-8 0 4 4 0 0 1 8 0zM7 16a4 4 0 0 1-8 0 4 4 0 0 1 8 0z" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
        <span class="status-item">Ln {{ currentLine }}, Col {{ currentColumn }}</span>
        <span class="status-sep">·</span>
        <span class="status-item">{{ lineCount }} 行</span>
        <span class="status-item">{{ wordCount }} 词</span>
        <span class="status-item">{{ markdown.length }} 字符</span>
      </div>
      <div class="footer-right">
        <span class="status-item">Markdown</span>
        <span class="status-sep">·</span>
        <span class="status-item">UTF-8</span>
        <button class="theme-toggle-btn" @click="themeStore.toggleTheme" :title="getThemeTooltip()">
          <svg v-if="themeStore.currentTheme === 'light'" width="12" height="12" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="5" stroke="currentColor" stroke-width="2"/>
            <line x1="12" y1="1" x2="12" y2="3" stroke="currentColor" stroke-width="2"/>
            <line x1="12" y1="21" x2="12" y2="23" stroke="currentColor" stroke-width="2"/>
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" stroke="currentColor" stroke-width="2"/>
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" stroke="currentColor" stroke-width="2"/>
            <line x1="1" y1="12" x2="3" y2="12" stroke="currentColor" stroke-width="2"/>
            <line x1="21" y1="12" x2="23" y2="12" stroke="currentColor" stroke-width="2"/>
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" stroke="currentColor" stroke-width="2"/>
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" stroke="currentColor" stroke-width="2"/>
          </svg>
          <svg v-else width="12" height="12" viewBox="0 0 24 24" fill="none">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" stroke="currentColor" stroke-width="2" fill="none"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- 设置面板 -->
    <SettingsPanel :visible="showSettings" @close="showSettings = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { Editor } from '@bytemd/vue-next'
import FileTree from './components/FileTree.vue'

import SettingsPanel from './components/SettingsPanel.vue'
import { sampleFiles, type FileItem } from './data/sampleFiles'
import { useThemeStore } from './stores/theme'

// 导入 ByteMD 插件
import gfm from '@bytemd/plugin-gfm'
import highlight from '@bytemd/plugin-highlight'
import math from '@bytemd/plugin-math'
import mermaid from '@bytemd/plugin-mermaid'
import mediumZoom from '@bytemd/plugin-medium-zoom'

// 导入 Tauri 相关模块
import { open, save, ask } from '@tauri-apps/plugin-dialog'
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

// 主题管理
const themeStore = useThemeStore()

const SIDEBAR_STORAGE_KEY = 'markfly-sidebar-collapsed'
const SIDEBAR_BTN_TOP_KEY = 'markfly-sidebar-btn-top'
const TOOLBAR_STORAGE_KEY = 'markfly-toolbar-collapsed'
const EDITOR_MODE_STORAGE_KEY = 'markfly-editor-mode'
const SIDEBAR_WIDTH = 280
const SIDEBAR_BTN_HEIGHT = 48

type EditorLayoutMode = 'preview-only' | 'split' | 'tab'

const loadEditorLayout = (): EditorLayoutMode => {
  const stored = localStorage.getItem(EDITOR_MODE_STORAGE_KEY)
  // 兼容旧版无效的 'preview' 值
  if (stored === 'preview' || stored === 'preview-only') return 'preview-only'
  if (stored === 'split' || stored === 'tab') return stored
  return 'preview-only'
}

const loadSidebarBtnTop = (): number | null => {
  const stored = localStorage.getItem(SIDEBAR_BTN_TOP_KEY)
  if (!stored || stored === 'center') return null
  const value = Number.parseInt(stored, 10)
  if (!Number.isNaN(value) && value >= 0) return value
  return null
}

const getCenteredSidebarBtnTop = () => {
  const bodyHeight = appBodyRef.value?.clientHeight ?? 600
  return Math.max(0, (bodyHeight - SIDEBAR_BTN_HEIGHT) / 2)
}

// 响应式数据
const sidebarCollapsed = ref(localStorage.getItem(SIDEBAR_STORAGE_KEY) !== 'false')
const isBootstrapping = ref(true)
const sidebarBtnTop = ref<number | null>(loadSidebarBtnTop())
const appBodyHeight = ref(0)
const isDraggingSidebarBtn = ref(false)
const appBodyRef = ref<HTMLElement | null>(null)
let sidebarBtnDragStartY = 0
let sidebarBtnDragStartTop = 0
let sidebarBtnDidDrag = false
const toolbarCollapsed = ref(localStorage.getItem(TOOLBAR_STORAGE_KEY) === 'true')
const files = ref<FileItem[]>([])
const currentFile = ref<FileItem | null>(null)
const markdown = ref('')
const isModified = ref(false)
const editorLayout = ref<EditorLayoutMode>(loadEditorLayout())
const currentLine = ref(1)
const currentColumn = ref(1)
const showSettings = ref(false)
const editorContentRef = ref<HTMLElement | null>(null)
const toolbarHostRef = ref<HTMLElement | null>(null)
let toolbarObserver: MutationObserver | null = null
const externalChangePaths = ref<string[]>([])
const watchSuppressUntil = new Map<string, number>()
let unlistenFileChanged: UnlistenFn | null = null
let reloadPromptOpen = false

const markExternalChange = (filePath: string) => {
  if (!externalChangePaths.value.includes(filePath)) {
    externalChangePaths.value = [...externalChangePaths.value, filePath]
  }
}

const clearExternalChange = (filePath: string) => {
  if (externalChangePaths.value.includes(filePath)) {
    externalChangePaths.value = externalChangePaths.value.filter((path) => path !== filePath)
  }
}

// 计算属性
const currentFilePath = computed(() => currentFile.value?.path ?? '')
const showUnifiedHeader = computed(() => files.value.length > 0 && !!currentFile.value)

const resolvedSidebarBtnTop = computed(() => {
  void appBodyHeight.value
  if (sidebarBtnTop.value !== null) return sidebarBtnTop.value
  return getCenteredSidebarBtnTop()
})

const sidebarToggleBtnStyle = computed(() => ({
  top: `${resolvedSidebarBtnTop.value}px`,
  left: sidebarCollapsed.value ? '0px' : `${SIDEBAR_WIDTH}px`,
}))

const mountToolbarToHeader = () => {
  if (!showUnifiedHeader.value) return

  const host = toolbarHostRef.value
  const root = editorContentRef.value
  if (!host || !root) return

  const toolbar = root.querySelector('.bytemd .bytemd-toolbar') as HTMLElement | null
  if (!toolbar || toolbar.parentElement === host) return

  host.querySelectorAll('.bytemd-toolbar').forEach((node) => node.remove())
  host.appendChild(toolbar)
}

const scheduleToolbarMount = () => {
  nextTick(() => {
    mountToolbarToHeader()
    requestAnimationFrame(mountToolbarToHeader)
    for (const delay of [50, 150, 300, 600, 1200]) {
      setTimeout(mountToolbarToHeader, delay)
    }
  })
}

const setupToolbarObserver = () => {
  toolbarObserver?.disconnect()
  toolbarObserver = null

  const root = editorContentRef.value
  if (!root) return

  toolbarObserver = new MutationObserver(() => mountToolbarToHeader())
  toolbarObserver.observe(root, { childList: true, subtree: true })
}

const wordCount = computed(() => {
  return markdown.value.trim().split(/\s+/).filter((word: string) => word.length > 0).length
})

const lineCount = computed(() => {
  if (!markdown.value) return 0
  return markdown.value.split('\n').length
})

// ByteMD 插件配置
const plugins = [
  gfm(),
  highlight(),
  math(),
  mermaid(),
  mediumZoom()
]

// 本地化配置（ByteMD 使用顶层 locale 键）
const locale = {
  bold: '粗体',
  italic: '斜体',
  quote: '引用',
  link: '链接',
  image: '图片',
  code: '代码',
  codeBlock: '代码块',
  ul: '无序列表',
  ol: '有序列表',
  h1: '一级标题',
  h2: '二级标题',
  h3: '三级标题',
  table: '表格',
  hr: '分割线',
  fullscreen: '全屏',
  source: '源码模式',
  write: 'Write',
  preview: 'Preview',
  writeOnly: '仅编辑',
  previewOnly: '仅预览',
  exitWriteOnly: '退出仅编辑',
  exitPreviewOnly: '退出仅预览',
  toc: '目录',
  help: '帮助',
  closeToc: '关闭目录',
  closeHelp: '关闭帮助'
}

// 方法
const handleChange = (value: string) => {
  markdown.value = value
  if (currentFile.value) {
    currentFile.value.content = value
  }
  isModified.value = true
  
  // 更新光标位置（简化版本）
  const lines = value.split('\n')
  currentLine.value = lines.length
  currentColumn.value = lines[lines.length - 1].length + 1
}

const selectFile = async (file: FileItem) => {
  // 保存当前文件的修改
  if (isModified.value && currentFile.value) {
    currentFile.value.content = markdown.value
  }
  
  // 切换到新文件
  currentFile.value = file
  markdown.value = file.content
  isModified.value = false
  await checkPendingExternalChange(file)
}

const createNewFile = () => {
  const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-')
  const newFile: FileItem = {
    name: `新建文档_${timestamp}.md`,
    path: `新建文档_${timestamp}.md`,
    content: `# 新建文档

欢迎使用 MarkFly！开始你的创作之旅...

## 快速开始

这是一个全新的 Markdown 文档。你可以：

### 基础格式
- 使用 **粗体** 和 *斜体* 强调文本
- 创建 [链接](https://example.com) 和引用
- 添加 \`行内代码\` 和代码块

### 列表和表格
1. 有序列表项目一
2. 有序列表项目二
   - 无序子项目
   - 另一个子项目

| 列标题1 | 列标题2 | 列标题3 |
|---------|---------|---------|
| 数据1   | 数据2   | 数据3   |
| 数据4   | 数据5   | 数据6   |

### 代码示例
\`\`\`javascript
// JavaScript 代码示例
function greet(name) {
    return \`Hello, \${name}! Welcome to MarkFly!\`;
}

console.log(greet('World'));
\`\`\`

### 数学公式
行内公式：$E = mc^2$

块级公式：
$$
\\sum_{i=1}^{n} x_i = x_1 + x_2 + \\cdots + x_n
$$

### 引用
> 这是一个引用块。你可以在这里添加重要的引用内容或者注释。

---

💡 **提示**: 使用左侧的文件树管理你的文档，右侧的目录可以快速导航到不同的章节。

🎯 **开始创作**: 删除这些示例内容，开始编写你自己的文档吧！`
  }
  
  files.value.push(newFile)
  selectFile(newFile)
}

const closeFile = (file: FileItem) => {
  const index = files.value.findIndex(f => f.path === file.path)
  if (index > -1) {
    files.value.splice(index, 1)
    clearExternalChange(file.path)
    watchSuppressUntil.delete(file.path)
    
    // 如果关闭的是当前文件，切换到其他文件
    if (file.path === currentFilePath.value) {
      if (files.value.length > 0) {
        const nextFile = files.value[Math.max(0, index - 1)]
        selectFile(nextFile)
      } else {
        currentFile.value = null
        markdown.value = ''
      }
    }
  }
}

const openFolder = () => {
  // 这里可以实现打开文件夹的功能
  console.log('Open folder')
}

const openSettings = () => {
  showSettings.value = true
}

const toggleSidebar = () => {
  sidebarCollapsed.value = !sidebarCollapsed.value
  localStorage.setItem(SIDEBAR_STORAGE_KEY, String(sidebarCollapsed.value))
}

const clampSidebarBtnTop = (top: number) => {
  const maxTop = Math.max(0, (appBodyRef.value?.clientHeight ?? 600) - SIDEBAR_BTN_HEIGHT)
  return Math.max(0, Math.min(maxTop, top))
}

const onSidebarBtnPointerDown = (event: PointerEvent) => {
  sidebarBtnDidDrag = false
  isDraggingSidebarBtn.value = true
  sidebarBtnDragStartY = event.clientY
  sidebarBtnDragStartTop = resolvedSidebarBtnTop.value
  ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
}

const onSidebarBtnPointerMove = (event: PointerEvent) => {
  if (!isDraggingSidebarBtn.value) return
  const deltaY = event.clientY - sidebarBtnDragStartY
  if (Math.abs(deltaY) > 3) sidebarBtnDidDrag = true
  sidebarBtnTop.value = clampSidebarBtnTop(sidebarBtnDragStartTop + deltaY)
}

const onSidebarBtnPointerUp = (event: PointerEvent) => {
  if (!isDraggingSidebarBtn.value) return
  isDraggingSidebarBtn.value = false
  ;(event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId)
  if (sidebarBtnTop.value !== null) {
    localStorage.setItem(SIDEBAR_BTN_TOP_KEY, String(sidebarBtnTop.value))
  }
}

const onSidebarBtnClick = () => {
  if (sidebarBtnDidDrag) return
  toggleSidebar()
}

const clampSidebarBtnPosition = () => {
  appBodyHeight.value = appBodyRef.value?.clientHeight ?? 0
  if (sidebarBtnTop.value !== null) {
    sidebarBtnTop.value = clampSidebarBtnTop(sidebarBtnTop.value)
  }
}

const toggleToolbar = () => {
  toolbarCollapsed.value = !toolbarCollapsed.value
  localStorage.setItem(TOOLBAR_STORAGE_KEY, String(toolbarCollapsed.value))
}

const togglePreview = () => {
  const modes: EditorLayoutMode[] = ['preview-only', 'split', 'tab']
  const currentIndex = modes.indexOf(editorLayout.value)
  editorLayout.value = modes[(currentIndex + 1) % modes.length]
  localStorage.setItem(EDITOR_MODE_STORAGE_KEY, editorLayout.value)
}

const handleToolbarLayoutClick = (event: MouseEvent) => {
  const icon = (event.target as HTMLElement).closest('.bytemd-toolbar-right .bytemd-toolbar-icon')
  if (!icon) return

  const icons = icon.parentElement?.querySelectorAll('.bytemd-toolbar-icon')
  if (!icons) return

  const index = Array.from(icons).indexOf(icon)
  // 右侧工具栏：目录、帮助、仅编辑、仅预览、全屏、源码
  if (index === 2) {
    editorLayout.value = 'tab'
    localStorage.setItem(EDITOR_MODE_STORAGE_KEY, 'tab')
  } else if (index === 3) {
    editorLayout.value = 'preview-only'
    localStorage.setItem(EDITOR_MODE_STORAGE_KEY, 'preview-only')
  }
}

const getThemeTooltip = () => {
  const modeMap = {
    light: '切换到深色模式',
    dark: '切换到自动模式',
    auto: '切换到浅色模式'
  }
  return modeMap[themeStore.mode] || '切换主题'
}

const getFileNameFromPath = (filePath: string) =>
  filePath.split(/[/\\]/).pop() || '未命名.md'

const isDiskFilePath = (filePath: string) =>
  /[/\\]/.test(filePath) || /^[A-Za-z]:/.test(filePath)

const markFileSavedOnDisk = (filePath: string) => {
  watchSuppressUntil.set(filePath, Date.now() + 1500)
}

const isWatchSuppressed = (filePath: string) => {
  const until = watchSuppressUntil.get(filePath)
  return until !== undefined && Date.now() < until
}

const syncDiskFileWatches = async () => {
  const paths = files.value.map((file) => file.path).filter(isDiskFilePath)
  try {
    await invoke('sync_file_watches', { paths })
  } catch (error) {
    console.error('同步文件监听失败:', error)
  }
}

const reloadFileFromDisk = async (file: FileItem, newContent?: string) => {
  const content = newContent ?? await readTextFile(file.path)
  file.content = content
  clearExternalChange(file.path)

  if (currentFile.value?.path === file.path) {
    markdown.value = content
    isModified.value = false
  }
}

const promptReloadFile = async (file: FileItem, newContent: string) => {
  if (reloadPromptOpen) {
    markExternalChange(file.path)
    return
  }

  reloadPromptOpen = true
  try {
    const message = isModified.value && currentFile.value?.path === file.path
      ? `「${file.name}」已在磁盘上被其他程序修改。\n\n是否重新加载？未保存的更改将会丢失。`
      : `「${file.name}」已在磁盘上被其他程序修改。\n\n是否重新加载？`

    const reload = await ask(message, {
      title: 'MarkFly',
      kind: 'warning',
      okLabel: '重新加载',
      cancelLabel: '取消'
    })

    if (reload) {
      await reloadFileFromDisk(file, newContent)
    } else {
      markExternalChange(file.path)
    }
  } finally {
    reloadPromptOpen = false
  }
}

const handleExternalFileChange = async (filePath: string) => {
  if (isWatchSuppressed(filePath)) {
    return
  }

  const file = files.value.find((item) => item.path === filePath)
  if (!file) {
    return
  }

  let newContent: string
  try {
    newContent = await readTextFile(filePath)
  } catch (error) {
    console.error('读取外部变更失败:', error)
    return
  }

  if (newContent === file.content) {
    return
  }

  if (currentFile.value?.path === filePath) {
    await promptReloadFile(file, newContent)
  } else {
    markExternalChange(filePath)
  }
}

const checkPendingExternalChange = async (file: FileItem) => {
  if (!externalChangePaths.value.includes(file.path)) {
    return
  }

  let newContent: string
  try {
    newContent = await readTextFile(file.path)
  } catch (error) {
    console.error('读取外部变更失败:', error)
    return
  }

  if (newContent === file.content) {
    clearExternalChange(file.path)
    watchSuppressUntil.delete(file.path)
    return
  }

  await promptReloadFile(file, newContent)
}

const loadWelcomeSample = () => {
  if (files.value.length > 0) {
    return
  }

  files.value = [...sampleFiles]
  if (sampleFiles.length > 0) {
    selectFile(sampleFiles[0])
  }
}

const openFileFromPath = async (filePath: string) => {
  try {
    const existing = files.value.find((file) => file.path === filePath)
    if (existing) {
      selectFile(existing)
      return
    }

    const content = await readTextFile(filePath)
    const newFile: FileItem = {
      name: getFileNameFromPath(filePath),
      path: filePath,
      content
    }

    files.value.push(newFile)
    selectFile(newFile)
  } catch (error) {
    console.error('打开文件失败:', error)
  }
}

const openFilePaths = async (paths: string[]) => {
  if (paths.length === 0) {
    return
  }

  for (const filePath of paths) {
    await openFileFromPath(filePath)
  }
}

// 新增：打开本地 Markdown 文件
const openLocalFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Markdown Files',
        extensions: ['md', 'markdown', 'txt']
      }]
    })
    
    if (selected) {
      await openFileFromPath(selected as string)
    }
  } catch (error) {
    console.error('打开文件失败:', error)
  }
}

// 新增：保存 Markdown 文件到本地
const saveFileToLocal = async () => {
  if (!currentFile.value) return
  
  try {
    // 如果文件已经有路径，直接保存
    if (currentFile.value.path && currentFile.value.path !== currentFile.value.name) {
      markFileSavedOnDisk(currentFile.value.path)
      await writeTextFile(currentFile.value.path, markdown.value)
      currentFile.value.content = markdown.value
      isModified.value = false
      console.log('文件已保存:', currentFile.value.path)
    } else {
      // 弹出保存对话框
      const filePath = await save({
        filters: [{
          name: 'Markdown Files',
          extensions: ['md']
        }]
      })
      
      if (filePath) {
        markFileSavedOnDisk(filePath)
        await writeTextFile(filePath, markdown.value)
        currentFile.value.content = markdown.value
        // 更新文件路径
        currentFile.value.path = filePath
        currentFile.value.name = getFileNameFromPath(filePath)
        isModified.value = false
        console.log('文件已保存到:', filePath)
      }
    }
  } catch (error) {
    console.error('保存文件失败:', error)
  }
}

// 新增：另存为功能
const saveFileAs = async () => {
  if (!currentFile.value) return
  
  try {
    // 弹出保存对话框，使用当前文件名作为默认文件名
    const defaultPath = currentFile.value.path && currentFile.value.path !== currentFile.value.name 
      ? currentFile.value.path 
      : currentFile.value.name || '未命名.md'
      
    const filePath = await save({
      defaultPath: defaultPath,
      filters: [{
        name: 'Markdown Files',
        extensions: ['md']
      }]
    })
    
    if (filePath) {
      markFileSavedOnDisk(filePath)
      await writeTextFile(filePath, markdown.value)
      currentFile.value.content = markdown.value
      // 更新文件路径
      currentFile.value.path = filePath
      currentFile.value.name = getFileNameFromPath(filePath)
      isModified.value = false
      console.log('文件已另存为:', filePath)
    }
  } catch (error) {
    console.error('另存为文件失败:', error)
  }
}

// 新增：处理键盘快捷键
const handleKeyDown = (event: KeyboardEvent) => {
  // Ctrl/Cmd + B 显示/隐藏侧边栏（捕获阶段优先于编辑器粗体）
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'b' && !event.shiftKey && !event.altKey) {
    event.preventDefault()
    event.stopPropagation()
    toggleSidebar()
    return
  }
  // Ctrl/Cmd + O 打开文件
  if ((event.ctrlKey || event.metaKey) && event.key === 'o') {
    event.preventDefault()
    openLocalFile()
  }
  // Ctrl/Cmd + S 保存文件
  else if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault()
    saveFileToLocal()
  }
  // Ctrl/Cmd + N 新建文件
  else if ((event.ctrlKey || event.metaKey) && event.key === 'n') {
    event.preventDefault()
    createNewFile()
  }
}

// 初始化主题
onMounted(async () => {
  themeStore.initTheme()
  
  // 添加键盘事件监听器（capture 确保 Ctrl+B 不被编辑器拦截）
  window.addEventListener('keydown', handleKeyDown, true)

  try {
    const pendingFiles = await invoke<string[]>('get_pending_open_files')
    if (pendingFiles.length > 0) {
      await openFilePaths(pendingFiles)
    } else if (files.value.length === 0) {
      loadWelcomeSample()
    }
  } catch (error) {
    console.error('启动时打开文件失败:', error)
    if (files.value.length === 0) {
      loadWelcomeSample()
    }
  } finally {
    isBootstrapping.value = false
  }

  await listen<string[]>('open-file-path', async (event) => {
    await openFilePaths(event.payload)
  })
  
  // 监听菜单事件
  listen('menu', (event) => {
    console.log('收到菜单事件:', event.payload);
    switch (event.payload) {
      case 'new-file':
        console.log('新建文件');
        createNewFile()
        break
      case 'open-file':
        console.log('打开文件');
        openLocalFile()
        break
      case 'save-file':
        console.log('保存文件');
        saveFileToLocal()
        break
      case 'save-file-as':
        console.log('另存为');
        saveFileAs()
        break
      case 'toggle-sidebar':
        toggleSidebar()
        break
      case 'toggle-preview':
        togglePreview()
        break
      case 'toggle-theme':
        themeStore.toggleTheme()
        break
      default:
        console.log('未知菜单事件:', event.payload);
    }
  }).then(() => {
    console.log('菜单事件监听器已注册');
  }).catch((error) => {
    console.error('注册菜单事件监听器失败:', error);
  });

  unlistenFileChanged = await listen<string>('file-changed', async (event) => {
    await handleExternalFileChange(event.payload)
  })

  await syncDiskFileWatches()
  scheduleToolbarMount()
  nextTick(setupToolbarObserver)

  clampSidebarBtnPosition()
  window.addEventListener('resize', clampSidebarBtnPosition)
})

watch(
  () => files.value.map((file) => file.path).join('\0'),
  () => {
    void syncDiskFileWatches()
  }
)

watch(showUnifiedHeader, (visible) => {
  if (visible) {
    scheduleToolbarMount()
    nextTick(setupToolbarObserver)
  } else {
    toolbarObserver?.disconnect()
    toolbarObserver = null
  }
})

watch(editorLayout, scheduleToolbarMount)

watch(currentFilePath, () => {
  scheduleToolbarMount()
  nextTick(setupToolbarObserver)
})

// 清理事件监听器
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown, true)
  window.removeEventListener('resize', clampSidebarBtnPosition)
  unlistenFileChanged?.()
  toolbarObserver?.disconnect()
  void invoke('sync_file_watches', { paths: [] })
})
</script>

<style scoped>
.markfly-app {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  color: var(--text-primary);
  transition: background-color 0.3s ease, color 0.3s ease;
}

/* 头部区域样式 */
.app-header {
  display: flex;
  flex-direction: column;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

/* 文件标签栏 */
.file-tabs {
  background: var(--tab-bar-bg);
  border-bottom: 1px solid var(--border-color);
  padding: 0 8px;
  display: flex;
  align-items: flex-end;
  gap: 0;
  height: 35px;
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: thin;
  box-sizing: border-box;
  position: relative;
  flex-shrink: 0;
  z-index: 10;
}

.file-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 10px 0 12px;
  height: 100%;
  max-width: 220px;
  min-width: 0;
  background: transparent;
  border-radius: 0;
  cursor: pointer;
  font-size: 12px;
  color: var(--tab-inactive-fg);
  transition: background-color 0.15s ease, color 0.15s ease;
  border: none;
  border-top: 2px solid transparent;
  flex-shrink: 0;
}

.file-tab:hover {
  background: var(--hover-bg);
  color: var(--tab-active-fg);
}

.file-tab.active {
  background: var(--tab-active-bg);
  color: var(--tab-active-fg);
  border-top-color: var(--tab-active-border);
}

.tab-file-icon {
  flex-shrink: 0;
  color: var(--text-secondary);
}

.file-tab.active .tab-file-icon {
  color: var(--tab-active-fg);
}

.tab-name {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab-reload-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #f59e0b;
  flex-shrink: 0;
}

.tab-close {
  width: 16px;
  height: 16px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s ease, background-color 0.15s ease, color 0.15s ease;
  flex-shrink: 0;
}

.file-tab:hover .tab-close,
.file-tab.active .tab-close {
  opacity: 0.65;
}

.file-tab.active .tab-close {
  color: var(--tab-inactive-fg);
}

.tab-close:hover {
  background: var(--danger-color);
  color: white;
}

/* 主体区域 */
.app-body {
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
  position: relative;
}

.sidebar-toggle-btn {
  position: absolute;
  z-index: 30;
  width: 20px;
  height: 48px;
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab;
  touch-action: none;
  user-select: none;
  padding: 0;
  transition:
    left 0.2s ease,
    background-color 0.2s ease,
    color 0.2s ease;
}

.sidebar-toggle-btn.is-collapsed {
  border-left: none;
  border-radius: 0 6px 6px 0;
  transform: none;
}

.sidebar-toggle-btn.is-expanded {
  border-right: none;
  border-radius: 6px 0 0 6px;
  transform: translateX(-100%);
}

.sidebar-toggle-btn.dragging {
  cursor: grabbing;
  transition: left 0.2s ease;
}

.sidebar-toggle-btn:hover {
  background: var(--hover-bg);
  color: var(--text-primary);
}

.editor-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--bg-primary);
}

.editor-unified-header.is-merged {
  display: flex;
  align-items: stretch;
  height: 35px;
  flex-shrink: 0;
  background: var(--tab-bar-bg);
  border-bottom: 1px solid var(--border-color);
}

.editor-unified-header.is-merged .file-tabs {
  flex: 0 1 auto;
  max-width: 45%;
  min-width: 0;
  height: 100%;
  border-bottom: none;
  background: transparent;
  padding-right: 4px;
}

.toolbar-host {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 2px;
}

.toolbar-host.collapsed {
  flex: 0;
}

.toolbar-toggle-btn {
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
  flex-shrink: 0;
  transition: background-color 0.2s ease, color 0.2s ease;
}

.toolbar-toggle-btn:hover {
  background: var(--hover-bg);
  color: var(--text-primary);
}

.toolbar-host.collapsed :deep(.bytemd-toolbar) {
  display: none !important;
}

.toolbar-host :deep(.bytemd-toolbar) {
  flex: 1;
  min-width: 0;
  width: 100%;
  border-bottom: none !important;
  background: transparent !important;
  padding: 0 8px !important;
  min-height: 35px !important;
  height: 35px !important;
}

.toolbar-host :deep(.bytemd-toolbar-left),
.toolbar-host :deep(.bytemd-toolbar-right) {
  flex-wrap: nowrap;
}

.editor-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* 添加编辑器包装器样式 */
.bytemd-editor-wrapper {
  height: 100%;
  width: 100%;
}

/* split 模式：隐藏 Write/Preview 文字标签，使用右侧图标 */
.bytemd-editor-wrapper :deep(.bytemd-toolbar-tab) {
  display: none !important;
}

/* 隐藏 ByteMD 内置状态栏，统一使用应用底部状态栏 */
.bytemd-editor-wrapper :deep(.bytemd-status) {
  display: none !important;
}

.markfly-layout-preview-only :deep(.bytemd-editor) {
  display: none !important;
}

.markfly-layout-preview-only :deep(.bytemd-preview) {
  display: block !important;
  width: 100% !important;
}

.markfly-layout-tab :deep(.bytemd-preview) {
  display: none !important;
}

.markfly-layout-tab :deep(.bytemd-editor) {
  display: block !important;
  width: 100% !important;
}

.app-footer {
  height: 22px;
  background: var(--status-bar-bg);
  color: var(--status-bar-fg);
  border-top: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px;
  font-size: 11px;
  flex-shrink: 0;
}

.footer-left,
.footer-right {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.status-item {
  white-space: nowrap;
  font-size: 11px;
}

.status-sep {
  opacity: 0.55;
  user-select: none;
}

.settings-btn,
.theme-toggle-btn {
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--status-bar-fg);
  cursor: pointer;
  transition: all 0.2s ease;
  margin-right: 8px;
  opacity: 0.85;
}

.settings-btn:hover,
.theme-toggle-btn:hover {
  background: var(--hover-bg);
  color: var(--text-primary);
  opacity: 1;
}

.settings-btn {
  margin-right: 12px;
}

.theme-toggle-btn {
  margin-left: 8px;
  margin-right: 0;
}
</style>
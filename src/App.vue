<template>
  <div class="markfly-app">
    <!-- 主编辑区域 -->
    <div class="app-body" ref="appBodyRef">
      <!-- 左侧文件树 -->
      <FileTree 
        v-if="showDeferredChrome"
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
              @contextmenu.prevent="openTabContextMenu($event, file)"
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
            <div ref="toolbarActionsRef" class="toolbar-actions">
              <div v-if="usePreviewViewer && !toolbarCollapsed" class="preview-layout-toolbar">
                <button
                  type="button"
                  class="layout-mode-btn"
                  :class="{ 'is-active': previewTocVisible }"
                  :title="previewTocVisible ? '关闭目录' : '目录'"
                  @click.stop="togglePreviewToc"
                >
                  <span class="layout-mode-icon" v-html="toolbarIcons.toc" />
                </button>
                <button
                  type="button"
                  class="layout-mode-btn"
                  title="分屏"
                  @click.stop="setEditorLayout('split')"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                    <rect x="3" y="4" width="8" height="16" stroke="currentColor" stroke-width="2"/>
                    <rect x="13" y="4" width="8" height="16" stroke="currentColor" stroke-width="2"/>
                  </svg>
                </button>
                <button
                  type="button"
                  class="layout-mode-btn"
                  title="仅编辑"
                  @click.stop="setEditorLayout('tab')"
                >
                  <span class="layout-mode-icon" v-html="toolbarIcons.leftExpand" />
                </button>
                <button type="button" class="layout-mode-btn is-active" title="仅预览" disabled>
                  <span class="layout-mode-icon" v-html="toolbarIcons.rightExpand" />
                </button>
              </div>
            </div>
            <button
              class="toolbar-toggle-btn"
              @click.stop="toggleToolbar"
              :title="toolbarCollapsed ? '显示工具栏' : '隐藏工具栏'"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                <polyline
                  :points="toolbarCollapsed ? '15,6 9,12 15,18' : '9,6 15,12 9,18'"
                  stroke="currentColor"
                  stroke-width="2"
                />
              </svg>
            </button>
          </div>
        </div>

        <!-- 外部变更：整条可点刷新，不抢焦点 -->
        <div
          v-if="showExternalChangeBanner"
          class="external-change-banner"
          role="button"
          tabindex="0"
          title="点击重新加载"
          @click="reloadCurrentFileFromDisk"
          @keydown.enter.prevent="reloadCurrentFileFromDisk"
        >
          <svg class="external-change-banner-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M21 12a9 9 0 1 1-2.64-6.36"
              stroke="currentColor"
              stroke-width="1.75"
              stroke-linecap="round"
            />
            <polyline
              points="21,3 21,9 15,9"
              stroke="currentColor"
              stroke-width="1.75"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          <span class="external-change-banner-text">
            <span class="external-change-banner-label">磁盘文件已更新</span>
            <span class="external-change-banner-sep">·</span>
            <span class="external-change-banner-hint">
              {{ isModified ? '重新加载将丢弃未保存更改' : '点击重新加载' }}
            </span>
          </span>
          <span class="external-change-banner-arrow" aria-hidden="true">→</span>
          <button
            type="button"
            class="external-change-banner-close"
            title="关闭提示"
            aria-label="关闭提示"
            @click.stop="dismissExternalChangeBanner"
          >
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
              <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
            </svg>
          </button>
        </div>
        
        <div class="editor-main">
          <div
            class="editor-content"
            ref="editorContentRef"
            :class="{ 'is-background': !currentFile }"
          >
            <div v-if="usePreviewViewer" class="preview-viewer-shell">
              <div
                ref="previewBodyRef"
                class="markfly-preview-viewer markdown-body"
                v-html="previewHtml"
              />
              <TableOfContents
                v-if="previewTocVisible"
                :content="previewSource"
                :scroll-root="previewBodyRef"
                @close="previewTocVisible = false"
              />
            </div>
            <component
              v-if="!usePreviewViewer && EditorComponent"
              :is="EditorComponent"
              :value="markdown"
              :plugins="plugins"
              @change="handleChange"
              :locale="locale"
              mode="split"
              :class="['bytemd-editor-wrapper', `markfly-layout-${editorLayout}`]"
            />
          </div>

          <!-- 欢迎界面（启动加载待打开文件时不显示，避免闪屏） -->
          <div class="welcome-screen" v-show="!currentFile && !isBootstrapping">
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
    </div>

    <!-- 状态栏（合并 ByteMD 内置状态，避免双行重复） -->
    <div class="app-footer" v-if="showDeferredChrome">
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

    <!-- 标签右键菜单（挂到 body，避免被顶栏裁剪） -->
    <Teleport to="body">
      <div
        v-if="tabContextMenu"
        class="tab-context-menu"
        :style="{ left: `${tabContextMenu.x}px`, top: `${tabContextMenu.y}px` }"
        @click.stop
        @contextmenu.prevent
      >
        <button
          type="button"
          class="tab-context-item"
          title="复制文件名"
          @click="copyTabFileName"
        >
          复制文件名
        </button>
        <button
          type="button"
          class="tab-context-item"
          :disabled="!tabContextMenu.isDiskFile"
          :title="tabContextMenu.isDiskFile ? '复制完整文件路径' : '内置示例或未保存文件没有磁盘路径'"
          @click="copyTabFilePath"
        >
          复制文件路径
        </button>
        <button
          type="button"
          class="tab-context-item"
          :disabled="!tabContextMenu.isDiskFile"
          :title="tabContextMenu.isDiskFile ? '在资源管理器中显示并选中该文件' : '内置示例或未保存文件没有磁盘路径'"
          @click="revealTabFileInExplorer"
        >
          打开文件目录
        </button>
        <button
          type="button"
          class="tab-context-item"
          :disabled="!tabContextMenu.isDiskFile"
          :title="tabContextMenu.isDiskFile ? '从磁盘重新读取并刷新显示' : '内置示例或未保存文件无法刷新'"
          @click="refreshTabFile"
        >
          刷新
        </button>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick, shallowRef, defineAsyncComponent, type Component } from 'vue'
const FileTree = defineAsyncComponent(() => import('./components/FileTree.vue'))
const SettingsPanel = defineAsyncComponent(() => import('./components/SettingsPanel.vue'))
const TableOfContents = defineAsyncComponent(() => import('./components/TableOfContents.vue'))
import type { FileItem } from './data/sampleFiles'
import { useThemeStore } from './stores/theme'
import { lightMarkdownToHtml } from './utils/lightMarkdown'
import { needsFullPreview, stripFrontmatter } from './utils/markdownPreview'
import { consumeMediumPreviewPreload, startMediumPreviewPreload } from './utils/previewEngine'
import {
  allowMarkdownAssets,
  fixPreviewImages,
  rewritePreviewAssetUrls,
} from './utils/previewAssets'
import { bindPreviewImageZoom, unbindPreviewImageZoom } from './utils/previewImageZoom'
import { toolbarIcons } from './utils/toolbarIcons'
import {
  tauriInvoke,
  tauriListen,
  tauriReadTextFile,
  tauriWriteTextFile,
  tauriOpenFile,
  tauriSaveFile,
  tauriAsk,
} from './utils/tauriBridge'

import type { BytemdPlugin } from 'bytemd'

type PendingOpenFile = {
  path: string
  content: string
  truncated?: boolean
}

declare global {
  interface Window {
    __MARKFLY_BOOT__?: PendingOpenFile[]
    __MARKFLY_BOOT_DISMISSED__?: boolean
    __markflyHideBoot?: () => void
  }
}

const readPendingBootstrap = (): Promise<PendingOpenFile[]> => {
  const boot = window.__MARKFLY_BOOT__
  if (Array.isArray(boot) && boot.length > 0) {
    if (boot[0]?.content) {
      startMediumPreviewPreload(boot[0].content)
    }
    return Promise.resolve(boot)
  }
  return tauriInvoke<PendingOpenFile[]>('get_pending_open_files').catch(() => [])
}

// 应用启动后立即拉取待打开文件（与 WebView 初始化并行）
const pendingBootstrapPromise = readPendingBootstrap()

const EditorComponent = shallowRef<Component | null>(null)
let editorComponentLoading: Promise<void> | null = null

const ensureEditorComponent = async () => {
  if (EditorComponent.value) return
  if (editorComponentLoading) return editorComponentLoading

  editorComponentLoading = (async () => {
    const mod = await import('@bytemd/vue-next')
    await import('bytemd/dist/index.css')
    EditorComponent.value = mod.Editor
  })()

  return editorComponentLoading
}

// 主题管理
const themeStore = useThemeStore()

const SIDEBAR_STORAGE_KEY = 'markfly-sidebar-collapsed'
const SIDEBAR_BTN_TOP_KEY = 'markfly-sidebar-btn-top'
const TOOLBAR_STORAGE_KEY = 'markfly-toolbar-collapsed'
const PREVIEW_TOC_STORAGE_KEY = 'markfly-preview-toc-visible'
const EDITOR_MODE_STORAGE_KEY = 'markfly-editor-mode'
/** WebView 刷新后用于恢复已打开的磁盘文件列表 */
const OPEN_SESSION_KEY = 'markfly-open-session'
const SIDEBAR_WIDTH = 280
const SIDEBAR_BTN_HEIGHT = 48

type OpenSession = {
  paths: string[]
  currentPath: string
}

type TabContextMenuState = {
  x: number
  y: number
  file: FileItem
  /** 是否为真实磁盘文件（内置示例 / 未另存新建文件为 false） */
  isDiskFile: boolean
}

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
const showDeferredChrome = ref(false)
const sidebarBtnTop = ref<number | null>(loadSidebarBtnTop())
const appBodyHeight = ref(0)
const isDraggingSidebarBtn = ref(false)
const appBodyRef = ref<HTMLElement | null>(null)
let sidebarBtnDragStartY = 0
let sidebarBtnDragStartTop = 0
let sidebarBtnDidDrag = false
const toolbarCollapsed = ref(localStorage.getItem(TOOLBAR_STORAGE_KEY) === 'true')
const previewTocVisible = ref(localStorage.getItem(PREVIEW_TOC_STORAGE_KEY) === 'true')
const files = ref<FileItem[]>([])
const currentFile = ref<FileItem | null>(null)
const markdown = ref('')
const isModified = ref(false)
const editorLayout = ref<EditorLayoutMode>(loadEditorLayout())
const currentLine = ref(1)
const currentColumn = ref(1)
const showSettings = ref(false)
const editorContentRef = ref<HTMLElement | null>(null)
const previewBodyRef = ref<HTMLElement | null>(null)
const toolbarHostRef = ref<HTMLElement | null>(null)
const toolbarActionsRef = ref<HTMLElement | null>(null)
let toolbarObserver: MutationObserver | null = null
let previewViewerEffectCleanups: Array<() => void> = []
let bytemdGetProcessor: ((options: { plugins: BytemdPlugin[] }) => { processSync: (value: string) => { toString: () => string } }) | null = null
const externalChangePaths = ref<string[]>([])
/** 用户关掉顶部横幅的路径；磁盘内容再次变更时会重新显示 */
const dismissedExternalBannerPaths = ref<string[]>([])
/** 最近一次检测到的磁盘内容，用于区分「重复监听」与「又改了一版」 */
const lastExternalContents = new Map<string, string>()
const watchSuppressUntil = new Map<string, number>()
let unlistenFileChanged: (() => void) | null = null
const tabContextMenu = ref<TabContextMenuState | null>(null)
/** 防止右键抬起触发的 click 立刻关掉刚打开的菜单 */
let tabContextMenuOpenedAt = 0

const markExternalChange = (filePath: string) => {
  if (!externalChangePaths.value.includes(filePath)) {
    externalChangePaths.value = [...externalChangePaths.value, filePath]
  }
}

const clearExternalChange = (filePath: string) => {
  lastExternalContents.delete(filePath)
  if (dismissedExternalBannerPaths.value.includes(filePath)) {
    dismissedExternalBannerPaths.value = dismissedExternalBannerPaths.value.filter(
      (path) => path !== filePath
    )
  }
  if (externalChangePaths.value.includes(filePath)) {
    externalChangePaths.value = externalChangePaths.value.filter((path) => path !== filePath)
  }
}

/** 磁盘出现新一版内容时，重新打开已被关掉的横幅 */
const revealExternalChangeBanner = (filePath: string) => {
  if (!dismissedExternalBannerPaths.value.includes(filePath)) return
  dismissedExternalBannerPaths.value = dismissedExternalBannerPaths.value.filter(
    (path) => path !== filePath
  )
}

// 计算属性
const currentFilePath = computed(() => currentFile.value?.path ?? '')
const showUnifiedHeader = computed(() => files.value.length > 0 && !!currentFile.value)
const usePreviewViewer = computed(() => editorLayout.value === 'preview-only')
/** 当前文件有外部变更且未主动关闭横幅时显示 */
const showExternalChangeBanner = computed(() => {
  const path = currentFile.value?.path
  return (
    !!path &&
    externalChangePaths.value.includes(path) &&
    !dismissedExternalBannerPaths.value.includes(path)
  )
})

const clearByteMdToolbarFromHost = () => {
  toolbarActionsRef.value?.querySelectorAll('.bytemd-toolbar').forEach((node) => node.remove())
}

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
  if (!showUnifiedHeader.value || usePreviewViewer.value) return

  const actions = toolbarActionsRef.value
  const root = editorContentRef.value
  if (!actions || !root) return

  const toolbar = root.querySelector('.bytemd .bytemd-toolbar') as HTMLElement | null
  if (!toolbar || toolbar.parentElement === actions) return

  actions.querySelectorAll('.bytemd-toolbar').forEach((node) => node.remove())
  actions.appendChild(toolbar)
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

  if (usePreviewViewer.value) return

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

// ByteMD 插件：首屏为空，gfm/highlight/math/mermaid 均懒加载
const plugins = ref<BytemdPlugin[]>([])
const previewEngineReady = ref(false)
let mediumPluginsLoaded = false
let mediumPluginsLoading: Promise<void> | null = null
let heavyPluginsLoaded = false
let heavyPluginsLoading: Promise<void> | null = null
let fullPreviewProcessorReady = false

const needsMediumPlugins = (content: string) => needsFullPreview(content)

const needsHeavyPlugins = (content: string) => {
  if (!content) return false
  if (/```\s*mermaid\b/.test(content)) return true
  if (/\$\$[\s\S]+?\$\$/.test(content)) return true
  if (/(?:^|[^\\])\$(?!\$)[^\n$]+\$(?!\$)/m.test(content)) return true
  return false
}

const ensureFullPreviewProcessor = async () => {
  if (fullPreviewProcessorReady) return
  const bytemd = await import('bytemd')
  bytemdGetProcessor = bytemd.getProcessor
  fullPreviewProcessorReady = true
}

const loadMediumPlugins = async () => {
  if (mediumPluginsLoaded) return
  if (mediumPluginsLoading) return mediumPluginsLoading

  mediumPluginsLoading = (async () => {
    const preloaded = consumeMediumPreviewPreload()
    if (preloaded) {
      try {
        const modules = await preloaded
        bytemdGetProcessor = modules.getProcessor
        fullPreviewProcessorReady = true
        plugins.value = modules.createPlugins()
      } catch (error) {
        console.warn('GFM 预加载失败，改为直接加载:', error)
      }
    }

    if (!mediumPluginsLoaded && plugins.value.length === 0) {
      await ensureFullPreviewProcessor()
      const [{ default: gfm }, { default: highlight }] = await Promise.all([
        import('@bytemd/plugin-gfm'),
        import('@bytemd/plugin-highlight'),
        import('highlight.js/styles/vs.css'),
      ])
      plugins.value = [gfm(), highlight()]
    }

    mediumPluginsLoaded = true
    previewEngineReady.value = true
  })().catch((error) => {
    mediumPluginsLoading = null
    console.error('加载预览插件失败:', error)
    throw error
  })

  return mediumPluginsLoading
}

const loadHeavyPlugins = async () => {
  if (heavyPluginsLoaded) return
  if (heavyPluginsLoading) return heavyPluginsLoading

  heavyPluginsLoading = (async () => {
    await loadMediumPlugins()
    const [{ default: math }, { default: mermaid }] = await Promise.all([
      import('@bytemd/plugin-math'),
      import('@bytemd/plugin-mermaid'),
      import('katex/dist/katex.css'),
    ])
    plugins.value = [...plugins.value, math(), mermaid()]
    heavyPluginsLoaded = true
  })()

  return heavyPluginsLoading
}

const scheduleMediumPluginLoad = (content = markdown.value) => {
  if (mediumPluginsLoaded || mediumPluginsLoading) return
  if (editorLayout.value === 'preview-only' || needsMediumPlugins(content)) {
    void loadMediumPlugins()
    return
  }
  const run = () => void loadMediumPlugins()
  if (typeof requestIdleCallback === 'function') {
    requestIdleCallback(run, { timeout: 3000 })
  } else {
    setTimeout(run, 1000)
  }
}

const ensurePreviewPipeline = async () => {
  await loadMediumPlugins()
  if (needsHeavyPlugins(markdown.value)) {
    await loadHeavyPlugins()
  } else {
    scheduleHeavyPluginLoad(markdown.value)
  }
}

const scheduleHeavyPluginLoad = (content = markdown.value) => {
  if (heavyPluginsLoaded || heavyPluginsLoading) return
  if (needsHeavyPlugins(content)) {
    void loadHeavyPlugins()
    return
  }
  const run = () => void loadHeavyPlugins()
  if (typeof requestIdleCallback === 'function') {
    requestIdleCallback(run, { timeout: 4000 })
  } else {
    setTimeout(run, 1500)
  }
}

const previewSource = computed(() => stripFrontmatter(markdown.value))

const previewWaitingForPipeline = computed(
  () => usePreviewViewer.value && needsFullPreview(markdown.value) && !previewEngineReady.value
)

const normalizePreviewHtml = (html: string): string =>
  html.replace(/>\s*\n{2,}\s*</g, '><')

const previewHtml = computed(() => {
  const source = previewSource.value
  const filePath = currentFilePath.value
  let html = ''
  if (previewEngineReady.value && bytemdGetProcessor && plugins.value.length > 0) {
    html = bytemdGetProcessor({ plugins: plugins.value }).processSync(source).toString()
    html = normalizePreviewHtml(html)
  } else if (needsFullPreview(markdown.value)) {
    return '<p class="preview-loading-hint">正在加载完整预览…</p>'
  } else {
    html = lightMarkdownToHtml(source)
  }
  return rewritePreviewAssetUrls(html, filePath)
})

const applyPreviewViewerEffects = () => {
  previewViewerEffectCleanups.forEach((cleanup) => cleanup())
  previewViewerEffectCleanups = []

  const body = previewBodyRef.value
  if (!body || !previewEngineReady.value || !bytemdGetProcessor) {
    if (body) bindPreviewImageZoom(body)
    return
  }

  fixPreviewImages(body, currentFilePath.value)

  const file = bytemdGetProcessor({ plugins: plugins.value }).processSync(previewSource.value)
  for (const plugin of plugins.value) {
    const cleanup = plugin.viewerEffect?.({ markdownBody: body, file } as never)
    if (typeof cleanup === 'function') {
      previewViewerEffectCleanups.push(cleanup)
    }
  }

  bindPreviewImageZoom(body)
}

const syncPreviewAssets = async () => {
  const filePath = currentFilePath.value
  if (!filePath) return
  await allowMarkdownAssets(filePath, markdown.value)
  nextTick(() => {
    fixPreviewImages(previewBodyRef.value, filePath)
    const byteMdPreview = editorContentRef.value?.querySelector(
      '.bytemd-preview .markdown-body'
    ) as HTMLElement | null
    fixPreviewImages(byteMdPreview, filePath)
    bindPreviewImageZoom(previewBodyRef.value)
    bindPreviewImageZoom(byteMdPreview)
  })
}

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
  void checkPendingExternalChange(file)
  if (needsFullPreview(file.content)) {
    startMediumPreviewPreload(file.content)
    await loadMediumPlugins()
  }
  await ensurePreviewPipeline()
  void syncPreviewAssets()
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

const togglePreviewToc = () => {
  previewTocVisible.value = !previewTocVisible.value
  localStorage.setItem(PREVIEW_TOC_STORAGE_KEY, String(previewTocVisible.value))
}

const togglePreview = () => {
  const modes: EditorLayoutMode[] = ['preview-only', 'split', 'tab']
  const currentIndex = modes.indexOf(editorLayout.value)
  editorLayout.value = modes[(currentIndex + 1) % modes.length]
  localStorage.setItem(EDITOR_MODE_STORAGE_KEY, editorLayout.value)
}

const setEditorLayout = (layout: EditorLayoutMode) => {
  editorLayout.value = layout
  localStorage.setItem(EDITOR_MODE_STORAGE_KEY, layout)
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

const persistOpenSession = () => {
  const paths = files.value.map((file) => file.path).filter(isDiskFilePath)
  const currentPath =
    currentFile.value && isDiskFilePath(currentFile.value.path)
      ? currentFile.value.path
      : ''
  try {
    const session: OpenSession = { paths, currentPath }
    sessionStorage.setItem(OPEN_SESSION_KEY, JSON.stringify(session))
  } catch {
    // 忽略私密模式 / 配额错误
  }
}

const readOpenSession = (): OpenSession | null => {
  try {
    const raw = sessionStorage.getItem(OPEN_SESSION_KEY)
    if (!raw) return null
    const parsed = JSON.parse(raw) as OpenSession
    if (!Array.isArray(parsed.paths)) return null
    return {
      paths: parsed.paths.filter((path) => typeof path === 'string' && isDiskFilePath(path)),
      currentPath: typeof parsed.currentPath === 'string' ? parsed.currentPath : ''
    }
  } catch {
    return null
  }
}

const closeTabContextMenu = (force = false) => {
  // 注意：不要把 DOM Event 当成 force；只有显式 true 才强制关闭
  if (force !== true && Date.now() - tabContextMenuOpenedAt < 120) return
  tabContextMenu.value = null
}

const onWindowClickCloseTabMenu = () => {
  closeTabContextMenu(false)
}

const openTabContextMenu = (event: MouseEvent, file: FileItem) => {
  tabContextMenuOpenedAt = Date.now()
  tabContextMenu.value = {
    x: event.clientX,
    y: event.clientY,
    file,
    isDiskFile: isDiskFilePath(file.path)
  }
}

const copyTabFilePath = async () => {
  const menu = tabContextMenu.value
  if (!menu?.isDiskFile) return
  try {
    await navigator.clipboard.writeText(menu.file.path)
  } catch (error) {
    console.error('复制文件路径失败:', error)
  } finally {
    closeTabContextMenu(true)
  }
}

const copyTabFileName = async () => {
  const menu = tabContextMenu.value
  if (!menu) return
  try {
    await navigator.clipboard.writeText(menu.file.name)
  } catch (error) {
    console.error('复制文件名失败:', error)
  } finally {
    closeTabContextMenu(true)
  }
}

const revealTabFileInExplorer = async () => {
  const menu = tabContextMenu.value
  if (!menu?.isDiskFile) return
  const filePath = menu.file.path
  closeTabContextMenu(true)
  try {
    await tauriInvoke('reveal_in_file_manager', { path: filePath })
  } catch (error) {
    console.error('打开所在目录失败:', error)
  }
}

const refreshTabFile = async () => {
  const menu = tabContextMenu.value
  if (!menu?.isDiskFile) return
  const file = menu.file
  closeTabContextMenu(true)
  await refreshFileFromDisk(file)
}

const syncDiskFileWatches = async () => {
  const paths = files.value.map((file) => file.path).filter(isDiskFilePath)
  try {
    await tauriInvoke('sync_file_watches', { paths })
  } catch (error) {
    console.error('同步文件监听失败:', error)
  }
}

const reloadFileFromDisk = async (file: FileItem, newContent?: string) => {
  const content = newContent ?? await tauriReadTextFile(file.path)
  file.content = content
  clearExternalChange(file.path)

  if (currentFile.value?.path === file.path) {
    markdown.value = content
    isModified.value = false
    await ensurePreviewPipeline()
    void syncPreviewAssets()
  }
}

/** 手动刷新：从磁盘重读最新内容并更新显示 */
const refreshFileFromDisk = async (file: FileItem) => {
  if (!isDiskFilePath(file.path)) return

  if (isModified.value && currentFile.value?.path === file.path) {
    const ok = await tauriAsk(
      `「${file.name}」有未保存更改，重新加载将丢失修改。\n\n是否继续？`,
      { title: 'MarkFly', kind: 'warning', okLabel: '重新加载', cancelLabel: '取消' }
    )
    if (!ok) return
  }

  try {
    await reloadFileFromDisk(file)
  } catch (error) {
    console.error('从磁盘重新加载失败:', error)
  }
}

/**
 * 按路径从磁盘重新打开文件（忽略缓存内容，始终读最新）。
 * 用于启动恢复与 WebView Refresh。
 */
const openDiskPathsFresh = async (paths: string[], preferredCurrent = '') => {
  const uniquePaths = [...new Set(paths.filter(isDiskFilePath))]
  if (uniquePaths.length === 0) return false

  for (const path of uniquePaths) {
    const existing = files.value.find((file) => file.path === path)
    if (existing) {
      try {
        await reloadFileFromDisk(existing)
      } catch (error) {
        console.error('重新读取文件失败:', path, error)
      }
      continue
    }

    try {
      const content = await tauriReadTextFile(path)
      files.value.push({
        name: getFileNameFromPath(path),
        path,
        content
      })
    } catch (error) {
      console.error('重新读取文件失败:', path, error)
    }
  }

  const targetPath =
    (preferredCurrent && files.value.some((file) => file.path === preferredCurrent)
      ? preferredCurrent
      : '') ||
    uniquePaths.find((path) => files.value.some((file) => file.path === path)) ||
    ''

  if (targetPath) {
    const file = files.value.find((item) => item.path === targetPath)
    if (file) await selectFile(file)
  }

  return files.value.some((file) => isDiskFilePath(file.path))
}

/** 顶部横幅：从磁盘重新加载当前文件 */
const reloadCurrentFileFromDisk = async () => {
  if (!currentFile.value) return
  await refreshFileFromDisk(currentFile.value)
}

/** 关闭顶部提示（保留标签黄点，仍可通过右键 / Ctrl+R 刷新） */
const dismissExternalChangeBanner = () => {
  const path = currentFile.value?.path
  if (!path || dismissedExternalBannerPaths.value.includes(path)) return
  dismissedExternalBannerPaths.value = [...dismissedExternalBannerPaths.value, path]
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
    newContent = await tauriReadTextFile(filePath)
  } catch (error) {
    console.error('读取外部变更失败:', error)
    return
  }

  if (newContent === file.content) {
    clearExternalChange(filePath)
    return
  }

  const prevNotified = lastExternalContents.get(filePath)
  lastExternalContents.set(filePath, newContent)
  // 仅在内容相对上次通知发生变化时重新弹出横幅，避免保存时重复事件刷屏
  if (prevNotified !== newContent) {
    revealExternalChangeBanner(filePath)
  }
  markExternalChange(filePath)
}

const checkPendingExternalChange = async (file: FileItem) => {
  if (!externalChangePaths.value.includes(file.path)) {
    return
  }

  let newContent: string
  try {
    newContent = await tauriReadTextFile(file.path)
  } catch (error) {
    console.error('读取外部变更失败:', error)
    return
  }

  if (newContent === file.content) {
    clearExternalChange(file.path)
    watchSuppressUntil.delete(file.path)
    return
  }

  // 切回该标签时重新展示横幅（关闭只影响当前停留期间）
  revealExternalChangeBanner(file.path)
}

const loadWelcomeSample = async () => {
  if (files.value.length > 0) {
    return
  }

  const { sampleFiles } = await import('./data/sampleFiles')
  files.value = [...sampleFiles]
  if (sampleFiles.length > 0) {
    selectFile(sampleFiles[0])
  }
}

const openFileFromPath = async (filePath: string, preloadedContent?: string) => {
  try {
    const existing = files.value.find((file) => file.path === filePath)
    if (existing) {
      selectFile(existing)
      return
    }

    const content = preloadedContent ?? await tauriReadTextFile(filePath)
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

const openFilePaths = async (items: PendingOpenFile[] | string[]) => {
  if (items.length === 0) return

  for (let index = 0; index < items.length; index++) {
    const item = items[index]
    if (typeof item === 'string') {
      await openFileFromPath(item)
      continue
    }

    const existing = files.value.find((file) => file.path === item.path)
    if (existing) {
      if (index === 0) selectFile(existing)
      continue
    }

    const newFile: FileItem = {
      name: getFileNameFromPath(item.path),
      path: item.path,
      content: item.content
    }
    files.value.push(newFile)
    if (index === 0) selectFile(newFile)
  }
}

// 新增：打开本地 Markdown 文件
const openLocalFile = async () => {
  try {
    const selected = await tauriOpenFile({
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
      await tauriWriteTextFile(currentFile.value.path, markdown.value)
      currentFile.value.content = markdown.value
      isModified.value = false
      console.log('文件已保存:', currentFile.value.path)
    } else {
      // 弹出保存对话框
      const filePath = await tauriSaveFile({
        filters: [{
          name: 'Markdown Files',
          extensions: ['md']
        }]
      })
      
      if (filePath) {
        markFileSavedOnDisk(filePath)
        await tauriWriteTextFile(filePath, markdown.value)
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
      
    const filePath = await tauriSaveFile({
      defaultPath: defaultPath,
      filters: [{
        name: 'Markdown Files',
        extensions: ['md']
      }]
    })
    
    if (filePath) {
      markFileSavedOnDisk(filePath)
      await tauriWriteTextFile(filePath, markdown.value)
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
  if (event.key === 'Escape' && tabContextMenu.value) {
    closeTabContextMenu()
    return
  }

  // Ctrl/Cmd + B 显示/隐藏侧边栏（捕获阶段优先于编辑器粗体）
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'b' && !event.shiftKey && !event.altKey) {
    event.preventDefault()
    event.stopPropagation()
    toggleSidebar()
    return
  }
  // Ctrl/Cmd + R：从磁盘重新加载当前文件（避免 WebView 整页刷新用到旧缓存）
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'r' && !event.shiftKey && !event.altKey) {
    event.preventDefault()
    event.stopPropagation()
    if (currentFile.value) {
      void refreshFileFromDisk(currentFile.value)
    }
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

const hideBootLayer = async () => {
  if (!currentFile.value) {
    window.__markflyHideBoot?.()
    return
  }
  await nextTick()
  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
  })
  window.__markflyHideBoot?.()
}

const mountDeferredChrome = () => {
  showDeferredChrome.value = true
}

// 初始化主题
onMounted(async () => {
  themeStore.initTheme()
  window.addEventListener('keydown', handleKeyDown, true)
  window.addEventListener('click', onWindowClickCloseTabMenu)
  window.addEventListener('blur', onWindowClickCloseTabMenu)
  window.addEventListener('beforeunload', persistOpenSession)

  // 尽早关闭 boot 层，避免 PageLoad Finished 再次注入 boot 盖住 Vue UI
  await hideBootLayer()

  try {
    const pendingFiles = await pendingBootstrapPromise
    const session = readOpenSession()
    // boot / pending 里的 content 可能是启动时的旧缓存；刷新后一律按路径重读磁盘
    const pendingPaths = pendingFiles.map((file) => file.path).filter(isDiskFilePath)
    const sessionPaths = session?.paths ?? []
    const pathsToRestore = [...new Set([...pendingPaths, ...sessionPaths])]
    const preferredCurrent =
      (session?.currentPath && pathsToRestore.includes(session.currentPath)
        ? session.currentPath
        : '') ||
      pendingPaths[0] ||
      sessionPaths[0] ||
      ''

    if (pathsToRestore.length > 0) {
      const restored = await openDiskPathsFresh(pathsToRestore, preferredCurrent)
      if (!restored && files.value.length === 0) {
        await loadWelcomeSample()
      }
    } else if (files.value.length === 0) {
      await loadWelcomeSample()
    }
  } catch (error) {
    console.error('启动时打开文件失败:', error)
    if (files.value.length === 0) {
      await loadWelcomeSample()
    }
  } finally {
    isBootstrapping.value = false
    persistOpenSession()
    try {
      if (currentFile.value) {
        await ensurePreviewPipeline()
        await syncPreviewAssets()
      }
    } catch (error) {
      console.error('预览管道加载失败:', error)
    }
    mountDeferredChrome()
    if (typeof requestIdleCallback === 'function') {
      requestIdleCallback(() => void ensureEditorComponent(), { timeout: 5000 })
    }
  }

  queueMicrotask(() => {
    void setupDeferredAppServices()
  })
})

const setupDeferredAppServices = async () => {
  await tauriListen<string[]>('open-file-path', async (paths) => {
    await openFilePaths(paths)
  })

  tauriListen<string>('menu', (payload) => {
    switch (payload) {
      case 'new-file':
        createNewFile()
        break
      case 'open-file':
        openLocalFile()
        break
      case 'save-file':
        saveFileToLocal()
        break
      case 'save-file-as':
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
    }
  }).catch((error) => {
    console.error('注册菜单事件监听器失败:', error)
  })

  unlistenFileChanged = await tauriListen<string>('file-changed', async (filePath) => {
    await handleExternalFileChange(filePath)
  })

  await syncDiskFileWatches()
  scheduleToolbarMount()
  nextTick(setupToolbarObserver)
  clampSidebarBtnPosition()
  window.addEventListener('resize', clampSidebarBtnPosition)
}

watch(
  () => files.value.map((file) => file.path).join('\0'),
  () => {
    void syncDiskFileWatches()
    persistOpenSession()
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

watch(editorLayout, async (layout) => {
  if (layout !== 'preview-only') {
    await loadMediumPlugins()
    await ensureEditorComponent()
  } else {
    void ensurePreviewPipeline()
  }
  if (layout === 'preview-only') {
    clearByteMdToolbarFromHost()
  } else {
    scheduleToolbarMount()
  }
})

watch(currentFilePath, () => {
  persistOpenSession()
  closeTabContextMenu()
  scheduleToolbarMount()
  nextTick(setupToolbarObserver)
})

watch(markdown, (content) => {
  if (!mediumPluginsLoaded && needsMediumPlugins(content)) {
    void loadMediumPlugins()
  }
  if (!heavyPluginsLoaded && needsHeavyPlugins(content)) {
    void loadHeavyPlugins()
  }
})

watch([previewHtml, plugins, usePreviewViewer, currentFilePath], () => {
  if (!usePreviewViewer.value) return
  void syncPreviewAssets()
  nextTick(() => applyPreviewViewerEffects())
})

watch([markdown, editorLayout, currentFilePath], () => {
  if (usePreviewViewer.value) return
  void syncPreviewAssets()
})

// 清理事件监听器
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown, true)
  window.removeEventListener('click', onWindowClickCloseTabMenu)
  window.removeEventListener('blur', onWindowClickCloseTabMenu)
  window.removeEventListener('beforeunload', persistOpenSession)
  window.removeEventListener('resize', clampSidebarBtnPosition)
  unlistenFileChanged?.()
  toolbarObserver?.disconnect()
  unbindPreviewImageZoom(previewBodyRef.value)
  previewViewerEffectCleanups.forEach((cleanup) => cleanup())
  void tauriInvoke('sync_file_watches', { paths: [] })
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

.tab-context-menu {
  position: fixed;
  z-index: 1000;
  min-width: 160px;
  padding: 4px 0;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18);
}

.tab-context-item {
  display: block;
  width: 100%;
  border: none;
  background: transparent;
  color: var(--text-primary);
  text-align: left;
  padding: 6px 12px;
  font-size: 12px;
  cursor: pointer;
}

.tab-context-item:hover:not(:disabled) {
  background: var(--hover-bg);
}

.tab-context-item:disabled {
  opacity: 0.45;
  cursor: not-allowed;
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

/* 外部文件变更：单行轻提示（整条可点，右侧关闭） */
.external-change-banner {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 32px;
  padding: 6px 10px 6px 14px;
  font-size: 13px;
  line-height: 1.4;
  color: var(--text-secondary);
  background: color-mix(in srgb, var(--accent-color) 5%, var(--bg-primary));
  border-bottom: 1px solid var(--border-color);
  border-left: 2px solid var(--accent-color);
  cursor: pointer;
  user-select: none;
  transition: background-color 0.15s ease;
}

.external-change-banner:hover {
  background: color-mix(in srgb, var(--accent-color) 8%, var(--bg-primary));
}

.external-change-banner:focus-visible {
  outline: 1px solid var(--accent-color);
  outline-offset: -1px;
}

.external-change-banner-icon {
  flex-shrink: 0;
  color: var(--accent-color);
  opacity: 0.75;
}

.external-change-banner-text {
  min-width: 0;
  flex: 1 1 auto;
  display: flex;
  align-items: center;
  gap: 6px;
  overflow: hidden;
  white-space: nowrap;
}

.external-change-banner-label {
  flex-shrink: 0;
  color: var(--text-primary);
  font-weight: 500;
}

.external-change-banner-sep {
  flex-shrink: 0;
  color: var(--text-tertiary);
}

.external-change-banner-hint {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-secondary);
}

.external-change-banner:hover .external-change-banner-hint {
  color: var(--accent-color);
}

.external-change-banner-arrow {
  flex-shrink: 0;
  margin-left: auto;
  padding-right: 2px;
  font-size: 14px;
  line-height: 1;
  color: var(--text-tertiary);
  transition: color 0.15s ease, transform 0.15s ease;
}

.external-change-banner:hover .external-change-banner-arrow {
  color: var(--accent-color);
  transform: translateX(2px);
}

.external-change-banner-close {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  opacity: 0.7;
  transition: opacity 0.15s ease, background-color 0.15s ease, color 0.15s ease;
}

.external-change-banner-close:hover {
  opacity: 1;
  background: var(--hover-bg);
  color: var(--text-secondary);
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
  min-height: 35px;
  height: auto;
  flex-shrink: 0;
  background: var(--tab-bar-bg);
  border-bottom: 1px solid var(--border-color);
}

.editor-unified-header.is-merged .file-tabs {
  flex: 1 1 0;
  min-width: 64px;
  height: auto;
  min-height: 35px;
  border-bottom: none;
  background: transparent;
  padding-right: 4px;
}

.toolbar-host {
  flex: 0 1 auto;
  min-width: 0;
  display: flex;
  align-items: center;
  align-self: stretch;
  justify-content: flex-end;
  gap: 2px;
}

.toolbar-host.collapsed {
  flex: 0 0 auto;
  min-width: 28px;
}

.toolbar-actions {
  flex: 0 1 auto;
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-wrap: wrap;
  align-content: center;
  max-height: 70px;
  overflow: hidden;
}

.toolbar-host.collapsed .toolbar-actions {
  display: none;
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
  flex: 0 0 auto;
  flex-shrink: 0;
  margin-right: 4px;
  transition: background-color 0.2s ease, color 0.2s ease;
}

.toolbar-host.collapsed .toolbar-toggle-btn {
  color: var(--text-primary);
}

.toolbar-toggle-btn:hover {
  background: var(--hover-bg);
  color: var(--text-primary);
}

.toolbar-actions :deep(.bytemd-toolbar) {
  flex: 0 1 auto;
  min-width: 0;
  width: auto;
  max-width: 100%;
  border-bottom: none !important;
  background: transparent !important;
  padding: 0 8px !important;
  min-height: 35px !important;
  height: auto !important;
  max-height: 70px !important;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-content: center;
  overflow: hidden;
  box-sizing: border-box;
}

.toolbar-actions :deep(.bytemd-toolbar-left),
.toolbar-actions :deep(.bytemd-toolbar-right) {
  flex-wrap: wrap;
  justify-content: flex-end;
}

.editor-main {
  flex: 1;
  min-height: 0;
  position: relative;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.editor-content.is-background {
  position: absolute;
  inset: 0;
  visibility: hidden;
  pointer-events: none;
  z-index: 0;
}

.welcome-screen {
  flex: 1;
  min-height: 0;
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: auto;
}

.markfly-preview-viewer {
  flex: 1;
  min-width: 0;
  height: 100%;
  overflow: auto;
  padding: 16px 24px;
  box-sizing: border-box;
  background: var(--editor-bg);
}

.preview-viewer-shell {
  display: flex;
  flex: 1;
  min-width: 0;
  min-height: 0;
  width: 100%;
  height: 100%;
}

.preview-layout-toolbar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 2px;
  padding-right: 4px;
}

.layout-mode-btn {
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
  transition: background-color 0.15s ease, color 0.15s ease;
}

.layout-mode-icon :deep(svg) {
  width: 14px;
  height: 14px;
  display: block;
}

.layout-mode-btn:hover:not(:disabled) {
  background: var(--hover-bg);
  color: var(--text-primary);
}

.layout-mode-btn.is-active {
  color: var(--accent-color);
  cursor: default;
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
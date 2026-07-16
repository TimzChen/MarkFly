<template>
  <div class="markfly-app">
    <!-- 主编辑区域 -->
    <div class="app-body">
      <!-- 左侧文件树 -->
      <FileTree 
        :files="files"
        :activeFile="currentFilePath"
        :collapsed="sidebarCollapsed"
        @selectFile="selectFile"
        @newFile="createNewFile"
        @openFolder="openFolder"
        @closeFile="closeFile"
        @toggleCollapse="toggleSidebar"
      />

      <button
        v-if="sidebarCollapsed"
        class="sidebar-expand-btn"
        @click="toggleSidebar"
        title="显示侧边栏 (视图菜单 Ctrl+B)"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
          <polyline points="9,6 15,12 9,18" stroke="currentColor" stroke-width="2"/>
        </svg>
      </button>
      
      <!-- 中央编辑区域 -->
      <div class="editor-container">
        <!-- 文件标签栏 -->
        <div class="file-tabs" v-if="files.length > 0">
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
            <button class="tab-close" @click.stop="closeFile(file)" v-if="files.length > 1" title="关闭文件">
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none">
                <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
                <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
              </svg>
            </button>
          </div>
        </div>
        
        <div class="editor-content" v-if="currentFile" @click.capture="handleToolbarLayoutClick">
          <Editor 
            :key="editorLayout"
            :value="markdown" 
            :plugins="plugins"
            @change="handleChange"
            :locale="locale"
            mode="split"
            :class="['bytemd-editor-wrapper', `markfly-layout-${editorLayout}`]"
          />
        </div>
        
        <!-- 欢迎界面 -->
        <div class="welcome-screen" v-else>
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

    <!-- 状态栏 -->
    <div class="app-footer">
      <div class="footer-left">
        <button class="settings-btn" @click="openSettings" title="设置">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
            <path d="M12 1v6m0 6v6m11-7h-6m-6 0H1m17-4a4 4 0 0 1-8 0 4 4 0 0 1 8 0zM7 16a4 4 0 0 1-8 0 4 4 0 0 1 8 0z" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
        <span class="status-item">Ln {{ currentLine }}, Col {{ currentColumn }}</span>
        <span class="status-item">{{ wordCount }} words</span>
        <span class="status-item">{{ markdown.length }} characters</span>
      </div>
      <div class="footer-right">
        <span class="status-item">Markdown</span>
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
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
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
const EDITOR_MODE_STORAGE_KEY = 'markfly-editor-mode'

type EditorLayoutMode = 'preview-only' | 'split' | 'tab'

const loadEditorLayout = (): EditorLayoutMode => {
  const stored = localStorage.getItem(EDITOR_MODE_STORAGE_KEY)
  // 兼容旧版无效的 'preview' 值
  if (stored === 'preview' || stored === 'preview-only') return 'preview-only'
  if (stored === 'split' || stored === 'tab') return stored
  return 'preview-only'
}

// 响应式数据
const sidebarCollapsed = ref(localStorage.getItem(SIDEBAR_STORAGE_KEY) !== 'false')
const files = ref<FileItem[]>([])
const currentFile = ref<FileItem | null>(null)
const markdown = ref('')
const isModified = ref(false)
const editorLayout = ref<EditorLayoutMode>(loadEditorLayout())
const currentLine = ref(1)
const currentColumn = ref(1)
const showSettings = ref(false)
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

const wordCount = computed(() => {
  return markdown.value.trim().split(/\s+/).filter((word: string) => word.length > 0).length
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

  const pendingFiles = await invoke<string[]>('get_pending_open_files')
  if (pendingFiles.length > 0) {
    await openFilePaths(pendingFiles)
  } else if (files.value.length === 0) {
    loadWelcomeSample()
  }

  await syncDiskFileWatches()
})

watch(
  () => files.value.map((file) => file.path).join('\0'),
  () => {
    void syncDiskFileWatches()
  }
)

// 清理事件监听器
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown, true)
  unlistenFileChanged?.()
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
  background: var(--bg-secondary); /* 与知识库背景色一致 */
  border-bottom: 1px solid var(--border-color);
  padding: 0 12px; /* 与文件树头部保持一致的内边距 */
  display: flex;
  align-items: center;
  gap: 8px;
  height: 35px; /* 确保固定高度与文件树头部一致 */
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: thin;
  box-sizing: border-box;
  position: relative; /* 使position: sticky生效的父元素 */
  flex-shrink: 0; /* 防止在flex布局中被压缩 */
  z-index: 10; /* 确保在编辑器上方 */
}

.file-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  background: var(--bg-secondary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s ease;
  border: 1px solid transparent;
  flex-shrink: 0; /* 防止标签被压缩 */
}

.file-tab:hover {
  background: var(--hover-bg);
  border-color: var(--border-color);
}

.file-tab.active {
  background: var(--accent-color);
  color: var(--bg-primary);
  border-color: var(--accent-color);
}

.tab-file-icon {
  flex-shrink: 0;
  color: var(--text-secondary);
}

.file-tab.active .tab-file-icon {
  color: var(--bg-primary);
}

.tab-name {
  flex: 1;
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
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.file-tab:hover .tab-close {
  opacity: 1;
}

.file-tab.active .tab-close {
  color: var(--bg-primary);
  opacity: 1;
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

.sidebar-expand-btn {
  position: absolute;
  left: 0;
  top: 8px;
  z-index: 20;
  width: 20px;
  height: 48px;
  border: 1px solid var(--border-color);
  border-left: none;
  border-radius: 0 6px 6px 0;
  background: var(--bg-secondary);
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.2s ease, color 0.2s ease;
}

.sidebar-expand-btn:hover {
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

.editor-content {
  flex: 1;
  min-height: 0;
  /* 确保编辑器容器可以正确处理滚动 */
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
  background: var(--accent-color);
  color: var(--bg-primary);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  font-size: 12px;
}

.footer-left,
.footer-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-item {
  white-space: nowrap;
  font-size: 12px;
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
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
  margin-right: 8px;
}

.settings-btn:hover,
.theme-toggle-btn:hover {
  background: var(--hover-bg);
  color: var(--text-primary);
}

.settings-btn {
  margin-right: 12px;
}

.theme-toggle-btn {
  margin-left: 8px;
  margin-right: 0;
}
</style>
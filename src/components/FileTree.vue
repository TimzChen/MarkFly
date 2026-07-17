<template>
  <div class="file-tree" :class="{ collapsed }" :aria-hidden="collapsed">
    <div class="file-tree-header">
      <div class="header-title">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
          <path d="M3 3h7l2 3h9v12H3V3z" stroke="currentColor" stroke-width="2" fill="none"/>
        </svg>
        <span>知识库</span>
      </div>
      <div class="header-actions">
        <button class="action-btn" @click="$emit('newFile')" title="新建文件">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path d="M12 5v14m-7-7h14" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
        <button class="action-btn" @click="$emit('openFolder')" title="打开文件夹">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path d="M3 3h7l2 3h9v12H3V3z" stroke="currentColor" stroke-width="2" fill="none"/>
          </svg>
        </button>
      </div>
    </div>
    
    <!-- 文件列表 -->
    <div class="file-list">
      <div 
        v-for="file in files" 
        :key="file.path"
        class="file-item"
        :class="{ active: file.path === activeFile }"
        @click="$emit('selectFile', file)"
      >
        <div class="file-icon">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" stroke="currentColor" stroke-width="2" fill="none"/>
            <polyline points="14,2 14,8 20,8" stroke="currentColor" stroke-width="2"/>
          </svg>
        </div>
        <span class="file-name">{{ file.name }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface FileItem {
  name: string
  content: string
  path: string
}

withDefaults(defineProps<{
  files: FileItem[]
  activeFile: string
  collapsed?: boolean
}>(), {
  collapsed: false
})

defineEmits<{
  selectFile: [file: FileItem]
  newFile: []
  openFolder: []
  closeFile: [file: FileItem]
}>()
</script>

<style scoped>
.file-tree {
  width: 280px;
  min-width: 280px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color);
  transition: width 0.2s ease, min-width 0.2s ease, opacity 0.2s ease, border-color 0.2s ease;
  overflow: hidden;
  flex-shrink: 0;
}

.file-tree.collapsed {
  width: 0;
  min-width: 0;
  opacity: 0;
  border-right-color: transparent;
  pointer-events: none;
}

.file-tree-header {
  height: auto;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
  font-size: 11px;
  text-transform: uppercase;
  font-weight: 600;
  letter-spacing: 0.5px;
  flex-shrink: 0;
  background: var(--bg-secondary);
  box-sizing: border-box;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-primary);
  min-height: 35px;
}

.header-actions {
  display: flex;
  gap: 4px;
  margin-left: auto;
  min-height: 35px;
  align-items: center;
}

.action-btn {
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  color: var(--text-primary);
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.2s;
}

.action-btn:hover {
  background: var(--hover-bg);
}

.file-list {
  flex: 1;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: 8px 0;
  scrollbar-width: thin;
  background: var(--bg-secondary);
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px;
  cursor: pointer;
  font-size: 13px;
  transition: background-color 0.2s;
}

.file-item:hover {
  background: var(--hover-bg);
}

.file-item.active {
  background: var(--list-active-bg);
  color: var(--list-active-fg);
}

.file-icon {
  display: flex;
  align-items: center;
  color: var(--text-secondary);
}

.file-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>

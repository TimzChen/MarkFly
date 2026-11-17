<template>
  <div class="settings-overlay" v-if="visible" @click="close">
    <div class="settings-panel" @click.stop>
      <div class="settings-header">
        <h3>设置</h3>
        <button class="close-btn" @click="close">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>
      
      <div class="settings-content">
        <div class="setting-group">
          <h4>外观</h4>
          <div class="setting-item">
            <label>主题</label>
            <select :value="themeStore.mode" @change="onThemeChange">
              <option value="light">浅色</option>
              <option value="dark">深色</option>
              <option value="auto">跟随系统</option>
            </select>
          </div>
        </div>
        
        <div class="setting-group">
          <h4>编辑器</h4>
          <div class="setting-item">
            <label>字体大小</label>
            <select v-model="fontSize">
              <option value="12">12px</option>
              <option value="14">14px</option>
              <option value="16">16px</option>
              <option value="18">18px</option>
            </select>
          </div>
          <div class="setting-item">
            <label>自动换行</label>
            <input type="checkbox" v-model="wordWrap">
          </div>
        </div>
        
        <div class="setting-group">
          <h4>快捷键</h4>
          <div class="shortcut-list">
            <div class="shortcut-item">
              <span>新建文件</span>
              <kbd>Ctrl/Cmd + N</kbd>
            </div>
            <div class="shortcut-item">
              <span>打开文件</span>
              <kbd>Ctrl/Cmd + O</kbd>
            </div>
            <div class="shortcut-item">
              <span>保存文件</span>
              <kbd>Ctrl/Cmd + S</kbd>
            </div>
            <div class="shortcut-item">
              <span>切换主题</span>
              <kbd>Ctrl/Cmd + T</kbd>
            </div>
            <div class="shortcut-item">
              <span>刷新目录 / 当前文件</span>
              <kbd>F5</kbd>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useThemeStore, type ThemeMode } from '../stores/theme'

interface Props {
  visible: boolean
}

interface Emits {
  (e: 'close'): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()

const themeStore = useThemeStore()
const fontSize = ref(14)
const wordWrap = ref(true)

const onThemeChange = (event: Event) => {
  const value = (event.target as HTMLSelectElement).value
  if (value === 'light' || value === 'dark' || value === 'auto') {
    themeStore.setTheme(value as ThemeMode)
  }
}

const close = () => {
  emit('close')
}
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-panel {
  width: 500px;
  max-height: 600px;
  background: var(--bg-primary);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.settings-header h3 {
  margin: 0;
  font-size: 16px;
  color: var(--text-primary);
}

.close-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: var(--hover-bg);
  color: var(--text-primary);
}

.settings-content {
  padding: 20px;
  max-height: 500px;
  overflow-y: auto;
}

.setting-group {
  margin-bottom: 24px;
}

.setting-group:last-child {
  margin-bottom: 0;
}

.setting-group h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
}

.setting-item label {
  font-size: 13px;
  color: var(--text-primary);
}

.setting-item select {
  padding: 4px 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
}

.setting-item input[type="checkbox"] {
  width: 16px;
  height: 16px;
}

.shortcut-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.shortcut-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
}

.shortcut-item span {
  font-size: 13px;
  color: var(--text-primary);
}

.shortcut-item kbd {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  padding: 2px 6px;
  font-size: 11px;
  font-family: monospace;
  color: var(--text-secondary);
}
</style>
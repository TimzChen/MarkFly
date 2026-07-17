import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

export type ThemeMode = 'light' | 'dark' | 'auto'

export const useThemeStore = defineStore('theme', () => {
  // 状态
  const mode = ref<ThemeMode>('auto')
  const systemPrefersDark = ref(false)

  // 计算属性
  const isDark = computed(() => {
    if (mode.value === 'auto') {
      return systemPrefersDark.value
    }
    return mode.value === 'dark'
  })

  const currentTheme = computed(() => {
    return isDark.value ? 'dark' : 'light'
  })

  // 方法
  const setTheme = (newMode: ThemeMode) => {
    mode.value = newMode
    localStorage.setItem('theme-mode', newMode)
    applyTheme()
  }

  const toggleTheme = () => {
    if (mode.value === 'light') {
      setTheme('dark')
    } else if (mode.value === 'dark') {
      setTheme('auto')
    } else {
      setTheme('light')
    }
  }

  const applyTheme = () => {
    const root = document.documentElement
    
    if (isDark.value) {
      root.classList.add('dark')
      root.classList.remove('light')
    } else {
      root.classList.add('light')
      root.classList.remove('dark')
    }
    
    // 设置CSS变量
    if (isDark.value) {
      root.style.setProperty('--bg-primary', '#1e1e1e')
      root.style.setProperty('--bg-secondary', '#252526')
      root.style.setProperty('--bg-tertiary', '#2d2d30')
      root.style.setProperty('--text-primary', '#cccccc')
      root.style.setProperty('--text-secondary', '#969696')
      root.style.setProperty('--border-color', '#3e3e42')
      root.style.setProperty('--accent-color', '#3794ff')
      root.style.setProperty('--hover-bg', '#2a2d2e')
      root.style.setProperty('--tab-bar-bg', '#252526')
      root.style.setProperty('--tab-active-bg', '#1e1e1e')
      root.style.setProperty('--tab-active-fg', '#e7e7e7')
      root.style.setProperty('--tab-inactive-fg', '#969696')
      root.style.setProperty('--tab-active-border', '#007acc')
      root.style.setProperty('--status-bar-bg', '#181818')
      root.style.setProperty('--status-bar-fg', '#cccccc')
      root.style.setProperty('--list-active-bg', '#37373d')
      root.style.setProperty('--list-active-fg', '#cccccc')
    } else {
      root.style.setProperty('--bg-primary', '#ffffff')
      root.style.setProperty('--bg-secondary', '#f3f3f3')
      root.style.setProperty('--bg-tertiary', '#e8e8e8')
      root.style.setProperty('--text-primary', '#333333')
      root.style.setProperty('--text-secondary', '#616161')
      root.style.setProperty('--border-color', '#e5e5e5')
      root.style.setProperty('--accent-color', '#006ab1')
      root.style.setProperty('--hover-bg', '#ebebeb')
      root.style.setProperty('--tab-bar-bg', '#ececec')
      root.style.setProperty('--tab-active-bg', '#ffffff')
      root.style.setProperty('--tab-active-fg', '#333333')
      root.style.setProperty('--tab-inactive-fg', '#616161')
      root.style.setProperty('--tab-active-border', '#006ab1')
      root.style.setProperty('--status-bar-bg', '#f3f3f3')
      root.style.setProperty('--status-bar-fg', '#616161')
      root.style.setProperty('--list-active-bg', '#e8e8e8')
      root.style.setProperty('--list-active-fg', '#333333')
    }
  }

  const initTheme = () => {
    // 检测系统主题偏好
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    systemPrefersDark.value = mediaQuery.matches
    
    // 监听系统主题变化
    mediaQuery.addEventListener('change', (e) => {
      systemPrefersDark.value = e.matches
      if (mode.value === 'auto') {
        applyTheme()
      }
    })

    // 从localStorage恢复主题设置
    const savedMode = localStorage.getItem('theme-mode') as ThemeMode
    if (savedMode && ['light', 'dark', 'auto'].includes(savedMode)) {
      mode.value = savedMode
    }

    applyTheme()
  }

  return {
    mode,
    isDark,
    currentTheme,
    setTheme,
    toggleTheme,
    initTheme
  }
})
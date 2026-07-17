import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './styles/global.css'
import 'github-markdown-css/github-markdown-light.css'

const loadEditorStyles = () =>
  import('bytemd/dist/index.css').catch((error) => {
    console.error('加载编辑器样式失败:', error)
  })

void loadEditorStyles()
const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')
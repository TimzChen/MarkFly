import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './styles/global.css'
import 'github-markdown-css/github-markdown-light.css'
import { startMediumPreviewPreload } from './utils/previewEngine'

const boot = window.__MARKFLY_BOOT__
if (Array.isArray(boot) && boot[0]?.content) {
  startMediumPreviewPreload(boot[0].content)
}

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')

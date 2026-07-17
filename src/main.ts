import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './styles/global.css'
import 'github-markdown-css/github-markdown-light.css'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')

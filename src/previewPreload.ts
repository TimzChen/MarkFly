import { startMediumPreviewPreload } from './utils/previewEngine'

declare global {
  interface Window {
    __MARKFLY_BOOT__?: Array<{ path: string; content: string; truncated?: boolean }>
  }
}

const boot = window.__MARKFLY_BOOT__
if (Array.isArray(boot) && boot[0]?.content) {
  startMediumPreviewPreload(boot[0].content)
}

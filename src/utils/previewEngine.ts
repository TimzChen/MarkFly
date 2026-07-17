import type { BytemdPlugin } from 'bytemd'
import { needsFullPreview } from './markdownPreview'

export type MediumPreviewModules = {
  getProcessor: typeof import('bytemd').getProcessor
  createPlugins: () => BytemdPlugin[]
}

declare global {
  interface Window {
    __MARKFLY_MEDIUM_PRELOAD__?: Promise<MediumPreviewModules>
  }
}

const loadMediumModules = (): Promise<MediumPreviewModules> => {
  return (async () => {
    const [bytemd, { default: gfm }, { default: highlight }] = await Promise.all([
        import('bytemd'),
        import('@bytemd/plugin-gfm'),
        import('@bytemd/plugin-highlight'),
        import('highlight.js/styles/vs.css'),
      ])

    return {
      getProcessor: bytemd.getProcessor,
      createPlugins: () => [gfm(), highlight()],
    }
  })()
}

/** Boot / 主应用共用：尽早开始加载 GFM 预览依赖 */
export const startMediumPreviewPreload = (content?: string): Promise<MediumPreviewModules> | null => {
  if (window.__MARKFLY_MEDIUM_PRELOAD__) {
    return window.__MARKFLY_MEDIUM_PRELOAD__
  }
  if (content !== undefined && !needsFullPreview(content)) {
    return null
  }

  window.__MARKFLY_MEDIUM_PRELOAD__ = loadMediumModules()
  return window.__MARKFLY_MEDIUM_PRELOAD__
}

export const consumeMediumPreviewPreload = (): Promise<MediumPreviewModules> | null => {
  return window.__MARKFLY_MEDIUM_PRELOAD__ ?? null
}

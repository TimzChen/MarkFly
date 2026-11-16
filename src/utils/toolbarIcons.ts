/** ByteMD 工具栏图标（viewBox 0 0 48 48），与 bytemd/svelte/icons.js 保持一致 */
export const toolbarIcons = {
  toc: '<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 48 48"><path stroke="currentColor" stroke-linejoin="round" stroke-width="4" d="M39 6H9a3 3 0 0 0-3 3v30a3 3 0 0 0 3 3h30a3 3 0 0 0 3-3V9a3 3 0 0 0-3-3Z"/><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="4" d="M26 24H14M34 15H14M32 33H14"/></svg>',
  leftExpand:
    '<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 48 48"><rect width="28" height="36" x="6" y="6" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="4" rx="2"/><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="4" d="M42 6v36"/></svg>',
  rightExpand:
    '<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 48 48"><rect width="28" height="36" x="14" y="6" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="4" rx="2"/><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="4" d="M6 6v36"/></svg>',
  fullscreen:
    '<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><path stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M8 3H3v5M16 3h5v5M8 21H3v-5M16 21h5v-5"/></svg>',
  exitFullscreen:
    '<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><path stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M3 8h5V3M21 8h-5V3M3 16h5v5M21 16h-5v5"/></svg>',
} as const

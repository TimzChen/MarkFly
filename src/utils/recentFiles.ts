/** 知识库当前页签：已打开的标签 / 磁盘最近记录 */
export type KnowledgePane = 'opened' | 'recents' | 'folder'

export type RecentFile = {
  path: string
  name: string
  parentDir: string
}

const STORAGE_KEY = 'markfly-recent-files'
const MAX_RECENTS = 15

export function getFileNameFromPath(filePath: string): string {
  return filePath.split(/[/\\]/).filter(Boolean).pop() || '未命名.md'
}

export function getParentDirName(filePath: string): string {
  const parts = filePath.split(/[/\\]/).filter(Boolean)
  if (parts.length < 2) return ''
  return parts[parts.length - 2]
}

export function sameFilePath(a: string, b: string): boolean {
  return a.replace(/\\/g, '/').toLowerCase() === b.replace(/\\/g, '/').toLowerCase()
}

export function joinDiskPath(dir: string, name: string): string {
  const trimmed = dir.replace(/[\\/]+$/, '')
  const sep = dir.includes('\\') ? '\\' : '/'
  return `${trimmed}${sep}${name}`
}

export function siblingDiskPath(filePath: string, newName: string): string {
  const slash = Math.max(filePath.lastIndexOf('\\'), filePath.lastIndexOf('/'))
  if (slash < 0) return newName
  return filePath.slice(0, slash + 1) + newName
}

export function sanitizeFileName(raw: string): string {
  return raw.trim().replace(/[\\/:*?"<>|]/g, '')
}

export function ensureMarkdownName(raw: string, currentName = ''): string {
  const cleaned = sanitizeFileName(raw)
  if (!cleaned) return ''
  if (/\.(md|markdown|txt)$/i.test(cleaned)) return cleaned
  const currentExt = currentName.match(/\.(md|markdown|txt)$/i)?.[0] || '.md'
  return `${cleaned}${currentExt}`
}

export function fileNameStem(name: string): string {
  return name.replace(/\.[^.]+$/, '')
}

function toRecentFile(path: string): RecentFile {
  return {
    path,
    name: getFileNameFromPath(path),
    parentDir: getParentDirName(path),
  }
}

function persist(items: RecentFile[]): RecentFile[] {
  const next = items.slice(0, MAX_RECENTS)
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(next.map((item) => item.path)))
  } catch {
    // 忽略私密模式 / 配额错误
  }
  return next
}

export function readRecentFiles(): RecentFile[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    return parsed
      .filter((path): path is string => typeof path === 'string' && path.length > 0)
      .slice(0, MAX_RECENTS)
      .map(toRecentFile)
  } catch {
    return []
  }
}

export function rememberRecentFile(path: string): RecentFile[] {
  const current = readRecentFiles().filter((item) => !sameFilePath(item.path, path))
  return persist([toRecentFile(path), ...current])
}

export function removeRecentFile(path: string): RecentFile[] {
  return persist(readRecentFiles().filter((item) => !sameFilePath(item.path, path)))
}

export function clearRecentFiles(): RecentFile[] {
  return persist([])
}

export function replaceRecentFiles(paths: string[]): RecentFile[] {
  return persist(paths.map(toRecentFile))
}

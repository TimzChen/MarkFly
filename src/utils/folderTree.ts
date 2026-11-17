import type { RecentFile } from './recentFiles'

export type FolderDirRow = {
  kind: 'dir'
  key: string
  name: string
  depth: number
  collapsed: boolean
}

export type FolderFileRow = {
  kind: 'file'
  item: RecentFile
  depth: number
}

export type FolderTreeRow = FolderDirRow | FolderFileRow

type DirAccum = {
  name: string
  key: string
  dirs: Map<string, DirAccum>
  files: RecentFile[]
}

const splitDir = (parentDir: string) => parentDir.split(/[/\\]/).filter(Boolean)

/** 把扁平的「相对目录 + 文件」收成可折叠树，同级先文件夹后文件 */
export const flattenFolderTree = (
  files: RecentFile[],
  collapsedKeys: Record<string, boolean>
): FolderTreeRow[] => {
  const root: DirAccum = { name: '', key: '', dirs: new Map(), files: [] }

  const ensure = (segments: string[]): DirAccum => {
    let node = root
    let key = ''
    for (const segment of segments) {
      key = key ? `${key}/${segment}` : segment
      let child = node.dirs.get(segment)
      if (!child) {
        child = { name: segment, key, dirs: new Map(), files: [] }
        node.dirs.set(segment, child)
      }
      node = child
    }
    return node
  }

  for (const item of files) {
    ensure(item.parentDir ? splitDir(item.parentDir) : []).files.push(item)
  }

  const rows: FolderTreeRow[] = []
  const walk = (node: DirAccum, depth: number) => {
    const dirs = [...node.dirs.values()].sort((a, b) => a.name.localeCompare(b.name, 'zh'))
    for (const dir of dirs) {
      const collapsed = !!collapsedKeys[dir.key]
      rows.push({
        kind: 'dir',
        key: dir.key,
        name: dir.name,
        depth,
        collapsed,
      })
      if (!collapsed) walk(dir, depth + 1)
    }
    for (const item of node.files) {
      rows.push({ kind: 'file', item, depth })
    }
  }
  walk(root, 0)
  return rows
}

export const treePad = (depth: number) => ({
  paddingLeft: `${depth * 12 + 4}px`,
})

import type { BytemdPlugin } from 'bytemd'

const FOLD_CLASS = 'markfly-fold'
const COLLAPSED_CLASS = 'is-collapsed'
const BTN_CLASS = 'markfly-fold-btn'
const TITLE_CLASS = 'markfly-fold-title'

let docKeyGetter = () => ''

/** 按当前文档记住折叠，预览重绘后还能对上 */
export const bindPreviewFoldDoc = (getter: () => string) => {
  docKeyGetter = getter
}

const collapsedByDoc = new Map<string, Set<string>>()

export const forgetPreviewFoldDoc = (docKey: string) => {
  collapsedByDoc.delete(docKey)
}

export const migratePreviewFoldDoc = (from: string, to: string) => {
  if (!from || from === to) return
  const set = collapsedByDoc.get(from)
  if (!set) return
  collapsedByDoc.delete(from)
  collapsedByDoc.set(to, set)
}

/** 被收进折叠节里的标题（不是折叠节自己的标题） */
export const isHeadingFoldedAway = (el: HTMLElement) => {
  const fold = el.closest('.markfly-fold.is-collapsed')
  if (!(fold instanceof HTMLElement)) return false
  return fold.firstElementChild !== el
}

const collapsedSet = (docKey: string) => {
  let set = collapsedByDoc.get(docKey)
  if (!set) {
    set = new Set()
    collapsedByDoc.set(docKey, set)
  }
  return set
}

const headingLevel = (el: Element): number | null => {
  if (/^H[1-6]$/.test(el.tagName)) return Number(el.tagName[1])
  if (el.classList.contains(FOLD_CLASS)) {
    const heading = el.querySelector(':scope > h1, :scope > h2, :scope > h3, :scope > h4, :scope > h5, :scope > h6')
    return heading ? Number(heading.tagName[1]) : null
  }
  return null
}

const headingKey = (heading: HTMLElement, index: number) => {
  if (heading.id) return heading.id
  return `${heading.tagName}:${heading.textContent?.trim() || ''}:${index}`
}

const setCollapsed = (section: HTMLElement, heading: HTMLElement, collapsed: boolean, key: string) => {
  section.classList.toggle(COLLAPSED_CLASS, collapsed)
  const btn = heading.querySelector(`.${BTN_CLASS}`) as HTMLButtonElement | null
  if (btn) {
    btn.setAttribute('aria-expanded', collapsed ? 'false' : 'true')
    btn.title = collapsed ? '展开本节' : '折叠本节'
  }
  const set = collapsedSet(docKeyGetter())
  if (collapsed) set.add(key)
  else set.delete(key)
}

const makeFoldButton = () => {
  const btn = document.createElement('button')
  btn.type = 'button'
  btn.className = BTN_CLASS
  btn.setAttribute('aria-expanded', 'true')
  btn.title = '折叠本节'
  btn.innerHTML =
    '<svg viewBox="0 0 16 16" aria-hidden="true"><path fill="currentColor" d="M3.5 5.4h9L8 13.2z"/></svg>'
  return btn
}

const wrapHeading = (heading: HTMLElement, key: string) => {
  const parent = heading.parentElement
  if (!parent) return
  if (parent.classList.contains(FOLD_CLASS) && parent.firstElementChild === heading) return

  const level = Number(heading.tagName[1])
  const nodes: Node[] = []
  let sibling = heading.nextSibling
  while (sibling) {
    const el = sibling.nodeType === 1 ? (sibling as HTMLElement) : null
    if (el) {
      const nextLevel = headingLevel(el)
      if (nextLevel !== null && nextLevel <= level) break
    }
    nodes.push(sibling)
    sibling = sibling.nextSibling
  }
  if (nodes.length === 0) return

  const section = document.createElement('section')
  section.className = FOLD_CLASS
  heading.before(section)
  section.append(heading)
  for (const node of nodes) section.append(node)

  heading.dataset.markflyFold = key
  if (!heading.querySelector(`:scope > .${BTN_CLASS}`)) {
    // 标题正文包一层，方便 flex 居中，长标题也能正常换行
    const title = document.createElement('span')
    title.className = TITLE_CLASS
    while (heading.firstChild) title.appendChild(heading.firstChild)
    heading.append(makeFoldButton(), title)
  }

  const btn = heading.querySelector(`:scope > .${BTN_CLASS}`) as HTMLButtonElement | null
  btn?.addEventListener('click', (event) => {
    event.preventDefault()
    event.stopPropagation()
    setCollapsed(section, heading, !section.classList.contains(COLLAPSED_CLASS), key)
  })

  if (collapsedSet(docKeyGetter()).has(key)) {
    setCollapsed(section, heading, true, key)
  }
}

/** 点目录/锚点跳转前，把目标所在的折叠节展开，避免滚到看不见的地方 */
export const expandFoldsForElement = (target: Element) => {
  let node: HTMLElement | null = target instanceof HTMLElement ? target : target.parentElement
  while (node) {
    if (node.classList.contains(FOLD_CLASS) && node.classList.contains(COLLAPSED_CLASS)) {
      const heading = node.querySelector(
        ':scope > h1, :scope > h2, :scope > h3, :scope > h4, :scope > h5, :scope > h6'
      ) as HTMLElement | null
      if (heading) {
        const key = heading.dataset.markflyFold || headingKey(heading, 0)
        setCollapsed(node, heading, false, key)
      }
    }
    node = node.parentElement
  }
}

export const applyHeadingFold = (root: HTMLElement) => {
  const headings = [...root.querySelectorAll<HTMLElement>('h1,h2,h3,h4,h5,h6')].filter((heading) => {
    if (heading.closest('pre, code, svg, table')) return false
    return true
  })

  for (let index = headings.length - 1; index >= 0; index -= 1) {
    wrapHeading(headings[index], headingKey(headings[index], index))
  }
}

export const createHeadingFoldPlugin = (): BytemdPlugin => ({
  viewerEffect({ markdownBody }) {
    applyHeadingFold(markdownBody)
  },
})

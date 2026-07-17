type ZoomBinding = {
  detach: () => void
}

const zoomByContainer = new WeakMap<HTMLElement, ZoomBinding>()

const clamp = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value))

function openImageLightbox(source: HTMLImageElement) {
  const overlay = document.createElement('div')
  overlay.className = 'markfly-image-lightbox'

  const stage = document.createElement('div')
  stage.className = 'markfly-image-lightbox-stage'

  const image = document.createElement('img')
  image.className = 'markfly-image-lightbox-image'
  image.src = source.currentSrc || source.src
  image.alt = source.alt
  image.draggable = false

  const hint = document.createElement('div')
  hint.className = 'markfly-image-lightbox-hint'
  hint.textContent = '滚轮缩放 · 拖动平移 · Esc 关闭'

  stage.append(image)
  overlay.append(stage, hint)
  document.body.append(overlay)

  let scale = 1
  let translateX = 0
  let translateY = 0
  let dragging = false
  let dragStartX = 0
  let dragStartY = 0
  let dragOriginX = 0
  let dragOriginY = 0

  const applyTransform = () => {
    image.style.transform = `translate(${translateX}px, ${translateY}px) scale(${scale})`
  }

  const fitToViewport = () => {
    const maxWidth = window.innerWidth * 0.92
    const maxHeight = window.innerHeight * 0.88
    const naturalWidth = image.naturalWidth || source.naturalWidth || maxWidth
    const naturalHeight = image.naturalHeight || source.naturalHeight || maxHeight
    scale = Math.min(1, maxWidth / naturalWidth, maxHeight / naturalHeight)
    translateX = 0
    translateY = 0
    applyTransform()
  }

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') close()
  }

  const onWheel = (event: WheelEvent) => {
    event.preventDefault()
    const factor = event.deltaY > 0 ? 0.9 : 1.1
    scale = clamp(scale * factor, 0.2, 6)
    applyTransform()
  }

  const onPointerDown = (event: PointerEvent) => {
    if (event.button !== 0) return
    dragging = true
    dragStartX = event.clientX
    dragStartY = event.clientY
    dragOriginX = translateX
    dragOriginY = translateY
    image.setPointerCapture(event.pointerId)
    image.classList.add('is-dragging')
  }

  const onPointerMove = (event: PointerEvent) => {
    if (!dragging) return
    translateX = dragOriginX + (event.clientX - dragStartX)
    translateY = dragOriginY + (event.clientY - dragStartY)
    applyTransform()
  }

  const endDrag = (event: PointerEvent) => {
    if (!dragging) return
    dragging = false
    image.classList.remove('is-dragging')
    if (image.hasPointerCapture(event.pointerId)) {
      image.releasePointerCapture(event.pointerId)
    }
  }

  const onOverlayClick = (event: MouseEvent) => {
    if (event.target === overlay || event.target === stage) close()
  }

  const close = () => {
    document.removeEventListener('keydown', onKeyDown)
    overlay.remove()
  }

  image.addEventListener('load', fitToViewport, { once: true })
  if (image.complete) fitToViewport()

  overlay.addEventListener('click', onOverlayClick)
  stage.addEventListener('wheel', onWheel, { passive: false })
  image.addEventListener('pointerdown', onPointerDown)
  image.addEventListener('pointermove', onPointerMove)
  image.addEventListener('pointerup', endDrag)
  image.addEventListener('pointercancel', endDrag)
  document.addEventListener('keydown', onKeyDown)

  requestAnimationFrame(() => overlay.classList.add('is-visible'))
}

/** 预览区图片：点击查看大图，支持滚轮缩放与拖动 */
export function bindPreviewImageZoom(container: HTMLElement | null): void {
  if (!container) return

  zoomByContainer.get(container)?.detach()

  const images = [...container.querySelectorAll('img')].filter((img) => !img.closest('a'))
  if (images.length === 0) {
    zoomByContainer.delete(container)
    return
  }

  const bindings = images.map((img) => {
    const handler = (event: Event) => {
      event.preventDefault()
      event.stopPropagation()
      openImageLightbox(img)
    }
    img.addEventListener('click', handler)
    return { img, handler }
  })

  zoomByContainer.set(container, {
    detach: () => {
      bindings.forEach(({ img, handler }) => img.removeEventListener('click', handler))
    },
  })
}

export function unbindPreviewImageZoom(container: HTMLElement | null): void {
  if (!container) return
  zoomByContainer.get(container)?.detach()
  zoomByContainer.delete(container)
}

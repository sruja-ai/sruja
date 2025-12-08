import { useEffect } from 'react'
import { SrujaLoader } from '@sruja/ui'
import { createRoot } from 'react-dom/client'
import { initWasm } from '@sruja/shared'
import { createViewer } from '@sruja/viewer'
import LZString from 'lz-string'

async function renderSvg(preview: HTMLElement, dsl: string) {
  preview.innerHTML = ''
  const loaderContainer = document.createElement('div')
  loaderContainer.style.display = 'flex'
  loaderContainer.style.alignItems = 'center'
  loaderContainer.style.justifyContent = 'center'
  loaderContainer.style.padding = '16px'
  const loaderRoot = createRoot(loaderContainer)
  preview.appendChild(loaderContainer)
  loaderRoot.render(
    (<div style={{ textAlign: 'center', color: 'var(--color-text-secondary)' }}>
      <SrujaLoader size={32} />
      <div style={{ marginTop: 8, fontSize: 12 }}>Rendering diagram...</div>
    </div>)
  )
  try {
    const api = await initWasm()
    let svg: string | null = null
    try {
      svg = await api.dslToSvg(dsl)
    } catch (_) {
      const jsonStr = await api.parseDslToJson(dsl)
      const data = JSON.parse(jsonStr)
      const hidden = document.createElement('div')
      hidden.style.position = 'fixed'
      hidden.style.left = '-10000px'
      hidden.style.top = '0'
      hidden.style.width = '800px'
      hidden.style.height = '600px'
      document.body.appendChild(hidden)
      try {
        const viewer = createViewer({ container: hidden, data })
        await viewer.init()
        svg = viewer.exportSVG({ full: true, scale: 1 })
        viewer.destroy()
      } finally {
        document.body.removeChild(hidden)
      }
    }
    if (!svg) throw new Error('SVG export not available')
    // Wrap SVG to enable zoom/pan via CSS transform
    loaderRoot.unmount()
    preview.innerHTML = ''
    const inner = document.createElement('div')
    inner.style.transformOrigin = 'top left'
    inner.style.display = 'inline-block'
    inner.style.willChange = 'transform'
    inner.innerHTML = svg
    preview.appendChild(inner)
    preview.dataset.scale = '1'
    preview.dataset.tx = '0'
    preview.dataset.ty = '0'
    preview.dataset.inner = '1'
  } catch (e) {
    loaderRoot.unmount()
    const msg = e instanceof Error ? e.message : String(e)
    preview.innerHTML = `<div style=\"padding:8px;color:var(--color-error-500)\">Failed to render diagram: ${msg}</div>`
  }
}

function addToolbar(pre: HTMLElement, codeEl: HTMLElement, dsl: string) {
  const toolbar = document.createElement('div')
  toolbar.style.display = 'flex'
  toolbar.style.gap = '8px'
  toolbar.style.position = 'absolute'
  toolbar.style.right = '12px'
  toolbar.style.top = '12px'
  toolbar.style.zIndex = '10'

  const btnStyle = {
    padding: '6px 10px',
    borderRadius: '6px',
    border: '1px solid var(--color-border)',
    background: 'var(--color-background)',
    color: 'var(--color-text-primary)',
    cursor: 'pointer',
    fontSize: '12px',
  } as const

  const showBtn = document.createElement('button')
  showBtn.textContent = 'Show Diagram'
  Object.assign(showBtn.style, btnStyle)


  const viewBtn = document.createElement('button')
  viewBtn.textContent = 'Open in Viewer'
  Object.assign(viewBtn.style, btnStyle)
  viewBtn.onclick = () => {
    try {
      const b64 = encodeURIComponent(LZString.compressToBase64(dsl))
      window.open(`/viewer#code=${b64}`, '_blank')
    } catch (e) {
      window.open(`/viewer?code=${encodeURIComponent(dsl)}`, '_blank')
    }
    try { window.dispatchEvent(new CustomEvent('sruja:event', { detail: { type: 'tutorial.open_viewer' } })) } catch {}
  }

  toolbar.appendChild(showBtn)
  toolbar.appendChild(viewBtn)

  // Position wrapper
  const wrapper = document.createElement('div')
  wrapper.style.position = 'relative'
  pre.parentElement?.insertBefore(wrapper, pre)
  wrapper.appendChild(pre)
  wrapper.appendChild(toolbar)

  // Preview container below code
  const preview = document.createElement('div')
  preview.style.border = '1px solid var(--color-border)'
  preview.style.borderRadius = '6px'
  preview.style.marginTop = '12px'
  preview.style.overflow = 'hidden'
  preview.style.background = 'var(--color-background)'
  preview.style.maxHeight = '480px'
  preview.style.position = 'relative'
  preview.style.display = 'none'
  wrapper.appendChild(preview)

  const overlayControls = document.createElement('div')
  overlayControls.style.position = 'absolute'
  overlayControls.style.top = '8px'
  overlayControls.style.right = '8px'
  overlayControls.style.display = 'flex'
  overlayControls.style.gap = '6px'
  overlayControls.style.zIndex = '5'
  const mkBtn = (label: string) => {
    const b = document.createElement('button')
    Object.assign(b.style, btnStyle)
    b.textContent = label
    return b
  }
  const oZoomIn = mkBtn('+')
  const oZoomOut = mkBtn('-')
  const oReset = mkBtn('Reset')
  const oExpand = mkBtn('Expand')
  overlayControls.appendChild(oZoomIn)
  overlayControls.appendChild(oZoomOut)
  overlayControls.appendChild(oReset)
  overlayControls.appendChild(oExpand)
  overlayControls.style.display = 'none'
  preview.appendChild(overlayControls)

  const renderOnce = () => {
    if (preview.dataset.rendered === '1') return
    preview.dataset.rendered = '1'
    renderSvg(preview, dsl)
    overlayControls.style.display = 'flex'
  }

  // Show/Hide toggle
  showBtn.onclick = () => {
    const visible = preview.style.display !== 'none'
    if (visible) {
      preview.style.display = 'none'
      showBtn.textContent = 'Show Diagram'
    } else {
      try { window.dispatchEvent(new CustomEvent('sruja:event', { detail: { type: 'tutorial.diagram_show' } })) } catch {}
      renderOnce()
      preview.style.display = 'block'
      showBtn.textContent = 'Hide Diagram'
    }
  }

  const applyScale = (delta: number | null) => {
    const current = parseFloat(preview.dataset.scale || '1')
    const next = delta === null ? 1 : Math.max(0.25, Math.min(4, current + delta))
    const tx = parseFloat(preview.dataset.tx || '0')
    const ty = parseFloat(preview.dataset.ty || '0')
    applyTransform(next, tx, ty)
  }

  oZoomIn.onclick = () => {
    renderOnce()
    applyScale(0.25)
  }
  oZoomOut.onclick = () => {
    renderOnce()
    applyScale(-0.25)
  }
  oReset.onclick = () => {
    renderOnce()
    applyScale(null)
  }

  oExpand.onclick = async () => {
    try { window.dispatchEvent(new CustomEvent('sruja:event', { detail: { type: 'tutorial.diagram_expand' } })) } catch {}
    // Fullscreen overlay with interactive viewer
    const overlay = document.createElement('div')
    overlay.style.position = 'fixed'
    overlay.style.top = '0'
    overlay.style.left = '0'
    overlay.style.right = '0'
    overlay.style.bottom = '0'
    overlay.style.background = 'var(--overlay-scrim)'
    overlay.style.zIndex = '10000'
    overlay.style.display = 'flex'
    overlay.style.alignItems = 'center'
    overlay.style.justifyContent = 'center'

    const panel = document.createElement('div')
    panel.style.background = 'var(--color-background)'
    panel.style.width = '90vw'
    panel.style.height = '85vh'
    panel.style.borderRadius = '8px'
    panel.style.boxShadow = '0 20px 25px -5px rgba(0,0,0,0.1), 0 10px 10px -5px rgba(0,0,0,0.04)'
    panel.style.position = 'relative'

    const close = document.createElement('button')
    close.textContent = 'Close'
    Object.assign(close.style, btnStyle)
    close.style.position = 'absolute'
    close.style.top = '12px'
    close.style.right = '12px'

    const content = document.createElement('div')
    content.style.width = '100%'
    content.style.height = '100%'
    content.style.borderTopLeftRadius = '8px'
    content.style.borderTopRightRadius = '8px'
    content.style.overflow = 'hidden'

    panel.appendChild(close)
    panel.appendChild(content)
    overlay.appendChild(panel)
    document.body.appendChild(overlay)

    close.onclick = () => {
      document.body.removeChild(overlay)
    }

    const overlayLoader = document.createElement('div')
    overlayLoader.style.display = 'flex'
    overlayLoader.style.alignItems = 'center'
    overlayLoader.style.justifyContent = 'center'
    overlayLoader.style.height = '100%'
    const overlayRoot = createRoot(overlayLoader)
    content.appendChild(overlayLoader)
    overlayRoot.render(
      (<div style={{ textAlign: 'center', color: 'var(--color-text-tertiary)' }}>
        <SrujaLoader size={32} />
        <div style={{ marginTop: 8, fontSize: 12 }}>Initializing viewer...</div>
      </div>)
    )
    try {
      const api = await initWasm()
      const jsonStr = await api.parseDslToJson(dsl)
      const data = JSON.parse(jsonStr)
      const viewer = createViewer({ container: content, data })
      await viewer.init()
      overlayRoot.unmount()
      content.removeChild(overlayLoader)
    } catch (e) {
      overlayRoot.unmount()
      content.innerHTML = `<div style="padding:16px;color:var(--color-error-500)">Failed to open viewer</div>`
    }
  }

  let dragging = false
  let startX = 0
  let startY = 0
  preview.addEventListener('mousedown', (e) => {
    const scale = parseFloat(preview.dataset.scale || '1')
    if (scale <= 1) return
    dragging = true
    startX = e.clientX
    startY = e.clientY
    e.preventDefault()
  })
  window.addEventListener('mouseup', () => { dragging = false })
  window.addEventListener('mousemove', (e) => {
    if (!dragging) return
    const scale = parseFloat(preview.dataset.scale || '1')
    const tx = parseFloat(preview.dataset.tx || '0')
    const ty = parseFloat(preview.dataset.ty || '0')
    const dx = e.clientX - startX
    const dy = e.clientY - startY
    startX = e.clientX
    startY = e.clientY
    applyTransform(scale, tx + dx, ty + dy)
  })

  // Auto-render once the block is in viewport
  // Disable auto-render; user controls visibility

  const applyTransform = (scale: number, tx: number, ty: number) => {
    const inner = preview.firstElementChild as HTMLElement | null
    if (!inner) return
    inner.style.transform = `translate(${tx}px, ${ty}px) scale(${scale})`
    preview.dataset.scale = String(scale)
    preview.dataset.tx = String(tx)
    preview.dataset.ty = String(ty)
    preview.style.overflow = scale > 1 ? 'auto' : 'hidden'
  }
}

export default function CodeBlockActions() {
  useEffect(() => {
    const findBlocks = () => {
      const candidates = Array.from(document.querySelectorAll('pre > code')) as HTMLElement[]
      candidates.forEach((codeEl) => {
        const lang = codeEl.className || ''
        const isSruja = /language-sruja|lang-sruja/i.test(lang) || (codeEl.getAttribute('data-language') === 'sruja')
        const text = codeEl.textContent || ''
        if (!isSruja && !/\barchitecture\b|\bsystem\b|\bcontainer\b|\bscenario\b/.test(text)) return
        const pre = codeEl.parentElement as HTMLElement
        if (!pre) return
        if (pre.dataset.srujaToolbar === '1') return
        if (!text.trim()) return
        addToolbar(pre, codeEl, text)
        pre.dataset.srujaToolbar = '1'
      })
    }

    findBlocks()

    const mo = new MutationObserver(() => findBlocks())
    mo.observe(document.body, { childList: true, subtree: true })
    return () => mo.disconnect()
  }, [])
  return null
}

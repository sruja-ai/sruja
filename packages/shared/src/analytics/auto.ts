import { capture } from './posthog'

type AutoCfg = {
  clicks?: boolean
  pageviews?: boolean
  attribute?: string
  changes?: boolean
}

let installed = false

function textOf(el: Element): string {
  const t = (el as any).innerText || (el as any).textContent || ''
  return String(t).trim().slice(0, 200)
}

function propsFrom(el: Element): Record<string, any> {
  const d: Record<string, any> = {}
  const ds = (el as HTMLElement).dataset || {}
  for (const k of Object.keys(ds)) d[`data_${k}`] = ds[k]
  const tag = el.tagName.toLowerCase()
  const href = (el as HTMLAnchorElement).href || undefined
  const role = (el as any).role || (el.getAttribute('role') || undefined)
  const name = (el as any).name || el.getAttribute('name') || undefined
  const id = el.id || undefined
  const cls = (el as any).className || ''
  return {
    tag,
    id,
    role,
    name,
    href,
    text: textOf(el),
    class: typeof cls === 'string' ? cls : undefined,
    ...d,
    path: typeof window !== 'undefined' ? window.location.pathname : undefined,
  }
}

function isClickable(el: Element): boolean {
  const tag = el.tagName.toLowerCase()
  if (tag === 'button' || tag === 'a' || tag === 'label') return true
  if ((el as HTMLElement).onclick) return true
  const role = el.getAttribute('role')
  if (role === 'button' || role === 'link' || role === 'menuitem') return true
  if (el instanceof HTMLElement) {
    const style = window.getComputedStyle(el)
    if (style.cursor === 'pointer') return true
  }
  return false
}

function isChangeTarget(el: Element): boolean {
  const tag = el.tagName.toLowerCase()
  if (tag === 'select') return true
  if (tag === 'input') {
    const type = (el as HTMLInputElement).type
    return type === 'checkbox' || type === 'radio' || type === 'range' || type === 'color'
  }
  return false
}

function onChange(e: Event, attr: string) {
  const target = e.target as Element | null
  if (!target) return
  const el = target.closest(`[${attr}]`) || target
  const optOut = el.getAttribute('data-track-ignore') === 'true'
  if (optOut) return
  if (!isChangeTarget(el) && !el.hasAttribute(attr)) return
  const component = el.getAttribute('data-component') || 'form'
  const action = el.getAttribute(attr) || 'change'
  let value: any = undefined
  const tag = el.tagName.toLowerCase()
  if (tag === 'select') value = (el as HTMLSelectElement).value
  else if (tag === 'input') {
    const input = el as HTMLInputElement
    if (input.type === 'checkbox' || input.type === 'radio') value = input.checked
    else value = input.value
  }
  const name = (el as any).name || el.getAttribute('name') || undefined
  capture(`interaction.${component}.${action}`, { ...propsFrom(el), name, value })
}

function onClick(e: Event, attr: string) {
  const target = e.target as Element | null
  if (!target) return
  const el = target.closest(`[${attr}]`) || target.closest('button, a, [role="button"], [role="link"], [data-action]') || target
  if (!el) return
  const optOut = el.getAttribute('data-track-ignore') === 'true'
  if (optOut) return
  if (!isClickable(el)) return
  const component = el.getAttribute('data-component') || 'ui'
  const action = el.getAttribute(attr) || 'click'
  capture(`interaction.${component}.${action}`, propsFrom(el))
}

function capturePage() {
  capture('page.view', {
    path: window.location.pathname,
    title: document.title,
  })
}

function enablePageviews() {
  capturePage()
  const origPush = history.pushState
  const origReplace = history.replaceState
  history.pushState = function (...args: any[]) {
    const r = origPush.apply(this, args as any)
    queueMicrotask(capturePage)
    return r
  }
  history.replaceState = function (...args: any[]) {
    const r = origReplace.apply(this, args as any)
    queueMicrotask(capturePage)
    return r
  }
  window.addEventListener('popstate', () => capturePage())
}

export function enableAutoTracking(cfg: AutoCfg = {}) {
  if (typeof window === 'undefined' || installed) return
  installed = true
  const attr = cfg.attribute || 'data-track'
  const useClicks = cfg.clicks !== false
  const usePages = cfg.pageviews !== false
  const useChanges = cfg.changes !== false
  if (useClicks) window.addEventListener('click', (e) => onClick(e, attr), { capture: true })
  if (usePages) enablePageviews()
  if (useChanges) window.addEventListener('change', (e) => onChange(e, attr), { capture: true })
}

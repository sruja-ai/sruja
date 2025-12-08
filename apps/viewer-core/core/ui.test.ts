import { describe, it, expect, beforeEach } from 'vitest'
import { SrujaViewerUI } from './ui'

const SAMPLE_DATA: any = {
  architecture: {
    systems: [{ id: 'WebApp', label: 'Web App' }],
    requirements: [{ id: 'REQ-1', title: 'Login' }],
    adrs: [{ id: 'ADR-1', title: 'Use Postgres', status: 'accepted' }],
    scenarios: [{ id: 'S1', steps: [{ from: 'User', to: 'WebApp' }] }],
  },
}

function createMockViewer() {
  const styles: Record<string, any> = {}
  const elements: any[] = []
  return {
    cyInstance: {
      getElementById: (id: string) => {
        const node = { id, length: 1, addClass: (c: string) => {}, style: (k: string, v: string) => { styles[`${id}:${k}`] = v } }
        return node as any
      },
      elements: () => ({
        removeClass: () => {},
        style: () => {},
        length: elements.length,
      }) as any,
      fit: () => {},
    },
  }
}

describe('SrujaViewerUI', () => {
  let container: HTMLElement
  beforeEach(() => {
    container = document.createElement('div')
    document.body.innerHTML = ''
    document.body.appendChild(container)
  })

  it('creates tabs and content areas', async () => {
    const ui = new SrujaViewerUI({ container, data: SAMPLE_DATA, viewer: createMockViewer() as any })
    await ui.init()
    expect(container.querySelector('.sruja-tabs')).toBeTruthy()
    expect(container.querySelector('#sruja-tab-diagram')).toBeTruthy()
    expect(container.querySelector('#sruja-tab-requirements')).toBeTruthy()
    expect(container.querySelector('#sruja-tab-adrs')).toBeTruthy()
    expect(container.querySelector('#sruja-tab-scenarios')).toBeTruthy()
  })

  it('switches tabs and toggles content visibility', async () => {
    const ui = new SrujaViewerUI({ container, data: SAMPLE_DATA, viewer: createMockViewer() as any })
    await ui.init()
    const reqTab = container.querySelector('.sruja-tab[data-tab="requirements"]') as HTMLElement
    expect(reqTab).toBeTruthy()
    reqTab.click()
    const reqContent = container.querySelector('#sruja-tab-requirements') as HTMLElement
    const diagramContent = container.querySelector('#sruja-tab-diagram') as HTMLElement
    expect(reqContent.style.display).toBe('flex')
    expect(diagramContent.style.display).toBe('none')
  })

  it('renders ADRs with sanitized HTML', async () => {
    const DATA = {
      architecture: {
        adrs: [{ id: 'ADR-1', title: '<b>X</b>', context: '<script>y</script>' }],
      },
    }
    const ui = new SrujaViewerUI({ container, data: DATA as any, viewer: createMockViewer() as any })
    await ui.init()
    const adrContent = container.querySelector('#sruja-tab-adrs') as HTMLElement
    expect(adrContent.innerHTML.includes('&lt;b&gt;X&lt;/b&gt;')).toBe(true)
    expect(adrContent.innerHTML.includes('<b>')).toBe(false)
    expect(adrContent.innerHTML.includes('&lt;script&gt;y&lt;/script&gt;')).toBe(true)
  })
})

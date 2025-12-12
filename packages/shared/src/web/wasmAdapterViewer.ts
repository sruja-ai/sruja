// packages/shared/src/web/wasmAdapterViewer.ts
// Minimal WASM adapter for viewer - only loads JSON to DSL conversion
// Markdown export now handled by TypeScript exporters
import { logger } from '../utils/logger'
import { convertDslToMarkdown } from './wasmAdapter'

export type WasmApiViewer = {
  printJsonToDsl: (json: string) => Promise<string>
  dslToMarkdown: (dsl: string) => Promise<string> // Uses TypeScript exporter internally
}

async function ensureScript(src: string): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    const script = document.createElement('script')
    script.src = src
    script.async = true
    script.onload = () => resolve()
    script.onerror = () => {
      const error = new Error('Failed to load ' + src)
      logger.error('Failed to load wasm script', { component: 'wasm', action: 'load_script', scriptUrl: src, error: error.message })
      reject(error)
    }
    document.head.appendChild(script)
  })
}

async function ensureGoRuntimeLoaded(execUrl: string, base: string): Promise<void> {
  if ((window as any).Go) return
  const candidates = [
    execUrl,
    base.replace(/\/?$/, '/') + 'wasm_exec.js',
    '/wasm_exec.js',
    'https://cdn.jsdelivr.net/gh/golang/go@go1.25.0/misc/wasm/wasm_exec.js'
  ]
  let loaded = false
  for (const url of candidates) {
    try {
      await ensureScript(url)
      if ((window as any).Go) {
        loaded = true;
        break;
      }
    } catch {
      void 0;
    }
  }
  if (!loaded) {
    const error = new Error('Failed to load wasm_exec.js')
    logger.error('Failed to load Go wasm runtime', { component: 'wasm', action: 'load_go_runtime', candidates })
    throw error
  }
}

export async function initWasmViewer(options?: { base?: string; skipGoLoad?: boolean }): Promise<WasmApiViewer> {
  const base = options?.base || '/'
  const wasmExecUrl = base.replace(/\/?$/, '/') + 'wasm/wasm_exec.js'
  const wasmUrl = base.replace(/\/?$/, '/') + 'wasm/sruja.wasm'

  if (!options?.skipGoLoad) {
    await ensureGoRuntimeLoaded(wasmExecUrl, base)
  } else {
    if (!(window as any).Go) {
      try {
        await ensureGoRuntimeLoaded(wasmExecUrl, base)
      } catch {
        void 0;
      }
    }
  }

  const GoCtor = (window as any).Go
  if (!GoCtor) {
    const error = new Error('wasm_exec.js not loaded')
    logger.error('Go constructor missing', { component: 'wasm', action: 'init', errorType: 'go_constructor_missing', error: error.message })
    throw error
  }
  const go = new GoCtor()
  const importObject: any = go.importObject || {}

  // Provide gojs alias when missing
  if (!importObject.gojs) {
    const envObj = importObject.env || {}
    importObject.gojs = Object.keys(envObj).length ? envObj : (importObject.go || {})
  }
  // Polyfill TinyGo scheduleTimeoutEvent if missing
  if (!importObject.gojs['runtime.scheduleTimeoutEvent']) {
    importObject.gojs['runtime.scheduleTimeoutEvent'] = (ms: number) => {
      setTimeout(() => {
        try {
          if (go._resume) {
            go._resume();
          }
        } catch {
          void 0;
        }
      }, ms)
    }
  }

  let instance: WebAssembly.Instance | null = null
  const candidates = [
    wasmUrl,
    base.replace(/\/?$/, '/') + 'sruja.wasm',
    '/sruja.wasm',
    base.replace(/\/?$/, '/') + 'viewer/wasm/sruja.wasm',
    '/wasm/sruja.wasm'
  ]
  let lastError: any = null
  for (const url of candidates) {
    try {
      const response = await fetch(url)
      if (!response.ok) throw new Error('Failed to fetch ' + url)
      const bytes = await response.arrayBuffer()
      const mod = await WebAssembly.instantiate(bytes, importObject)
      instance = mod.instance
      lastError = null
      break
    } catch (e) {
      lastError = e
      continue
    }
  }
  if (!instance) {
    const error = lastError || new Error('Failed to load sruja.wasm')
    logger.error('Failed to load sruja.wasm', { component: 'wasm', action: 'load_wasm', errorType: 'wasm_load_failure', candidates, retries: candidates.length, error: error instanceof Error ? error.message : String(error) })
    throw error
  }

  go.run(instance)

  // Wait only for required viewer function (JSON to DSL)
  // Markdown export now uses TypeScript exporter, no WASM function needed
  let retries = 0
  const maxRetries = 50
  while (retries < maxRetries) {
    const jsonToDslFn = (window as any).sruja_json_to_dsl
    if (jsonToDslFn) {
      break
    }
    await new Promise(r => setTimeout(r, 50))
    retries++
  }

  const jsonToDslFn = (window as any).sruja_json_to_dsl

  // Only require JSON to DSL function for viewer
  if (!jsonToDslFn) {
    const available = Object.keys(window).filter(k => k.startsWith('sruja_'))
    const error = new Error(`WASM functions not found. Missing: sruja_json_to_dsl. Available: ${available.join(', ') || 'none'}`)
    logger.error('WASM functions missing', { component: 'wasm', action: 'init', errorType: 'wasm_functions_missing', missing: ['sruja_json_to_dsl'], available, retries, windowKeys: Object.keys(window).filter(k => k.includes('sruja') || k.includes('wasm')), error: error.message })
    throw error
  }

  return {
    printJsonToDsl: async (json: string) => {
      try {
        const r = jsonToDslFn(json)
        if (!r || !r.ok) {
          const error = new Error(r?.error || 'print failed')
          logger.error('WASM print failed', { component: 'wasm', action: 'print_json', errorType: 'print_failure', errorCode: r?.error, jsonLength: json.length, error: error.message })
          throw error
        }
        return r.dsl
      } catch (error) {
        logger.error('WASM print exception', { component: 'wasm', action: 'print_json', errorType: 'print_exception', error: error instanceof Error ? error.message : String(error) })
        throw error
      }
    },
    dslToMarkdown: async (dsl: string) => {
      // Use TypeScript exporter via shared helper
      const markdown = await convertDslToMarkdown(dsl)
      if (!markdown) {
        throw new Error('Failed to convert DSL to Markdown')
      }
      return markdown
    },
  }
}

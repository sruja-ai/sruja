// packages/shared/src/web/wasmAdapter.ts
import { logger } from '../utils/logger'

export type WasmApi = {
  parseDslToJson: (dsl: string) => Promise<string>
  printJsonToDsl: (json: string) => Promise<string>
  dslToMarkdown: (dsl: string) => Promise<string>
  dslToSvg: (dsl: string) => Promise<string>
  dslToHtml: (dsl: string) => Promise<string>
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
    'https://cdn.jsdelivr.net/gh/golang/go@go1.22.0/misc/wasm/wasm_exec.js'
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
      // Ignore errors
    }
  }
  if (!loaded) {
    const error = new Error('Failed to load wasm_exec.js')
    logger.error('Failed to load Go wasm runtime', { component: 'wasm', action: 'load_go_runtime', candidates })
    throw error
  }
}

export async function initWasm(options?: { base?: string; skipGoLoad?: boolean }): Promise<WasmApi> {
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
      // Ignore errors
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
          // Ignore errors
        }
      }, ms)
    }
  }

  let instance: WebAssembly.Instance | null = null
  const candidates = [
    wasmUrl,
    base.replace(/\/?$/, '/') + 'sruja.wasm',
    '/sruja.wasm',
    base.replace(/\/?$/, '/') + 'studio/wasm/sruja.wasm',
    base.replace(/\/?$/, '/') + 'viewer/wasm/sruja.wasm',
    '/wasm/sruja.wasm'
  ]
  let lastError: any = null
  for (const url of candidates) {
    try {
      try {
        const resp = await WebAssembly.instantiateStreaming(fetch(url), importObject)
        instance = resp.instance
        lastError = null
        break
      } catch (e) {
        const response = await fetch(url)
        if (!response.ok) throw e
        const bytes = await response.arrayBuffer()
        const mod = await WebAssembly.instantiate(bytes, importObject)
        instance = mod.instance
        lastError = null
        break
      }
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

  // Wait for required functions to be registered
  let retries = 0
  const maxRetries = 100
  while ((!(window as any).sruja_parse_dsl || !(window as any).sruja_dsl_to_markdown) && retries < maxRetries) {
    await new Promise(r => setTimeout(r, 50))
    retries++
  }

  // Wait a bit more for optional functions (HTML, SVG) to be registered
  retries = 0
  while (retries < 50 && (!(window as any).sruja_dsl_to_html || !(window as any).sruja_dsl_to_svg)) {
    await new Promise(r => setTimeout(r, 50))
    retries++
  }

  const parseFn = (window as any).sruja_parse_dsl
  const jsonToDslFn = (window as any).sruja_json_to_dsl
  const mdFn = (window as any).sruja_dsl_to_markdown
  const svgFn = (window as any).sruja_dsl_to_svg // Optional function
  const htmlFn = (window as any).sruja_dsl_to_html // Optional function

  // Only require core functions, svg is optional
  if (!parseFn || !jsonToDslFn || !mdFn) {
    const missing = []
    if (!parseFn) missing.push('sruja_parse_dsl')
    if (!jsonToDslFn) missing.push('sruja_json_to_dsl')
    if (!mdFn) missing.push('sruja_dsl_to_markdown')
    const available = Object.keys(window).filter(k => k.startsWith('sruja_'))
    const error = new Error(`WASM functions not found. Missing: ${missing.join(', ')}. Available: ${available.join(', ') || 'none'}`)
    logger.error('WASM functions missing', { component: 'wasm', action: 'init', errorType: 'wasm_functions_missing', missing, available, retries, windowKeys: Object.keys(window).filter(k => k.includes('sruja') || k.includes('wasm')), error: error.message })
    throw error
  }

  // Note: sruja_dsl_to_svg is optional - only warn if actually needed

  return {
    parseDslToJson: async (dsl: string) => {
      try {
        const r = parseFn(dsl)
        if (!r || !r.ok) {
          const error = new Error(r?.error || 'parse failed')
          logger.error('WASM parse failed', { component: 'wasm', action: 'parse_dsl', errorType: 'parse_failure', errorCode: r?.error, dslLength: dsl.length, error: error.message })
          throw error
        }
        return r.json
      } catch (error) {
        logger.error('WASM parse exception', { component: 'wasm', action: 'parse_dsl', errorType: 'parse_exception', error: error instanceof Error ? error.message : String(error) })
        throw error
      }
    },
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
      try {
        const r = mdFn(dsl)
        if (!r || !r.ok) {
          const error = new Error(r?.error || 'markdown export failed')
          logger.error('WASM markdown export failed', { component: 'wasm', action: 'export_markdown', errorType: 'export_failure', errorCode: r?.error, error: error.message })
          throw error
        }
        return r.dsl
      } catch (error) {
        logger.error('WASM markdown export exception', { component: 'wasm', action: 'export_markdown', errorType: 'export_exception', error: error instanceof Error ? error.message : String(error) })
        throw error
      }
    },
    dslToSvg: async (dsl: string) => {
      try {
        if (!svgFn) {
          const error = new Error('SVG export is not available. The sruja_dsl_to_svg function is not present in the WASM module.')
          logger.warn('WASM svg export unavailable', { component: 'wasm', action: 'export_svg', errorType: 'function_unavailable', error: error.message })
          throw error
        }
        const r = svgFn(dsl)
        if (!r || !r.ok) {
          const error = new Error(r?.error || 'svg export failed')
          logger.error('WASM svg export failed', { component: 'wasm', action: 'export_svg', errorType: 'export_failure', errorCode: r?.error, error: error.message })
          throw error
        }
        return r.dsl
      } catch (error) {
        logger.error('WASM svg export exception', { component: 'wasm', action: 'export_svg', errorType: 'export_exception', error: error instanceof Error ? error.message : String(error) })
        throw error
      }
    },
    dslToHtml: async (dsl: string) => {
      try {
        if (!htmlFn) {
          const error = new Error('HTML export is not available. The sruja_dsl_to_html function is not present in the WASM module.')
          logger.warn('WASM html export unavailable', { component: 'wasm', action: 'export_html', errorType: 'function_unavailable', error: error.message })
          throw error
        }
        const r = htmlFn(dsl)
        if (!r || !r.ok) {
          const error = new Error(r?.error || 'html export failed')
          logger.error('WASM html export failed', { component: 'wasm', action: 'export_html', errorType: 'export_failure', errorCode: r?.error, error: error.message })
          throw error
        }
        return r.dsl
      } catch (error) {
        logger.error('WASM html export exception', { component: 'wasm', action: 'export_html', errorType: 'export_exception', error: error instanceof Error ? error.message : String(error) })
        throw error
      }
    },
  }
}

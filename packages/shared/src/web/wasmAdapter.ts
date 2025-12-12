// packages/shared/src/web/wasmAdapter.ts
import { logger } from '../utils/logger'
import { exportToMarkdown } from '../export/markdown'
import { generateSystemDiagramForArch } from '../export/mermaid'
import type { ArchitectureJSON } from '../types/architecture'

export type WasmApi = {
  parseDslToJson: (dsl: string) => Promise<string>
  printJsonToDsl: (json: string) => Promise<string>
  // Export functions (dslToMarkdown, dslToMermaid) removed - now handled by TypeScript exporters
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
  const wasmBaseUrl = base.replace(/\/?$/, '/') + 'wasm/sruja.wasm'
  // Add cache-busting query parameter to ensure fresh WASM is loaded (only for primary URL)
  const wasmUrl = wasmBaseUrl + '?t=' + Date.now()

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
    wasmUrl, // Primary URL with cache-busting
    wasmBaseUrl, // Fallback without cache-busting
    base.replace(/\/?$/, '/') + 'sruja.wasm',
    '/sruja.wasm',
    base.replace(/\/?$/, '/') + 'studio/wasm/sruja.wasm',
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

  // Start the Go runtime - this is asynchronous
  go.run(instance)

  // Give Go runtime a moment to start
  await new Promise(r => setTimeout(r, 100))

  // Wait for required functions to be registered
  let retries = 0
  const maxRetries = 150 // Increased to give more time
  while (!(window as any).sruja_parse_dsl && retries < maxRetries) {
    await new Promise(r => setTimeout(r, 50))
    retries++
  }

  if (retries >= maxRetries) {
    const available = Object.keys(window).filter(k => k.startsWith('sruja_'))
    logger.error('Required WASM functions not found after waiting', {
      component: 'wasm',
      action: 'wait_for_required',
      retries,
      available,
      missing: {
        parse: !(window as any).sruja_parse_dsl
      }
    })
  }

  const parseFn = (window as any).sruja_parse_dsl
  const jsonToDslFn = (window as any).sruja_json_to_dsl

  // Debug: Log all registered functions
  const allSrujaFunctions = Object.keys(window).filter(k => k.startsWith('sruja_'))
  logger.info('WASM functions loaded', {
    component: 'wasm',
    action: 'init_complete',
    available: allSrujaFunctions
  })

  // Only require core functions (parsing and JSON conversion)
  if (!parseFn || !jsonToDslFn) {
    const missing = []
    if (!parseFn) missing.push('sruja_parse_dsl')
    if (!jsonToDslFn) missing.push('sruja_json_to_dsl')
    const available = Object.keys(window).filter(k => k.startsWith('sruja_'))
    const error = new Error(`WASM functions not found. Missing: ${missing.join(', ')}. Available: ${available.join(', ') || 'none'}`)
    logger.error('WASM functions missing', { component: 'wasm', action: 'init', errorType: 'wasm_functions_missing', missing, available, retries, windowKeys: Object.keys(window).filter(k => k.includes('sruja') || k.includes('wasm')), error: error.message })
    throw error
  }

  return {
    parseDslToJson: async (dsl: string) => {
      try {
        const filename = typeof location !== 'undefined' ? (location.pathname || 'input.sruja') : 'input.sruja'
        const r = parseFn(dsl, filename)
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
    // Export functions (dslToMarkdown, dslToMermaid) removed - now handled by TypeScript exporters
  }
}

// Singleton WASM API instance
let wasmApi: WasmApi | null = null
let initPromise: Promise<WasmApi> | null = null

/**
 * Auto-detects the base URL for WASM files based on the current environment.
 * Handles Vite, Astro, and other frameworks automatically.
 */
function detectBaseUrl(): string {
  if (typeof window === 'undefined') {
    return '/'
  }

  // Try to get BASE_URL from import.meta.env (Vite/Astro)
  const envBase = (import.meta as any).env?.BASE_URL
  if (envBase) {
    return envBase
  }

  // Fallback: use current location pathname
  const pathname = window.location.pathname

  // For studio page, use root as base since WASM is at /wasm/
  if (pathname.startsWith('/studio')) {
    return '/'
  }

  // For other paths, ensure it ends with /
  return pathname.endsWith('/') ? pathname : pathname + '/'
}

/**
 * Initialize WASM with auto-detected base URL.
 * Uses singleton pattern to ensure WASM is only initialized once.
 */
export async function initWasmAuto(options?: { base?: string; skipGoLoad?: boolean }): Promise<WasmApi> {
  if (wasmApi) return wasmApi

  if (initPromise) return initPromise

  initPromise = (async () => {
    const base = options?.base ?? detectBaseUrl()
    wasmApi = await initWasm({ ...options, base })
    return wasmApi
  })()

  return initPromise
}

/**
 * Get the initialized WASM API, initializing if necessary.
 * Returns null if initialization fails.
 */
export async function getWasmApi(): Promise<WasmApi | null> {
  if (wasmApi) return wasmApi

  try {
    return await initWasmAuto()
  } catch (error) {
    logger.error('Failed to initialize WASM', {
      component: 'wasm',
      action: 'get_api',
      error: error instanceof Error ? error.message : String(error)
    })
    return null
  }
}

/**
 * Convert DSL string to Architecture JSON object.
 * Returns parsed JSON object if successful, null on error.
 */
export async function convertDslToJson(dsl: string): Promise<object | null> {
  const api = await getWasmApi()
  if (!api) {
    logger.error('WASM not available', { component: 'wasm', action: 'convert_dsl_to_json' })
    return null
  }

  try {
    const jsonString = await api.parseDslToJson(dsl)
    return JSON.parse(jsonString)
  } catch (error) {
    logger.error('DSL parse error', {
      component: 'wasm',
      action: 'convert_dsl_to_json',
      error: error instanceof Error ? error.message : String(error)
    })
    return null
  }
}

/**
 * Convert DSL string to Markdown string.
 * Returns markdown string if successful, null on error.
 * Uses TypeScript markdown exporter instead of WASM.
 */
export async function convertDslToMarkdown(dsl: string): Promise<string | null> {
  const api = await getWasmApi()
  if (!api) {
    logger.error('WASM not available', { component: 'wasm', action: 'convert_dsl_to_markdown' })
    return null
  }

  try {
    // Parse DSL to JSON using WASM (keep parsing in WASM per roadmap)
    const jsonStr = await api.parseDslToJson(dsl)
    const archJson = JSON.parse(jsonStr) as ArchitectureJSON
    // Use TypeScript markdown exporter
    return exportToMarkdown(archJson)
  } catch (error) {
    logger.error('DSL to Markdown conversion error', {
      component: 'wasm',
      action: 'convert_dsl_to_markdown',
      error: error instanceof Error ? error.message : String(error)
    })
    return null
  }
}

/**
 * Convert DSL string to Mermaid diagram string.
 * Returns mermaid diagram string if successful, null on error.
 * Uses TypeScript mermaid exporter instead of WASM.
 */
export async function convertDslToMermaid(dsl: string): Promise<string | null> {
  const api = await getWasmApi()
  if (!api) {
    logger.error('WASM not available', { component: 'wasm', action: 'convert_dsl_to_mermaid' })
    return null
  }

  try {
    // Parse DSL to JSON using WASM (keep parsing in WASM per roadmap)
    const jsonStr = await api.parseDslToJson(dsl)
    if (!jsonStr || jsonStr.trim().length === 0) {
      logger.error('WASM parser returned empty JSON', {
        component: 'wasm',
        action: 'convert_dsl_to_mermaid',
        dslLength: dsl.length
      })
      return null
    }

    const archJson = JSON.parse(jsonStr) as ArchitectureJSON
    if (!archJson || !archJson.architecture) {
      logger.error('Invalid architecture JSON structure', {
        component: 'wasm',
        action: 'convert_dsl_to_mermaid',
        hasArchitecture: !!archJson?.architecture
      })
      return null
    }

    // Use TypeScript mermaid exporter (NOT WASM - this is TypeScript code)
    try {
      const mermaidCode = generateSystemDiagramForArch(archJson)
      if (!mermaidCode || mermaidCode.trim().length === 0) {
        logger.error('TypeScript mermaid exporter returned empty result', {
          component: 'wasm',
          action: 'convert_dsl_to_mermaid',
          hasArchitecture: !!archJson?.architecture,
          architectureName: archJson?.architecture?.name
        })
        return null
      }

      return mermaidCode
    } catch (exporterError) {
      logger.error('TypeScript mermaid exporter threw error', {
        component: 'wasm',
        action: 'convert_dsl_to_mermaid',
        error: exporterError instanceof Error ? exporterError.message : String(exporterError),
        errorType: exporterError instanceof Error ? exporterError.constructor.name : typeof exporterError,
        stack: exporterError instanceof Error ? exporterError.stack : undefined
      })
      return null
    }
  } catch (error) {
    logger.error('DSL to Mermaid conversion error', {
      component: 'wasm',
      action: 'convert_dsl_to_mermaid',
      error: error instanceof Error ? error.message : String(error),
      errorType: error instanceof Error ? error.constructor.name : typeof error,
      stack: error instanceof Error ? error.stack : undefined
    })
    return null
  }
}

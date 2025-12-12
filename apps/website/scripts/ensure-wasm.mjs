import { spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, copyFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const projectRoot = join(__dirname, '..', '..', '..')
const websiteDir = join(projectRoot, 'apps', 'website')
const publicWasmDir = join(websiteDir, 'public', 'wasm')
const wasmOut = join(publicWasmDir, 'sruja.wasm')

try {
  if (!existsSync(publicWasmDir)) {
    mkdirSync(publicWasmDir, { recursive: true })
  }

  const variant = (process.env.SRUJA_WASM_VARIANT || 'full').toLowerCase()
  let target = 'wasm'
  if (variant === 'tiny') {
    target = 'wasm-tiny'
  }
  // Note: 'minimal' variant removed - no separate viewer component on website

  const build = spawnSync('bash', ['-lc', `make ${target}`], {
    cwd: projectRoot,
    stdio: 'inherit'
  })
  if (build.status !== 0) {
    process.exit(build.status || 1)
  }

  const goroot = spawnSync('bash', ['-lc', 'go env GOROOT'], { cwd: projectRoot, encoding: 'utf8' })
  const root = goroot.stdout.trim()
  if (root) {
    const wasmExecSrc = join(root, 'lib', 'wasm', 'wasm_exec.js')
    try {
      copyFileSync(wasmExecSrc, join(publicWasmDir, 'wasm_exec.js'))
    } catch { }
  }
} catch {
  // Ignore errors, dev server will still run; HTML preview will show a friendly error
}

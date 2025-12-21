Place the WebAssembly runtime and module here for the Studio.

Required files:
- sruja.wasm — compiled Sruja DSL engine.
- Choose ONE runtime script matching your compiler:
  - wasm_exec.js — standard Go runtime (for Go-compiled WASM)
  - wasm_exec_tinygo.js — TinyGo runtime (for TinyGo-compiled WASM)

Paths expected by the app:
- ${BASE_URL}wasm/sruja.wasm
- ${BASE_URL}wasm/wasm_exec.js or ${BASE_URL}wasm/wasm_exec_tinygo.js

With current Vite config (base: "/studio/"), the URLs resolve to:
- /studio/wasm/wasm_exec.js
- /studio/wasm/sruja.wasm

Build sruja.wasm from sources using either:
- Go: GOOS=js GOARCH=wasm go build -o sruja.wasm ./cmd/sruja_wasm
- TinyGo: tinygo build -o sruja.wasm -target wasm ./cmd/sruja_wasm

Copy the generated files into this folder before running build or serving the static site.
If you used TinyGo, ensure wasm_exec_tinygo.js is present. If you used Go, ensure wasm_exec.js is present.

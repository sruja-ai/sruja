Sruja WASM build instructions

Go build:
GOOS=js GOARCH=wasm go build -o ../apps/studio/public/wasm/sruja.wasm ./
cp "$(go env GOROOT)/misc/wasm/wasm_exec.js" ../apps/studio/public/wasm/wasm_exec.js

TinyGo build:
tinygo build -o ../apps/studio/public/wasm/sruja.wasm -target wasm ./
cp wasm_exec_tinygo.js ../apps/studio/public/wasm/wasm_exec_tinygo.js

Open:
npm run dev

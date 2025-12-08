GO_WASM=GOOS=js GOARCH=wasm go build -o ../apps/studio/public/wasm/sruja.wasm ./
GO_EXEC=$(shell go env GOROOT)/misc/wasm/wasm_exec.js

go:
	$(GO_WASM)
	cp "$(GO_EXEC)" ../apps/studio/public/wasm/wasm_exec.js

tinygo:
	tinygo build -o ../apps/studio/public/wasm/sruja.wasm -target wasm ./
	cp wasm_exec_tinygo.js ../apps/studio/public/wasm/wasm_exec_tinygo.js

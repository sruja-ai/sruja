//go:build js && wasm

// cmd/wasm/results.go
// Result formatting functions for WASM module
package main

import "syscall/js"

// result formats a result for parseDsl and jsonToDsl functions
func result(ok bool, data string, err string) interface{} {
	res := make(map[string]interface{}, 3)
	res["ok"] = ok
	if ok {
		res["json"] = data
		res["dsl"] = data
		res["data"] = data
	} else {
		res["error"] = err
	}
	return js.ValueOf(res)
}

// lspResult formats a result for LSP functions
func lspResult(ok bool, data interface{}, err string) interface{} {
	res := make(map[string]interface{}, 3)
	res["ok"] = ok
	if ok {
		if data != nil {
			if str, ok := data.(string); ok {
				res["data"] = str
			} else {
				res["data"] = data
			}
		} else {
			res["data"] = nil
		}
	} else {
		res["error"] = err
	}
	return js.ValueOf(res)
}

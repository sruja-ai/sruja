//go:build js && wasm

// cmd/wasm/results.go
// Result formatting functions for WASM module
package main

import (
	"encoding/json"
	"syscall/js"
)

// result formats a result for parseDsl and jsonToDsl functions.
// This is the legacy function for backward compatibility.
func result(ok bool, data interface{}, err string) interface{} {
	res := make(map[string]interface{}, 4)
	res["ok"] = ok
	if ok {
		res["data"] = data
		// Backward compatibility for older callers expecting json/dsl fields
		if str, ok := data.(string); ok {
			res["json"] = str
			res["dsl"] = str
		}
	} else {
		res["error"] = err
	}
	return js.ValueOf(res)
}

// resultWithError formats a result using structured ExportError.
// This provides better error context and error codes for TypeScript.
func resultWithError(err *ExportError) interface{} {
	if err == nil {
		return result(true, nil, "")
	}

	res := make(map[string]interface{}, 4)
	res["ok"] = false
	res["error"] = err.Message
	res["code"] = string(err.Code)

	if len(err.Context) > 0 {
		res["context"] = err.Context
	}

	return js.ValueOf(res)
}

// resultWithData formats a successful result with data.
func resultWithData(data interface{}) interface{} {
	res := make(map[string]interface{}, 3)
	res["ok"] = true
	res["data"] = data

	// Backward compatibility for older callers expecting json/dsl fields
	if str, ok := data.(string); ok {
		res["json"] = str
		res["dsl"] = str
	}

	return js.ValueOf(res)
}

// resultWithJSON formats a result by marshaling data to JSON.
// This is useful for complex types that need proper JSON serialization.
func resultWithJSON(data interface{}) interface{} {
	jsonBytes, err := json.Marshal(data)
	if err != nil {
		return resultWithError(NewExportError(ErrCodeExportFailed,
			"failed to marshal result to JSON").
			WithContext("marshalError", err.Error()))
	}

	return resultWithData(string(jsonBytes))
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

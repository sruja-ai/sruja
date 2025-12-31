//go:build js && wasm

package main

import (
	"syscall/js"
	"testing"
)

func TestResultWithError(t *testing.T) {
	tests := []struct {
		name    string
		err     *ExportError
		checkFn func(js.Value) bool
	}{
		{
			name: "error without context",
			err:  NewExportError(ErrCodeInvalidInput, "test error"),
			checkFn: func(v js.Value) bool {
				if !v.Get("ok").Bool() {
					return false
				}
				if v.Get("error").String() != "test error" {
					return false
				}
				if v.Get("code").String() != string(ErrCodeInvalidInput) {
					return false
				}
				return true
			},
		},
		{
			name: "error with context",
			err: NewExportError(ErrCodeInvalidInput, "test error").
				WithContext("key", "value"),
			checkFn: func(v js.Value) bool {
				if !v.Get("ok").Bool() {
					return false
				}
				if v.Get("error").String() != "test error" {
					return false
				}
				if v.Get("code").String() != string(ErrCodeInvalidInput) {
					return false
				}
				context := v.Get("context")
				if !context.Truthy() {
					return false
				}
				return true
			},
		},
		{
			name: "nil error",
			err:  nil,
			checkFn: func(v js.Value) bool {
				// nil error should still return a result
				return v.Truthy()
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := resultWithError(tt.err)
			if result == nil {
				t.Fatal("result should not be nil")
			}

			jsVal, ok := result.(js.Value)
			if !ok {
				t.Fatalf("result should be js.Value, got %T", result)
			}

			if !tt.checkFn(jsVal) {
				t.Error("result validation failed")
			}
		})
	}
}

func TestResultWithData(t *testing.T) {
	tests := []struct {
		name    string
		data    interface{}
		checkFn func(js.Value) bool
	}{
		{
			name: "string data",
			data: "test output",
			checkFn: func(v js.Value) bool {
				if !v.Get("ok").Bool() {
					return false
				}
				if v.Get("data").String() != "test output" {
					return false
				}
				// Check backward compatibility fields
				if v.Get("json").String() != "test output" {
					return false
				}
				if v.Get("dsl").String() != "test output" {
					return false
				}
				return true
			},
		},
		{
			name: "map data",
			data: map[string]interface{}{
				"key": "value",
			},
			checkFn: func(v js.Value) bool {
				if !v.Get("ok").Bool() {
					return false
				}
				data := v.Get("data")
				if !data.Truthy() {
					return false
				}
				return true
			},
		},
		{
			name: "nil data",
			data: nil,
			checkFn: func(v js.Value) bool {
				if !v.Get("ok").Bool() {
					return false
				}
				return true
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := resultWithData(tt.data)
			if result == nil {
				t.Fatal("result should not be nil")
			}

			jsVal, ok := result.(js.Value)
			if !ok {
				t.Fatalf("result should be js.Value, got %T", result)
			}

			if !tt.checkFn(jsVal) {
				t.Error("result validation failed")
			}
		})
	}
}

func TestResult(t *testing.T) {
	// Test legacy result function for backward compatibility
	tests := []struct {
		name    string
		ok      bool
		data    interface{}
		err     string
		checkFn func(js.Value) bool
	}{
		{
			name: "success with string data",
			ok:   true,
			data: "test output",
			err:  "",
			checkFn: func(v js.Value) bool {
				if !v.Get("ok").Bool() {
					return false
				}
				if v.Get("data").String() != "test output" {
					return false
				}
				if v.Get("json").String() != "test output" {
					return false
				}
				return true
			},
		},
		{
			name: "error case",
			ok:   false,
			data: nil,
			err:  "test error",
			checkFn: func(v js.Value) bool {
				if v.Get("ok").Bool() {
					return false
				}
				if v.Get("error").String() != "test error" {
					return false
				}
				return true
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := result(tt.ok, tt.data, tt.err)
			if result == nil {
				t.Fatal("result should not be nil")
			}

			jsVal, ok := result.(js.Value)
			if !ok {
				t.Fatalf("result should be js.Value, got %T", result)
			}

			if !tt.checkFn(jsVal) {
				t.Error("result validation failed")
			}
		})
	}
}

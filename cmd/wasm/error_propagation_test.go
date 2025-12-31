//go:build js && wasm

package main

import (
	"strings"
	"testing"
)

// TestErrorPropagation_InputValidation tests that input validation errors
// propagate correctly through the parsing pipeline
func TestErrorPropagation_InputValidation(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		filename string
		wantCode ErrorCode
	}{
		{
			name:     "empty input",
			input:    "",
			filename: "test.sruja",
			wantCode: ErrCodeInvalidInput,
		},
		{
			name:     "input too large",
			input:    string(make([]byte, MaxInputSize+1)),
			filename: "test.sruja",
			wantCode: ErrCodeInputTooLarge,
		},
		{
			name:     "invalid filename characters",
			input:    "model { system s1 }",
			filename: "test@file.sruja",
			wantCode: ErrCodeInvalidFilename,
		},
		{
			name:     "filename too long",
			input:    "model { system s1 }",
			filename: strings.Repeat("a", MaxFilenameLength+1),
			wantCode: ErrCodeInvalidFilename,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := parseAndValidate(tt.input, tt.filename)

			if result.Error == nil {
				t.Fatal("expected error, got nil")
			}

			if result.Error.Code != tt.wantCode {
				t.Errorf("expected error code %s, got %s", tt.wantCode, result.Error.Code)
			}

			// Verify error message is present
			if result.Error.Message == "" {
				t.Error("error should have a message")
			}

			// Verify context is present for size errors
			if tt.wantCode == ErrCodeInputTooLarge {
				if result.Error.Context["size"] == nil {
					t.Error("InputTooLarge error should have size in context")
				}
				if result.Error.Context["maxSize"] == nil {
					t.Error("InputTooLarge error should have maxSize in context")
				}
			}
		})
	}
}

// TestErrorPropagation_ParseErrors tests that parse errors propagate correctly
func TestErrorPropagation_ParseErrors(t *testing.T) {
	tests := []struct {
		name     string
		dsl      string
		filename string
		wantCode ErrorCode
	}{
		{
			name:     "syntax error",
			dsl:      "model { system s1 // missing closing brace",
			filename: "test.sruja",
			wantCode: ErrCodeParseFailed,
		},
		{
			name:     "invalid syntax",
			dsl:      "invalid syntax here",
			filename: "test.sruja",
			wantCode: ErrCodeParseFailed,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := parseAndValidate(tt.dsl, tt.filename)

			if result.Error == nil {
				t.Fatal("expected error, got nil")
			}

			if result.Error.Code != tt.wantCode {
				t.Errorf("expected error code %s, got %s", tt.wantCode, result.Error.Code)
			}

			// Parse errors should have context with filename and inputLength
			if result.Error.Context["filename"] == nil {
				t.Error("parse error should have filename in context")
			}
			if result.Error.Context["inputLength"] == nil {
				t.Error("parse error should have inputLength in context")
			}
		})
	}
}

// TestErrorPropagation_ModelErrors tests that model validation errors propagate correctly
func TestErrorPropagation_ModelErrors(t *testing.T) {
	tests := []struct {
		name     string
		dsl      string
		filename string
		wantCode ErrorCode
	}{
		{
			name:     "empty model",
			dsl:      "model { }",
			filename: "test.sruja",
			wantCode: ErrCodeEmptyModel,
		},
		{
			name:     "model with only comments",
			dsl:      "model { // comment only }",
			filename: "test.sruja",
			wantCode: ErrCodeEmptyModel,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := parseAndValidate(tt.dsl, tt.filename)

			if result.Error == nil {
				t.Fatal("expected error, got nil")
			}

			if result.Error.Code != tt.wantCode {
				t.Errorf("expected error code %s, got %s", tt.wantCode, result.Error.Code)
			}

			// Empty model errors should have filename in context
			if result.Error.Context["filename"] == nil {
				t.Error("empty model error should have filename in context")
			}
		})
	}
}

// TestErrorPropagation_ContextChain tests that error context is preserved through the chain
func TestErrorPropagation_ContextChain(t *testing.T) {
	// Test that context from validation is preserved
	result := parseAndValidate("", "test.sruja")

	if result.Error == nil {
		t.Fatal("expected error")
	}

	// Verify error has all expected fields
	if result.Error.Code == "" {
		t.Error("error should have a code")
	}
	if result.Error.Message == "" {
		t.Error("error should have a message")
	}

	// For input validation errors, context should be minimal
	// For parse errors, context should include filename and inputLength
	if result.Error.Code == ErrCodeParseFailed {
		if result.Error.Context["filename"] == nil {
			t.Error("parse error should have filename in context")
		}
		if result.Error.Context["inputLength"] == nil {
			t.Error("parse error should have inputLength in context")
		}
	}
}

// TestErrorPropagation_WrapError tests that WrapError preserves original error information
func TestErrorPropagation_WrapError(t *testing.T) {
	originalErr := &testErrorPropagation{msg: "original error message"}
	wrapped := WrapError(originalErr, ErrCodeExportFailed)

	if wrapped == nil {
		t.Fatal("wrapped error should not be nil")
	}

	if wrapped.Code != ErrCodeExportFailed {
		t.Errorf("expected code %s, got %s", ErrCodeExportFailed, wrapped.Code)
	}

	if wrapped.Message != "original error message" {
		t.Errorf("expected message 'original error message', got '%s'", wrapped.Message)
	}

	// Test that wrapping an ExportError returns it unchanged
	exportErr := NewExportError(ErrCodeInvalidInput, "export error")
	wrapped2 := WrapError(exportErr, ErrCodeExportFailed)

	if wrapped2 != exportErr {
		t.Error("wrapping an ExportError should return it unchanged")
	}
	if wrapped2.Code != ErrCodeInvalidInput {
		t.Error("wrapped ExportError should keep original code")
	}
}

// testErrorPropagation is a simple error for testing error propagation
type testErrorPropagation struct {
	msg string
}

func (e *testErrorPropagation) Error() string {
	return e.msg
}

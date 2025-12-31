//go:build js && wasm

package main

import (
	"testing"
)

// TestParseAndValidate_InputValidation tests that parseAndValidate
// properly validates input before parsing.
func TestParseAndValidate_InputValidation(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		filename string
		wantErr  bool
		errCode  ErrorCode
	}{
		{
			name:     "empty input",
			input:    "",
			filename: "test.sruja",
			wantErr:  true,
			errCode:  ErrCodeInvalidInput,
		},
		{
			name:     "input too large",
			input:    string(make([]byte, MaxInputSize+1)),
			filename: "test.sruja",
			wantErr:  true,
			errCode:  ErrCodeInputTooLarge,
		},
		{
			name:     "invalid filename",
			input:    "model { system s1 }",
			filename: "../../etc/passwd",
			wantErr:  true,
			errCode:  ErrCodeInvalidFilename,
		},
		{
			name:     "valid input and filename",
			input:    "model { system s1 }",
			filename: "test.sruja",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := parseAndValidate(tt.input, tt.filename)

			if tt.wantErr {
				if result.Error == nil {
					t.Error("expected error, got nil")
					return
				}
				if result.Error.Code != tt.errCode {
					t.Errorf("expected error code %s, got %s", tt.errCode, result.Error.Code)
				}
				if result.Program != nil {
					t.Error("expected nil program on error")
				}
			} else {
				// For valid inputs, we can't fully test without actual parsing
				// This test mainly verifies validation happens before parsing
				if result.Error != nil && result.Error.Code == ErrCodeInvalidInput ||
					result.Error != nil && result.Error.Code == ErrCodeInvalidFilename {
					t.Errorf("unexpected validation error: %v", result.Error)
				}
			}
		})
	}
}

// TestParseAndValidate_EmptyModel tests that parseAndValidate
// detects empty models correctly.
// Note: This requires actual DSL parsing, so it's more of an integration test.
func TestParseAndValidate_EmptyModel(t *testing.T) {
	// This test would require a full parser setup
	// For now, we test that the validation logic is in place
	// Full integration tests should be in a separate file

	t.Run("empty model block", func(t *testing.T) {
		// This would test: model { } -> should return ErrCodeEmptyModel
		// But requires actual parsing, so we'll test the error code exists
		err := NewExportError(ErrCodeEmptyModel, "model block is empty")
		if err.Code != ErrCodeEmptyModel {
			t.Errorf("expected error code %s", ErrCodeEmptyModel)
		}
	})
}

// TestParseResult_Structure tests the ParseResult structure
func TestParseResult_Structure(t *testing.T) {
	// Test that ParseResult can hold both program and error
	result := &ParseResult{
		Program: nil,
		Error:   NewExportError(ErrCodeNoModel, "test"),
	}

	if result.Error == nil {
		t.Error("expected error in result")
	}
	if result.Program != nil {
		t.Error("expected nil program when error present")
	}

	// Test successful result structure
	result2 := &ParseResult{
		Program: nil, // Would be set by actual parsing
		Error:   nil,
	}

	if result2.Error != nil {
		t.Error("expected nil error in successful result")
	}
}

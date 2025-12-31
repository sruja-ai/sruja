//go:build js && wasm

package main

import (
	"strings"
	"testing"
)

func TestNewExportError(t *testing.T) {
	err := NewExportError(ErrCodeInvalidInput, "test error")
	if err == nil {
		t.Fatal("NewExportError should not return nil")
	}
	if err.Code != ErrCodeInvalidInput {
		t.Errorf("expected code %s, got %s", ErrCodeInvalidInput, err.Code)
	}
	if err.Message != "test error" {
		t.Errorf("expected message 'test error', got '%s'", err.Message)
	}
	if err.Context == nil {
		t.Error("Context should be initialized")
	}
}

func TestExportError_Error(t *testing.T) {
	tests := []struct {
		name     string
		err      *ExportError
		expected string
	}{
		{
			name:     "error without context",
			err:      NewExportError(ErrCodeInvalidInput, "test error"),
			expected: "[VALID_2002] test error",
		},
		{
			name: "error with context",
			err: NewExportError(ErrCodeInvalidInput, "test error").
				WithContext("key", "value"),
			expected: "[VALID_2002] test error (context: map[key:value])",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := tt.err.Error()
			if !strings.Contains(got, "[VALID_2002]") {
				t.Errorf("error string should contain error code, got: %s", got)
			}
			if !strings.Contains(got, "test error") {
				t.Errorf("error string should contain message, got: %s", got)
			}
		})
	}
}

func TestExportError_WithContext(t *testing.T) {
	err := NewExportError(ErrCodeInvalidInput, "test")

	// Add first context
	err = err.WithContext("key1", "value1")
	if err.Context["key1"] != "value1" {
		t.Error("first context not added")
	}

	// Add second context
	err = err.WithContext("key2", "value2")
	if err.Context["key2"] != "value2" {
		t.Error("second context not added")
	}

	// Verify both contexts exist
	if len(err.Context) != 2 {
		t.Errorf("expected 2 context items, got %d", len(err.Context))
	}
}

func TestWrapError(t *testing.T) {
	tests := []struct {
		name     string
		input    error
		code     ErrorCode
		expected *ExportError
	}{
		{
			name:     "nil error",
			input:    nil,
			code:     ErrCodeUnknown,
			expected: nil,
		},
		{
			name:  "standard error",
			input: &testErrorValidation{msg: "standard error"},
			code:  ErrCodeParseFailed,
			expected: &ExportError{
				Code:    ErrCodeParseFailed,
				Message: "standard error",
			},
		},
		{
			name:  "already ExportError",
			input: NewExportError(ErrCodeInvalidInput, "export error"),
			code:  ErrCodeParseFailed,
			expected: &ExportError{
				Code:    ErrCodeInvalidInput, // Should keep original code
				Message: "export error",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := WrapError(tt.input, tt.code)

			if tt.expected == nil {
				if result != nil {
					t.Errorf("expected nil, got %v", result)
				}
				return
			}

			if result == nil {
				t.Fatal("expected non-nil ExportError")
			}

			if result.Code != tt.expected.Code {
				t.Errorf("expected code %s, got %s", tt.expected.Code, result.Code)
			}
			if result.Message != tt.expected.Message {
				t.Errorf("expected message '%s', got '%s'", tt.expected.Message, result.Message)
			}
		})
	}
}

// testErrorValidation is a simple error type for testing validation
type testErrorValidation struct {
	msg string
}

func (e *testErrorValidation) Error() string {
	return e.msg
}

func TestValidateInput(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		wantErr bool
		errCode ErrorCode
	}{
		{
			name:    "empty input",
			input:   "",
			wantErr: true,
			errCode: ErrCodeInvalidInput,
		},
		{
			name:    "valid input",
			input:   "model { system s1 }",
			wantErr: false,
		},
		{
			name:    "input too large",
			input:   strings.Repeat("a", MaxInputSize+1),
			wantErr: true,
			errCode: ErrCodeInputTooLarge,
		},
		{
			name:    "input at max size",
			input:   strings.Repeat("a", MaxInputSize),
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateInput(tt.input)

			if tt.wantErr {
				if err == nil {
					t.Error("expected error, got nil")
					return
				}
				if err.Code != tt.errCode {
					t.Errorf("expected error code %s, got %s", tt.errCode, err.Code)
				}
				// Check context for size errors
				if tt.errCode == ErrCodeInputTooLarge {
					if err.Context["size"] == nil {
						t.Error("expected size in context")
					}
					if err.Context["maxSize"] != MaxInputSize {
						t.Errorf("expected maxSize %d in context, got %v", MaxInputSize, err.Context["maxSize"])
					}
				}
			} else {
				if err != nil {
					t.Errorf("unexpected error: %v", err)
				}
			}
		})
	}
}

func TestValidateFilename(t *testing.T) {
	tests := []struct {
		name     string
		filename string
		wantErr  bool
		errCode  ErrorCode
	}{
		{
			name:     "empty filename (allowed)",
			filename: "",
			wantErr:  false,
		},
		{
			name:     "valid filename",
			filename: "test.sruja",
			wantErr:  false,
		},
		{
			name:     "filename with path",
			filename: "path/to/file.sruja",
			wantErr:  false,
		},
		{
			name:     "filename too long",
			filename: strings.Repeat("a", MaxFilenameLength+1),
			wantErr:  true,
			errCode:  ErrCodeInvalidFilename,
		},
		{
			name:     "filename at max length",
			filename: strings.Repeat("a", MaxFilenameLength),
			wantErr:  false,
		},
		{
			name:     "filename with invalid characters",
			filename: "test@file.sruja",
			wantErr:  true,
			errCode:  ErrCodeInvalidFilename,
		},
		{
			name:     "filename with spaces",
			filename: "test file.sruja",
			wantErr:  true,
			errCode:  ErrCodeInvalidFilename,
		},
		{
			name:     "filename with underscores (allowed)",
			filename: "test_file.sruja",
			wantErr:  false,
		},
		{
			name:     "filename with dashes (allowed)",
			filename: "test-file.sruja",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateFilename(tt.filename)

			if tt.wantErr {
				if err == nil {
					t.Error("expected error, got nil")
					return
				}
				if err.Code != tt.errCode {
					t.Errorf("expected error code %s, got %s", tt.errCode, err.Code)
				}
			} else {
				if err != nil {
					t.Errorf("unexpected error: %v", err)
				}
			}
		})
	}
}

func TestValidateViewLevel(t *testing.T) {
	tests := []struct {
		name    string
		level   int
		wantErr bool
		errCode ErrorCode
	}{
		{
			name:    "valid level 1",
			level:   1,
			wantErr: false,
		},
		{
			name:    "valid level 2",
			level:   2,
			wantErr: false,
		},
		{
			name:    "valid level 3",
			level:   3,
			wantErr: false,
		},
		{
			name:    "level too low",
			level:   0,
			wantErr: true,
			errCode: ErrCodeInvalidView,
		},
		{
			name:    "level too high",
			level:   4,
			wantErr: true,
			errCode: ErrCodeInvalidView,
		},
		{
			name:    "negative level",
			level:   -1,
			wantErr: true,
			errCode: ErrCodeInvalidView,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateViewLevel(tt.level)

			if tt.wantErr {
				if err == nil {
					t.Error("expected error, got nil")
					return
				}
				if err.Code != tt.errCode {
					t.Errorf("expected error code %s, got %s", tt.errCode, err.Code)
				}
				// Check context
				if err.Context["viewLevel"] != tt.level {
					t.Errorf("expected viewLevel %d in context, got %v", tt.level, err.Context["viewLevel"])
				}
			} else {
				if err != nil {
					t.Errorf("unexpected error: %v", err)
				}
			}
		})
	}
}

func TestValidateNodeSizes(t *testing.T) {
	tests := []struct {
		name      string
		sizesJson string
		wantErr   bool
		errCode   ErrorCode
	}{
		{
			name:      "empty JSON (allowed)",
			sizesJson: "",
			wantErr:   false,
		},
		{
			name:      "valid JSON",
			sizesJson: `{"node1": {"width": 100, "height": 50}}`,
			wantErr:   false,
		},
		{
			name:      "JSON too large",
			sizesJson: strings.Repeat("a", 1024*1024+1), // 1MB + 1 byte
			wantErr:   true,
			errCode:   ErrCodeInvalidInput,
		},
		{
			name:      "JSON at max size",
			sizesJson: strings.Repeat("a", 1024*1024), // 1MB
			wantErr:   false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateNodeSizes(tt.sizesJson)

			if tt.wantErr {
				if err == nil {
					t.Error("expected error, got nil")
					return
				}
				if err.Code != tt.errCode {
					t.Errorf("expected error code %s, got %s", tt.errCode, err.Code)
				}
			} else {
				if err != nil {
					t.Errorf("unexpected error: %v", err)
				}
			}
		})
	}
}

func TestSanitizeFilename(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "normal filename",
			input:    "test.sruja",
			expected: "test.sruja",
		},
		{
			name:     "filename with path traversal",
			input:    "../../etc/passwd",
			expected: "____etc_passwd",
		},
		{
			name:     "filename with slashes",
			input:    "path/to/file.sruja",
			expected: "path_to_file.sruja",
		},
		{
			name:     "filename with backslashes",
			input:    "path\\to\\file.sruja",
			expected: "path_to_file.sruja",
		},
		{
			name:     "filename with null bytes",
			input:    "test\x00file.sruja",
			expected: "testfile.sruja",
		},
		{
			name:     "filename with leading/trailing spaces",
			input:    "  test.sruja  ",
			expected: "test.sruja",
		},
		{
			name:     "filename too long",
			input:    strings.Repeat("a", MaxFilenameLength+100),
			expected: strings.Repeat("a", MaxFilenameLength),
		},
		{
			name:     "empty filename",
			input:    "",
			expected: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := SanitizeFilename(tt.input)
			if got != tt.expected {
				t.Errorf("expected '%s', got '%s'", tt.expected, got)
			}
		})
	}
}

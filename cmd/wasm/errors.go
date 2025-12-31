//go:build js && wasm

// Package main provides structured error handling for WASM exports.
package main

import (
	"fmt"
	"regexp"
	"strings"
)

// ErrorCode represents a structured error code for WASM exports.
type ErrorCode string

const (
	// Parse errors (1000-1999)
	ErrCodeParseFailed   ErrorCode = "PARSE_1001"
	ErrCodeParseSyntax   ErrorCode = "PARSE_1002"
	ErrCodeParseSemantic ErrorCode = "PARSE_1003"

	// Validation errors (2000-2999)
	ErrCodeInvalidArgs     ErrorCode = "VALID_2001"
	ErrCodeInvalidInput    ErrorCode = "VALID_2002"
	ErrCodeInvalidView     ErrorCode = "VALID_2003"
	ErrCodeInvalidFilename ErrorCode = "VALID_2004"
	ErrCodeInputTooLarge   ErrorCode = "VALID_2005"

	// Model errors (3000-3999)
	ErrCodeNoModel    ErrorCode = "MODEL_3001"
	ErrCodeEmptyModel ErrorCode = "MODEL_3002"

	// Export errors (4000-4999)
	ErrCodeExportFailed  ErrorCode = "EXPORT_4001"
	ErrCodeExportEmpty   ErrorCode = "EXPORT_4002"
	ErrCodeExportTimeout ErrorCode = "EXPORT_4003"

	// System errors (5000-5999)
	ErrCodePanic   ErrorCode = "SYSTEM_5001"
	ErrCodeUnknown ErrorCode = "SYSTEM_5002"
)

// ExportError represents a structured error with code, message, and context.
type ExportError struct {
	Code    ErrorCode              `json:"code"`
	Message string                 `json:"message"`
	Context map[string]interface{} `json:"context,omitempty"`
}

// Error implements the error interface.
func (e *ExportError) Error() string {
	if len(e.Context) > 0 {
		return fmt.Sprintf("[%s] %s (context: %v)", e.Code, e.Message, e.Context)
	}
	return fmt.Sprintf("[%s] %s", e.Code, e.Message)
}

// NewExportError creates a new ExportError with the given code and message.
func NewExportError(code ErrorCode, message string) *ExportError {
	return &ExportError{
		Code:    code,
		Message: message,
		Context: make(map[string]interface{}),
	}
}

// WithContext adds context to the error.
func (e *ExportError) WithContext(key string, value interface{}) *ExportError {
	if e.Context == nil {
		e.Context = make(map[string]interface{})
	}
	e.Context[key] = value
	return e
}

// WrapError wraps a standard error into an ExportError.
func WrapError(err error, code ErrorCode) *ExportError {
	if err == nil {
		return nil
	}

	// If it's already an ExportError, return it
	if exportErr, ok := err.(*ExportError); ok {
		return exportErr
	}

	return NewExportError(code, err.Error())
}

// Validation constants
const (
	// MaxInputSize is the maximum allowed input size (10MB)
	MaxInputSize = 10 * 1024 * 1024

	// MaxViewLevel is the maximum C4 view level
	MaxViewLevel = 3

	// MinViewLevel is the minimum C4 view level
	MinViewLevel = 1

	// MaxFilenameLength is the maximum filename length
	MaxFilenameLength = 255

	// Valid filename pattern (alphanumeric, dots, dashes, underscores, slashes)
	filenamePattern = `^[a-zA-Z0-9._/-]+$`
)

var (
	filenameRegex = regexp.MustCompile(filenamePattern)
)

// ValidateInput validates the DSL input string.
func ValidateInput(input string) *ExportError {
	if input == "" {
		return NewExportError(ErrCodeInvalidInput, "input cannot be empty")
	}

	if len(input) > MaxInputSize {
		return NewExportError(ErrCodeInputTooLarge,
			fmt.Sprintf("input exceeds maximum size of %d bytes", MaxInputSize)).
			WithContext("size", len(input)).
			WithContext("maxSize", MaxInputSize)
	}

	return nil
}

// ValidateFilename validates a filename.
func ValidateFilename(filename string) *ExportError {
	if filename == "" {
		return nil // Empty filename is allowed (will use default)
	}

	if len(filename) > MaxFilenameLength {
		return NewExportError(ErrCodeInvalidFilename,
			fmt.Sprintf("filename exceeds maximum length of %d characters", MaxFilenameLength)).
			WithContext("length", len(filename)).
			WithContext("maxLength", MaxFilenameLength)
	}

	if !filenameRegex.MatchString(filename) {
		return NewExportError(ErrCodeInvalidFilename,
			"filename contains invalid characters").
			WithContext("filename", filename).
			WithContext("pattern", filenamePattern)
	}

	return nil
}

// ValidateViewLevel validates a C4 view level.
func ValidateViewLevel(level int) *ExportError {
	if level < MinViewLevel || level > MaxViewLevel {
		return NewExportError(ErrCodeInvalidView,
			fmt.Sprintf("view level must be between %d and %d, got %d",
				MinViewLevel, MaxViewLevel, level)).
			WithContext("viewLevel", level).
			WithContext("minLevel", MinViewLevel).
			WithContext("maxLevel", MaxViewLevel)
	}
	return nil
}

// ValidateNodeSizes validates node sizes JSON input.
func ValidateNodeSizes(sizesJson string) *ExportError {
	if sizesJson == "" {
		return nil // Empty is allowed
	}

	// Check size limit for JSON string
	if len(sizesJson) > 1024*1024 { // 1MB limit for node sizes
		return NewExportError(ErrCodeInvalidInput,
			"node sizes JSON exceeds maximum size of 1MB").
			WithContext("size", len(sizesJson))
	}

	return nil
}

// SanitizeFilename sanitizes a filename by removing dangerous characters.
func SanitizeFilename(filename string) string {
	// Remove any characters that could be problematic
	filename = strings.TrimSpace(filename)

	// Remove path traversal attempts
	filename = strings.ReplaceAll(filename, "..", "")
	filename = strings.ReplaceAll(filename, "/", "_")
	filename = strings.ReplaceAll(filename, "\\", "_")

	// Remove null bytes
	filename = strings.ReplaceAll(filename, "\x00", "")

	// Limit length
	if len(filename) > MaxFilenameLength {
		filename = filename[:MaxFilenameLength]
	}

	return filename
}

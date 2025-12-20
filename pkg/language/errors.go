// Package language provides DSL parsing and AST structures.
package language

import (
	"fmt"
	"strings"
)

// ErrorSeverity represents the severity of an error.
//
// Severity levels:
//   - error: Critical issue that prevents compilation/validation
//   - warning: Issue that should be addressed but doesn't prevent compilation
//   - info: Informational message (suggestions, best practices)
type ErrorSeverity string

const (
	ErrorSeverityError   ErrorSeverity = "error"   // Critical error
	ErrorSeverityWarning ErrorSeverity = "warning" // Warning
	ErrorSeverityInfo    ErrorSeverity = "info"    // Informational
)

// ParseError represents a parsing error.
//
// Parse errors occur when the DSL syntax is invalid. Examples:
//   - Missing closing brace
//   - Invalid keyword
//   - Malformed relation syntax
//
// Example:
//
//	ParseError{
//	    Location: SourceLocation{File: "example.sruja", Line: 5, Column: 12},
//	    Message:  "Expected ':' after identifier",
//	    Severity: ErrorSeverityError,
//	}
type ParseError struct {
	Location SourceLocation // Where the error occurred
	Message  string         // Error message
	Severity ErrorSeverity  // Error severity
	Err      error          // Wrapped sentinel error for errors.Is() support
}

// Error returns a formatted error string.
func (e *ParseError) Error() string {
	return fmt.Sprintf("%s: %s", e.Location, e.Message)
}

// Unwrap returns the wrapped error for errors.Is() and errors.As() support.
func (e *ParseError) Unwrap() error {
	return e.Err
}

// ValidationError represents a validation error.
//
// Validation errors occur when the architecture violates rules. Examples:
//   - Duplicate element IDs
//   - Invalid references (element doesn't exist)
//   - Circular dependencies
//   - Orphaned elements
//
// The RuleID identifies which validation rule was violated.
//
// Example:
//
//	ValidationError{
//	    Location: SourceLocation{File: "example.sruja", Line: 10, Column: 5},
//	    RuleID:   "semantic/duplicate-id",
//	    Message:  "Element 'API' is already defined",
//	    Severity: ErrorSeverityError,
//	}
type ValidationError struct {
	Location SourceLocation // Where the error occurred
	RuleID   string         // Validation rule ID (e.g., "semantic/duplicate-id")
	Message  string         // Error message
	Severity ErrorSeverity  // Error severity
	Err      error          // Wrapped sentinel error for errors.Is() support
}

// Error returns a formatted error string with rule ID.
func (e *ValidationError) Error() string {
	return fmt.Sprintf("%s [%s]: %s", e.Location, e.RuleID, e.Message)
}

// Unwrap returns the wrapped error for errors.Is() and errors.As() support.
func (e *ValidationError) Unwrap() error {
	return e.Err
}

// CompilationError represents a compilation error.
//
// Compilation errors occur when transforming AST to Model or Model to diagram format.
// Examples:
//   - Missing required fields
//   - Invalid element type
//   - Diagram generation failure
//
// Example:
//
//	CompilationError{
//	    Location: SourceLocation{File: "example.sruja", Line: 3, Column: 1},
//	    Message:  "System 'API' is missing required 'id' field",
//	    Severity: ErrorSeverityError,
//	}
type CompilationError struct {
	Location SourceLocation // Where the error occurred
	Message  string         // Error message
	Severity ErrorSeverity  // Error severity
	Err      error          // Wrapped sentinel error for errors.Is() support
}

// Error returns a formatted error string.
func (e *CompilationError) Error() string {
	return fmt.Sprintf("%s: %s", e.Location, e.Message)
}

// Unwrap returns the wrapped error for errors.Is() and errors.As() support.
func (e *CompilationError) Unwrap() error {
	return e.Err
}

// ErrorList represents a collection of errors.
//
// Used to collect multiple errors during parsing, validation, or compilation.
// This allows reporting all errors at once rather than stopping at the first error.
//
// Example usage:
//
//	var errors ErrorList
//	errors.Add(NewParseError(loc1, "Error 1"))
//	errors.Add(NewValidationError(loc2, "rule1", "Error 2"))
//	if errors.HasErrors() {
//	    return errors
//	}
type ErrorList struct {
	Errors []error // Collection of errors
}

// Add adds an error to the list.
// Pre-allocates slice capacity if needed for better performance.
func (el *ErrorList) Add(err error) {
	if err == nil {
		return
	}
	if el.Errors == nil {
		el.Errors = make([]error, 0, 8) // Pre-allocate small capacity
	}
	el.Errors = append(el.Errors, err)
}

// HasErrors returns true if there are any errors in the list.
func (el *ErrorList) HasErrors() bool {
	return len(el.Errors) > 0
}

// Error returns a formatted error string.
//
// If there are no errors, returns "no errors".
// If there is one error, returns that error's message.
// If there are multiple errors, returns a summary with the first error.
func (el *ErrorList) Error() string {
	if len(el.Errors) == 0 {
		return "no errors"
	}
	if len(el.Errors) == 1 {
		return el.Errors[0].Error()
	}
	// Build error message efficiently
	var sb strings.Builder
	firstErr := el.Errors[0].Error()
	sb.Grow(30 + len(firstErr))
	sb.WriteString(fmt.Sprintf("%d", len(el.Errors)))
	sb.WriteString(" errors (first: ")
	sb.WriteString(firstErr)
	sb.WriteString(")")
	return sb.String()
}

// NewParseError creates a new parse error with error severity.
// The error wraps ErrParse for use with errors.Is().
//
// Example:
//
//	err := NewParseError(
//	    SourceLocation{File: "example.sruja", Line: 5, Column: 12},
//	    "Expected ':' after identifier",
//	)
//	if errors.Is(err, ErrParse) { ... }
func NewParseError(loc SourceLocation, msg string) *ParseError {
	return &ParseError{
		Location: loc,
		Message:  msg,
		Severity: ErrorSeverityError,
		Err:      ErrParse,
	}
}

// NewValidationError creates a new validation error with error severity.
// The error wraps ErrValidation for use with errors.Is().
//
// Example:
//
//	err := NewValidationError(
//	    SourceLocation{File: "example.sruja", Line: 10, Column: 5},
//	    "semantic/duplicate-id",
//	    "Element 'API' is already defined",
//	)
//	if errors.Is(err, ErrValidation) { ... }
func NewValidationError(loc SourceLocation, ruleID, msg string) *ValidationError {
	return &ValidationError{
		Location: loc,
		RuleID:   ruleID,
		Message:  msg,
		Severity: ErrorSeverityError,
		Err:      ErrValidation,
	}
}

// NewCompilationError creates a new compilation error with error severity.
// The error wraps ErrCompilation for use with errors.Is().
//
// Example:
//
//	err := NewCompilationError(
//	    SourceLocation{File: "example.sruja", Line: 3, Column: 1},
//	    "System 'API' is missing required 'id' field",
//	)
//	if errors.Is(err, ErrCompilation) { ... }
func NewCompilationError(loc SourceLocation, msg string) *CompilationError {
	return &CompilationError{
		Location: loc,
		Message:  msg,
		Severity: ErrorSeverityError,
		Err:      ErrCompilation,
	}
}

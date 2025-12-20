// Package language provides sentinel errors for type-safe error checking.
// Use errors.Is() to check for these error types.
package language

import "errors"

// Sentinel errors for parsing
var (
	// ErrParse is the base error for all parsing errors.
	ErrParse = errors.New("parse error")

	// ErrSyntax indicates a syntax error in the DSL.
	ErrSyntax = errors.New("syntax error")

	// ErrUnexpectedToken indicates an unexpected token was encountered.
	ErrUnexpectedToken = errors.New("unexpected token")

	// ErrMissingBrace indicates a missing brace.
	ErrMissingBrace = errors.New("missing brace")
)

// Sentinel errors for validation
var (
	// ErrValidation is the base error for all validation errors.
	ErrValidation = errors.New("validation error")

	// ErrDuplicateID indicates a duplicate element ID was found.
	ErrDuplicateID = errors.New("duplicate ID")

	// ErrInvalidRef indicates an invalid reference to another element.
	ErrInvalidRef = errors.New("invalid reference")

	// ErrCycle indicates a circular dependency was detected.
	ErrCycle = errors.New("circular dependency")

	// ErrOrphan indicates an orphaned element (not connected to anything).
	ErrOrphan = errors.New("orphan element")

	// ErrLayerViolation indicates a layer architecture violation.
	ErrLayerViolation = errors.New("layer violation")
)

// Sentinel errors for compilation
var (
	// ErrCompilation is the base error for all compilation errors.
	ErrCompilation = errors.New("compilation error")

	// ErrMissingField indicates a required field is missing.
	ErrMissingField = errors.New("missing required field")

	// ErrInvalidType indicates an invalid type was used.
	ErrInvalidType = errors.New("invalid type")
)

// Sentinel errors for I/O
var (
	// ErrIO is the base error for all I/O errors.
	ErrIO = errors.New("I/O error")

	// ErrNotFound indicates that a resource was not found.
	ErrNotFound = errors.New("not found")

	// ErrPermission indicates a permission error.
	ErrPermission = errors.New("permission denied")
)

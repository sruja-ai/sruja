//go:build js && wasm

// Package main provides common parsing and validation logic for WASM exports.
package main

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// ParseResult contains the result of parsing and validation.
type ParseResult struct {
	Program *language.Program
	Error   *ExportError
}

// parseAndValidate parses DSL input and validates the resulting program.
// This centralizes the common parsing logic used across all export functions.
func parseAndValidate(input, filename string) *ParseResult {
	// Validate input
	if err := ValidateInput(input); err != nil {
		return &ParseResult{Error: err}
	}

	// Validate and sanitize filename
	if err := ValidateFilename(filename); err != nil {
		return &ParseResult{Error: err}
	}
	filename = SanitizeFilename(filename)
	if filename == "" {
		filename = defaultFilename
	}

	// Parse DSL
	_, program, err := parseToWorkspace(input, filename)
	if err != nil {
		return &ParseResult{
			Error: WrapError(err, ErrCodeParseFailed).
				WithContext("filename", filename).
				WithContext("inputLength", len(input)),
		}
	}

	// Validate program structure
	if program == nil {
		return &ParseResult{
			Error: NewExportError(ErrCodeNoModel, "program is nil").
				WithContext("filename", filename),
		}
	}

	if program.Model == nil {
		return &ParseResult{
			Error: NewExportError(ErrCodeNoModel, "no model found in program").
				WithContext("filename", filename),
		}
	}

	if len(program.Model.Items) == 0 {
		return &ParseResult{
			Error: NewExportError(ErrCodeEmptyModel, "model block is empty - no items found").
				WithContext("filename", filename),
		}
	}

	return &ParseResult{Program: program}
}

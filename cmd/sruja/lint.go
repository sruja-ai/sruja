package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func runLint(args []string, stdout, stderr io.Writer) int {
	lintCmd := flag.NewFlagSet("lint", flag.ContinueOnError)
	lintCmd.SetOutput(stderr)

	if err := lintCmd.Parse(args); err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing lint flags: %v", err)))
		return 1
	}

	if lintCmd.NArg() < 1 {
		_, _ = fmt.Fprintln(stderr, dx.Error("Usage: sruja lint <file>"))
		return 1
	}

	filePath := lintCmd.Arg(0)

	content, err := os.ReadFile(filePath)
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error reading file: %v", err)))
		return 1
	}

	p, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error creating parser: %v", err)))
		return 1
	}

	program, _, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Parser Error: %v", err)))
		return 1
	}

	// Validation
	validator := engine.NewValidator()
	validator.RegisterDefaultRules()

	diags := validator.Validate(program)

	// Filter diagnostics
	var blockingErrors []diagnostics.Diagnostic
	var warnings []diagnostics.Diagnostic

	for _, d := range diags {
		// Skip informational cycle detection messages (cycles are valid patterns)
		if d.Code == diagnostics.CodeCycleDetected && d.Severity == diagnostics.SeverityInfo {
			continue // Cycles are valid - skip informational messages
		}

		if d.Severity == diagnostics.SeverityError {
			blockingErrors = append(blockingErrors, d)
		} else if d.Severity == diagnostics.SeverityWarning {
			warnings = append(warnings, d)
		}
	}

	// Print warnings first (non-blocking)
	if len(warnings) > 0 {
		enhancer := dx.NewErrorEnhancer(filePath, strings.Split(string(content), "\n"), program)
		enhancedWarnings := make([]*dx.EnhancedError, 0, len(warnings))
		for _, w := range warnings {
			enhancedWarnings = append(enhancedWarnings, enhancer.Enhance(w))
		}
		// Print warnings but continue
		_, _ = fmt.Fprint(stderr, dx.FormatErrors(enhancedWarnings, dx.SupportsColor()))
	}

	if len(blockingErrors) > 0 {
		// Enhance errors with suggestions and context
		enhancer := dx.NewErrorEnhancer(filePath, strings.Split(string(content), "\n"), program)
		enhancedErrors := make([]*dx.EnhancedError, 0, len(blockingErrors))
		for _, err := range blockingErrors {
			enhancedErrors = append(enhancedErrors, enhancer.Enhance(err))
		}

		_, _ = fmt.Fprint(stderr, dx.FormatErrors(enhancedErrors, dx.SupportsColor()))
		return 1
	}

	_, _ = fmt.Fprintln(stdout, dx.Success("No linting errors found."))
	return 0
}

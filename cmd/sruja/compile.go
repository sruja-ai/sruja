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

func runCompile(args []string, stdout, stderr io.Writer) int {
	compileCmd := flag.NewFlagSet("compile", flag.ContinueOnError)
	compileCmd.SetOutput(stderr)

	if err := compileCmd.Parse(args); err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing compile flags: %v", err)))
		return 1
	}

	if compileCmd.NArg() < 1 {
		_, _ = fmt.Fprintln(stderr, dx.Error("Usage: sruja compile <file>"))
		return 1
	}

	filePath := compileCmd.Arg(0)
	info, err := os.Stat(filePath)
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error accessing path: %v", err)))
		return 1
	}

	p, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error creating parser: %v", err)))
		return 1
	}

	var program *language.Program
	var content string // For error context

	if info.IsDir() {
		ws, err := p.ParseWorkspace(filePath)
		if err != nil {
			_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Workspace Parser Error: %v", err)))
			return 1
		}
		// Resolve references across the workspace
		engine.RunWorkspaceResolution(ws)
		program = ws.MergedProgram()
		content = "// Merged workspace content"
	} else {
		fileContent, err := os.ReadFile(filePath)
		if err != nil {
			_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error reading file: %v", err)))
			return 1
		}
		content = string(fileContent)
		program, _, err = p.Parse(filePath, content)
		if err != nil {
			_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Parser Error: %v", err)))
			return 1
		}
		// Resolve references in the single file
		engine.RunResolution(program)
	}

	// Validation
	validator := engine.NewValidator()
	validator.RegisterDefaultRules()

	diags := validator.Validate(program)

	// Filter out informational cycle messages (cycles are valid in many architectures)
	var blockingErrors []diagnostics.Diagnostic
	//nolint:gocritic // copying small structs is fine
	for _, d := range diags {
		// Skip informational cycle detection messages (cycles are valid patterns)
		if d.Code == diagnostics.CodeCycleDetected && d.Severity == diagnostics.SeverityInfo {
			continue // Cycles are valid - skip informational messages
		}
		// Only consider Errors as blocking, unless we want to fail on Warnings too
		// For now, let's treat Errors as blocking
		if d.Severity == diagnostics.SeverityError {
			blockingErrors = append(blockingErrors, d)
		}
	}

	if len(blockingErrors) > 0 {
		// Enhance errors with suggestions and context
		enhancer := dx.NewErrorEnhancer(filePath, strings.Split(string(content), "\n"), program)
		enhancedErrors := make([]*dx.EnhancedError, 0, len(blockingErrors))
		//nolint:gocritic // copying small structs is fine
		for _, err := range blockingErrors {
			enhancedErrors = append(enhancedErrors, enhancer.Enhance(err))
		}

		_, _ = fmt.Fprint(stderr, dx.FormatErrors(enhancedErrors, dx.SupportsColor()))
		return 1
	}

	_, _ = fmt.Fprintln(stdout, dx.Success("Compilation successful."))
	return 0
}

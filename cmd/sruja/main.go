package main

import (
	"fmt"
	"os"
	"strings"

	"io"

	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/export/d2"
    "github.com/sruja-ai/sruja/pkg/export/svg"
    jexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/language"
)

const srujaFileExt = ".sruja"

// findSrujaFile finds a .sruja file in the current directory if filePath is empty
func findSrujaFile(filePath string) string {
	if filePath != "" {
		return filePath
	}
	files, err := os.ReadDir(".")
	if err != nil {
		return ""
	}
	for _, file := range files {
		if !file.IsDir() && len(file.Name()) > len(srujaFileExt) && file.Name()[len(file.Name())-len(srujaFileExt):] == srujaFileExt {
			return file.Name()
		}
	}
	return ""
}

// parseArchitectureFile parses an architecture file and returns the program
func parseArchitectureFile(filePath string, stderr io.Writer) (*language.Program, error) {
	content, err := os.ReadFile(filePath)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error reading file: %v\n", err)
		return nil, err
	}

	p, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error creating parser: %v\n", err)
		return nil, err
	}

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Parser Error: %v\n", err)
		return nil, err
	}

	return program, nil
}

func main() { os.Exit(Execute()) }

func Run(_ []string, _ io.Writer, _ io.Writer) int { return Execute() }

func runLint(filePath string, stdout, stderr io.Writer) int {
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

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Parser Error: %v", err)))
		return 1
	}

	// Validation
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	validator.RegisterRule(&engine.OrphanDetectionRule{})
	validator.RegisterRule(&engine.SimplicityRule{})

	validationErrors := validator.Validate(program)
	if len(validationErrors) > 0 {
		// Filter out informational cycle messages (cycles are valid in many architectures)
		var blockingErrors []engine.ValidationError
		for _, e := range validationErrors {
			// Skip informational cycle detection messages (cycles are valid patterns)
			if strings.Contains(e.Message, "Cycle detected") && strings.Contains(e.Message, "valid") {
				continue // Cycles are valid - skip informational messages
			}
			blockingErrors = append(blockingErrors, e)
		}
		validationErrors = blockingErrors
	}
	if len(validationErrors) > 0 {
		// Enhance errors with suggestions and context
		enhancer := dx.NewErrorEnhancer(filePath, strings.Split(string(content), "\n"), program)
		enhancedErrors := make([]*dx.EnhancedError, 0, len(validationErrors))
		for _, err := range validationErrors {
			enhancedErrors = append(enhancedErrors, enhancer.Enhance(err))
		}

		_, _ = fmt.Fprint(stderr, dx.FormatErrors(enhancedErrors, dx.SupportsColor()))
		return 1
	} else {
		_, _ = fmt.Fprintln(stdout, dx.Success("No linting errors found."))
		return 0
	}
}

func runFmt(filePath string, stdout, stderr io.Writer) int {
	content, err := os.ReadFile(filePath)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error reading file: %v\n", err)
		return 1
	}

	p, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error creating parser: %v\n", err)
		return 1
	}

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Parser Error: %v\n", err)
		return 1
	}

	printer := &language.Printer{}
	formatted := printer.Print(program)
	_, _ = fmt.Fprint(stdout, formatted)
	return 0
}

func runExport(format, filePath string, stdout, stderr io.Writer) int {
	content, err := os.ReadFile(filePath)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error reading file: %v\n", err)
		return 1
	}

	p, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error creating parser: %v\n", err)
		return 1
	}

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Parser Error: %v\n", err)
		return 1
	}

	var output string
    switch format {
    case "d2":
        exporter := d2.NewExporter()
        output, err = exporter.Export(program.Architecture)
    case "svg":
        exporter := svg.NewExporter()
        output, err = exporter.Export(program.Architecture)
    case "json":
        exporter := jexport.NewExporter()
        output, err = exporter.Export(program.Architecture)
    default:
        _, _ = fmt.Fprintf(stderr, "Unsupported export format: %s. Supported formats: d2, svg, json\n", format)
        return 1
    }

	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Export Error: %v\n", err)
		return 1
	}

	_, _ = fmt.Fprint(stdout, output)
	return 0
}

func runCompile(filePath string, stdout, stderr io.Writer) int {
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

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Parser Error: %v", err)))
		return 1
	}

	// Validation
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	validator.RegisterRule(&engine.OrphanDetectionRule{})
	validator.RegisterRule(&engine.SimplicityRule{})

	validationErrors := validator.Validate(program)
	if len(validationErrors) > 0 {
		// Filter out informational cycle messages (cycles are valid in many architectures)
		var blockingErrors []engine.ValidationError
		for _, e := range validationErrors {
			// Skip informational cycle detection messages (cycles are valid patterns)
			if strings.Contains(e.Message, "Cycle detected") && strings.Contains(e.Message, "valid") {
				continue // Cycles are valid - skip informational messages
			}
			blockingErrors = append(blockingErrors, e)
		}
		validationErrors = blockingErrors
	}
	if len(validationErrors) > 0 {
		// Enhance errors with suggestions and context
		enhancer := dx.NewErrorEnhancer(filePath, strings.Split(string(content), "\n"), program)
		enhancedErrors := make([]*dx.EnhancedError, 0, len(validationErrors))
		for _, err := range validationErrors {
			enhancedErrors = append(enhancedErrors, enhancer.Enhance(err))
		}

		_, _ = fmt.Fprint(stderr, dx.FormatErrors(enhancedErrors, dx.SupportsColor()))
		return 1
	} else {
		_, _ = fmt.Fprintln(stdout, dx.Success("Compilation successful."))
		return 0
	}
}

package main

import (
	"flag"
	"fmt"
	"os"
	"strings"

	"io"

	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
)

func main() {
	os.Exit(Run(os.Args, os.Stdout, os.Stderr))
}

func Run(args []string, stdout, stderr io.Writer) int {
	lintCmd := flag.NewFlagSet("lint", flag.ContinueOnError)
	lintCmd.SetOutput(stderr)
	fmtCmd := flag.NewFlagSet("fmt", flag.ContinueOnError)
	fmtCmd.SetOutput(stderr)
	exportCmd := flag.NewFlagSet("export", flag.ContinueOnError)
	exportCmd.SetOutput(stderr)

	if len(args) < 2 {
		_, _ = fmt.Fprintln(stderr, "expected 'compile', 'lint', 'fmt', 'export', 'explain', 'list', 'init', 'tree', 'diff', or 'version' subcommands")
		_, _ = fmt.Fprintln(stderr, "Run 'sruja <command> --help' for usage information")
		return 1
	}

	// Handle version flag globally
	if len(args) >= 2 && (args[1] == "--version" || args[1] == "-v" || args[1] == "version") {
		return runVersion(stdout)
	}

	switch args[1] {

	case "compile":
		compileCmd := flag.NewFlagSet("compile", flag.ContinueOnError)
		compileCmd.SetOutput(stderr)
		if err := compileCmd.Parse(args[2:]); err != nil {
			_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing compile flags: %v", err)))
			return 1
		}
		if compileCmd.NArg() < 1 {
			_, _ = fmt.Fprintln(stderr, "expected file path")
			return 1
		}
		return runCompile(compileCmd.Arg(0), stdout, stderr)

	case "lint":
		if err := lintCmd.Parse(args[2:]); err != nil {
			_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing lint flags: %v", err)))
			return 1
		}
		if lintCmd.NArg() < 1 {
			_, _ = fmt.Fprintln(stderr, "expected file path")
			return 1
		}
		return runLint(lintCmd.Arg(0), stdout, stderr)
	case "fmt":
		if err := fmtCmd.Parse(args[2:]); err != nil {
			_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing fmt flags: %v", err)))
			return 1
		}
		if fmtCmd.NArg() < 1 {
			_, _ = fmt.Fprintln(stderr, "expected file path")
			return 1
		}
		return runFmt(fmtCmd.Arg(0), stdout, stderr)

	case "export":
		if err := exportCmd.Parse(args[2:]); err != nil {
			_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing export flags: %v", err)))
			return 1
		}
		if exportCmd.NArg() < 2 {
			_, _ = fmt.Fprintln(stderr, "expected format and file path (e.g., 'sruja export d2 example.sruja')")
			return 1
		}
		return runExport(exportCmd.Arg(0), exportCmd.Arg(1), stdout, stderr)

	case "explain":
		return runExplain(stdout, stderr)
	case "list":
		return runList(stdout, stderr)

	case "tree":
		return runTree(stdout, stderr)
	case "diff":
		return runDiff(stdout, stderr)
	case "version", "--version", "-v":
		return runVersion(stdout)
	default:
		_, _ = fmt.Fprintln(stderr, "expected 'compile', 'lint', 'fmt', 'export', 'explain', 'list', 'init', 'tree', 'diff', or 'version' subcommands")
		_, _ = fmt.Fprintln(stderr, "Run 'sruja <command> --help' for usage information")
		return 1
	}
}

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

	validationErrors := validator.Validate(program)
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
	if format != "d2" {
		_, _ = fmt.Fprintf(stderr, "Unsupported export format: %s. Only 'd2' is currently supported.\n", format)
		return 1
	}

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

	exporter := d2.NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Export Error: %v\n", err)
		return 1
	}

	_, _ = fmt.Fprintln(stdout, output)
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

	validationErrors := validator.Validate(program)
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

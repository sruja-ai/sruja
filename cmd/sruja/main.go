package main

import (
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
)

func main() {

	lintCmd := flag.NewFlagSet("lint", flag.ExitOnError)
	fmtCmd := flag.NewFlagSet("fmt", flag.ExitOnError)
	exportCmd := flag.NewFlagSet("export", flag.ExitOnError)

	if len(os.Args) < 2 {
		fmt.Println("expected 'compile', 'lint', 'fmt', 'export', 'explain', 'list', 'init', 'tree', 'diff', or 'version' subcommands")
		fmt.Println("Run 'sruja <command> --help' for usage information")
		os.Exit(1)
	}

	// Handle version flag globally
	if len(os.Args) >= 2 && (os.Args[1] == "--version" || os.Args[1] == "-v" || os.Args[1] == "version") {
		runVersion()
	}

	switch os.Args[1] {

	case "compile":
		compileCmd := flag.NewFlagSet("compile", flag.ExitOnError)
		if err := compileCmd.Parse(os.Args[2:]); err != nil {
			fmt.Println(dx.Error(fmt.Sprintf("Error parsing compile flags: %v", err)))
			os.Exit(1)
		}
		if compileCmd.NArg() < 1 {
			fmt.Println("expected file path")
			os.Exit(1)
		}
		runCompile(compileCmd.Arg(0))

	case "lint":
		if err := lintCmd.Parse(os.Args[2:]); err != nil {
			fmt.Println(dx.Error(fmt.Sprintf("Error parsing lint flags: %v", err)))
			os.Exit(1)
		}
		if lintCmd.NArg() < 1 {
			fmt.Println("expected file path")
			os.Exit(1)
		}
		runLint(lintCmd.Arg(0))
	case "fmt":
		if err := fmtCmd.Parse(os.Args[2:]); err != nil {
			fmt.Println(dx.Error(fmt.Sprintf("Error parsing fmt flags: %v", err)))
			os.Exit(1)
		}
		if fmtCmd.NArg() < 1 {
			fmt.Println("expected file path")
			os.Exit(1)
		}
		runFmt(fmtCmd.Arg(0))

	case "export":
		if err := exportCmd.Parse(os.Args[2:]); err != nil {
			fmt.Println(dx.Error(fmt.Sprintf("Error parsing export flags: %v", err)))
			os.Exit(1)
		}
		if exportCmd.NArg() < 2 {
			fmt.Println("expected format and file path (e.g., 'sruja export d2 example.sruja')")
			os.Exit(1)
		}
		runExport(exportCmd.Arg(0), exportCmd.Arg(1))

	case "explain":
		runExplain()
	case "list":
		runList()

	case "tree":
		runTree()
	case "diff":
		runDiff()
	case "version", "--version", "-v":
		runVersion()
	default:
		fmt.Println("expected 'compile', 'lint', 'fmt', 'export', 'explain', 'list', 'init', 'tree', 'diff', or 'version' subcommands")
		fmt.Println("Run 'sruja <command> --help' for usage information")
		os.Exit(1)
	}
}

func runLint(filePath string) {
	content, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error reading file: %v", err)))
		os.Exit(1)
	}

	p, err := language.NewParser()
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error creating parser: %v", err)))
		os.Exit(1)
	}

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Parser Error: %v", err)))
		os.Exit(1)
	}

	// Validation
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	validator.RegisterRule(&engine.OrphanDetectionRule{})
	validator.RegisterRule(&engine.UniqueIDRule{})

	validationErrors := validator.Validate(program)
	if len(validationErrors) > 0 {
		// Enhance errors with suggestions and context
		enhancer := dx.NewErrorEnhancer(filePath, strings.Split(string(content), "\n"), program)
		enhancedErrors := make([]*dx.EnhancedError, 0, len(validationErrors))
		for _, err := range validationErrors {
			enhancedErrors = append(enhancedErrors, enhancer.Enhance(err))
		}

		fmt.Print(dx.FormatErrors(enhancedErrors, dx.SupportsColor()))
		os.Exit(1)
	} else {
		fmt.Println(dx.Success("No linting errors found."))
	}
}

func runFmt(filePath string) {
	content, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Printf("Error reading file: %v\n", err)
		os.Exit(1)
	}

	p, err := language.NewParser()
	if err != nil {
		fmt.Printf("Error creating parser: %v\n", err)
		os.Exit(1)
	}

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		fmt.Printf("Parser Error: %v\n", err)
		os.Exit(1)
	}

	printer := &language.Printer{}
	formatted := printer.Print(program)
	fmt.Print(formatted)
}

func runExport(format, filePath string) {
	if format != "d2" {
		fmt.Printf("Unsupported export format: %s. Only 'd2' is currently supported.\n", format)
		os.Exit(1)
	}

	content, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Printf("Error reading file: %v\n", err)
		os.Exit(1)
	}

	p, err := language.NewParser()
	if err != nil {
		fmt.Printf("Error creating parser: %v\n", err)
		os.Exit(1)
	}

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		fmt.Printf("Parser Error: %v\n", err)
		os.Exit(1)
	}

	exporter := d2.NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		fmt.Printf("Export Error: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(output)
}

func runCompile(filePath string) {
	content, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error reading file: %v", err)))
		os.Exit(1)
	}

	p, err := language.NewParser()
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error creating parser: %v", err)))
		os.Exit(1)
	}

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Parser Error: %v", err)))
		os.Exit(1)
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

		fmt.Print(dx.FormatErrors(enhancedErrors, dx.SupportsColor()))
		os.Exit(1)
	} else {
		fmt.Println(dx.Success("Compilation successful."))
	}
}

package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/mcp"
	"github.com/sruja-ai/sruja/pkg/notebook"
)

func main() {
	compileCmd := flag.NewFlagSet("compile", flag.ExitOnError)
	compileOutput := compileCmd.String("output", "", "write Mermaid output to file")
	compileJSON := compileCmd.String("json", "", "write compiled model JSON to file")
	compilePretty := compileCmd.Bool("pretty", false, "pretty-print JSON when writing")
	compileWatch := compileCmd.Bool("watch", false, "recompile on file changes")
	compileFormat := compileCmd.String("format", "auto", "diagram format: auto (default), d2, mermaid, or 'list' to see all")
	compileUseCase := compileCmd.String("use-case", "", "use case for auto-selection: presentation, documentation, version-control, export, github, markdown")
	compileInfo := compileCmd.Bool("info", false, "show format information and recommendations")
	compileExport := compileCmd.String("export", "", "export format: svg, png, pdf (requires D2 rendering libraries)")
	compileTheme := compileCmd.String("theme", "neutral-default", "D2 theme: neutral-default, gruvbox-dark, gruvbox-light, neon, terminal, violet, warm-neon")
	compileLayout := compileCmd.String("layout", "dagre", "D2 layout engine: dagre, elk")
	notebookCmd := flag.NewFlagSet("notebook", flag.ExitOnError)
	lintCmd := flag.NewFlagSet("lint", flag.ExitOnError)
	fmtCmd := flag.NewFlagSet("fmt", flag.ExitOnError)
	installCmd := flag.NewFlagSet("install", flag.ExitOnError)
	updateCmd := flag.NewFlagSet("update", flag.ExitOnError)
	exportCmd := flag.NewFlagSet("export", flag.ExitOnError)

	if len(os.Args) < 2 {
		fmt.Println("expected 'compile', 'notebook', 'lint', 'fmt', 'install', 'update', 'export', 'mcp', 'explain', 'list', 'init', 'tree', 'diff', or 'version' subcommands")
		fmt.Println("Run 'sruja <command> --help' for usage information")
		os.Exit(1)
	}

	// Handle version flag globally
	if len(os.Args) >= 2 && (os.Args[1] == "--version" || os.Args[1] == "-v" || os.Args[1] == "version") {
		runVersion()
	}

	switch os.Args[1] {
	case "compile":
		compileCmd.Parse(os.Args[2:])
		if compileCmd.NArg() < 1 {
			fmt.Println("expected file path")
			os.Exit(1)
		}
		compileFileWithOptions(compileCmd.Arg(0), *compileOutput, *compileJSON, *compilePretty, *compileWatch, *compileFormat, *compileUseCase, *compileInfo, *compileExport, *compileTheme, *compileLayout)
	case "notebook":
		notebookCmd.Parse(os.Args[2:])
		if notebookCmd.NArg() < 1 {
			fmt.Println("expected markdown file path")
			os.Exit(1)
		}
		runNotebook(notebookCmd.Arg(0))
	case "lint":
		lintCmd.Parse(os.Args[2:])
		if lintCmd.NArg() < 1 {
			fmt.Println("expected file path")
			os.Exit(1)
		}
		runLint(lintCmd.Arg(0))
	case "fmt":
		fmtCmd.Parse(os.Args[2:])
		if fmtCmd.NArg() < 1 {
			fmt.Println("expected file path")
			os.Exit(1)
		}
		runFmt(fmtCmd.Arg(0))
	case "install":
		installCmd.Parse(os.Args[2:])
		runInstall()
	case "update":
		updateCmd.Parse(os.Args[2:])
		runUpdate()
	case "export":
		exportCmd.Parse(os.Args[2:])
		if exportCmd.NArg() < 2 {
			fmt.Println("expected format and file path (e.g., 'sruja export d2 example.sruja')")
			os.Exit(1)
		}
		runExport(exportCmd.Arg(0), exportCmd.Arg(1))
	case "mcp":
		runMCP()
	case "explain":
		runExplain()
	case "list":
		runList()
	case "init":
		runInit()
	case "tree":
		runTree()
	case "diff":
		runDiff()
	case "version", "--version", "-v":
		runVersion()
	default:
		fmt.Println("expected 'compile', 'notebook', 'lint', 'fmt', 'install', 'update', 'export', 'mcp', 'explain', 'list', 'init', 'tree', 'diff', or 'version' subcommands")
		fmt.Println("Run 'sruja <command> --help' for usage information")
		os.Exit(1)
	}
}

func compileFileWithOptions(filePath, outputPath, jsonPath string, pretty bool, watch bool, format, useCase string, info bool, exportFormat, theme, layout string) {
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

	importPaths := []string{}

	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	validator.RegisterRule(&engine.OrphanDetectionRule{})
	validator.RegisterRule(&engine.ExternalBestPracticeRule{})

	errors := validator.Validate(program)
	if len(errors) > 0 {
		fmt.Println("Validation Errors:")
		for _, err := range errors {
			fmt.Println("\t" + err.String())
		}
		os.Exit(1)
	}

	compileOnce := func() {
		registry := compiler.NewRegistry()
		selector := compiler.NewSelector(registry)

		// Handle special commands
		if format == "list" {
			fmt.Println("Available formats:")
			for _, name := range registry.List() {
				info, err := selector.GetFormatInfo(name)
				if err == nil {
					fmt.Printf("  %s: %s\n", name, info.Description)
					fmt.Printf("    Use cases: %s\n", strings.Join(info.UseCases, ", "))
					fmt.Printf("    Capabilities: %s\n", strings.Join(info.Capabilities, ", "))
				} else {
					fmt.Printf("  %s\n", name)
				}
			}
			return
		}

		// Show format info if requested
		if info {
			rec, err := selector.Recommend(program, useCase)
			if err != nil {
				fmt.Printf("Error getting recommendation: %v\n", err)
				os.Exit(1)
			}
			fmt.Printf("Recommended format: %s (score: %.2f)\n", rec.Format, rec.Score)
			fmt.Println("Reasons:")
			for _, reason := range rec.Reasons {
				fmt.Printf("  - %s\n", reason)
			}
			if len(rec.Alternatives) > 0 {
				fmt.Printf("Alternatives: %s\n", strings.Join(rec.Alternatives, ", "))
			}
			fmt.Println()
		}

		// Determine format
		selectedFormat := format
		if selectedFormat == "auto" {
			selectedUseCase := useCase
			if selectedUseCase == "" {
				selectedUseCase = "general" // Default use case
			}
			rec, err := selector.Recommend(program, selectedUseCase)
			if err != nil {
				fmt.Printf("Error selecting format: %v\n", err)
				os.Exit(1)
			}
			selectedFormat = rec.Format
			if !info {
				fmt.Fprintf(os.Stderr, "Auto-selected format: %s\n", selectedFormat)
			}
		}

		// Compile
		var output string
		var err error
		output, err = registry.Compile(selectedFormat, program)
		if err != nil {
			fmt.Printf("Compilation error: %v\n", err)
			os.Exit(1)
		}

		// Handle export formats (SVG, PNG, PDF) for D2
		if exportFormat != "" && selectedFormat == "d2" {
			// Validate layout - reject TALA (commercial license required)
			if layout == "tala" {
				fmt.Fprintf(os.Stderr, "Warning: TALA layout requires a commercial license. Using 'dagre' instead.\n")
				layout = "dagre"
			}
			renderer := compiler.NewD2RendererWithOptions(compiler.RenderOptions{
				Theme:  theme,
				Layout: layout,
				Format: exportFormat,
			})

			// Determine output path
			exportPath := outputPath
			if exportPath == "" {
				// Generate output path from input file
				basePath := filePath
				if ext := filepath.Ext(basePath); ext != "" {
					basePath = basePath[:len(basePath)-len(ext)]
				}
				exportPath = basePath + "." + exportFormat
			}

			// Render and export
			err = renderer.RenderToFile(output, exportPath, exportFormat)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Export error: %v\n", err)
				fmt.Fprintf(os.Stderr, "Note: D2 rendering requires additional dependencies.\n")
				fmt.Fprintf(os.Stderr, "Install with: go get github.com/terrastruct/d2/d2lib github.com/terrastruct/d2/d2renderers/d2svg github.com/terrastruct/d2/d2renderers/d2png github.com/terrastruct/d2/d2themes/d2themescatalog\n")
				fmt.Fprintf(os.Stderr, "Build with: go build -tags d2render\n")
				os.Exit(1)
			}

			fmt.Fprintf(os.Stderr, "Exported to: %s\n", exportPath)
			// Also write D2 source if output path is specified
			if outputPath != "" && outputPath != exportPath {
				if err := os.WriteFile(outputPath, []byte(output), 0644); err != nil {
					fmt.Printf("Error writing D2 source file: %v\n", err)
					os.Exit(1)
				}
			}
		} else {
			// Write output (D2/Mermaid text)
			if outputPath != "" {
				if err := os.WriteFile(outputPath, []byte(output), 0644); err != nil {
					fmt.Printf("Error writing output file: %v\n", err)
					os.Exit(1)
				}
			} else {
				fmt.Println(output)
			}
		}

		// Write JSON if requested
		if jsonPath != "" {
			tr := compiler.NewTransformer()
			m, err := tr.Transform(program)
			if err != nil {
				fmt.Printf("Transformation error: %v\n", err)
				os.Exit(1)
			}
			var data []byte
			if pretty {
				data, err = m.ToJSON()
			} else {
				data, err = json.Marshal(m)
			}
			if err != nil {
				fmt.Printf("JSON serialization error: %v\n", err)
				os.Exit(1)
			}
			if err := os.WriteFile(jsonPath, data, 0644); err != nil {
				fmt.Printf("Error writing JSON file: %v\n", err)
				os.Exit(1)
			}
		}
	}

	if !watch {
		compileOnce()
		return
	}

	tracked := map[string]time.Time{}
	paths := append([]string{filePath}, importPaths...)
	for _, pth := range paths {
		if st, err := os.Stat(pth); err == nil {
			tracked[pth] = st.ModTime()
		}
	}
	compileOnce()
	for {
		changed := false
		for _, pth := range paths {
			if st, err := os.Stat(pth); err == nil {
				mt := st.ModTime()
				if prev, ok := tracked[pth]; !ok || mt.After(prev) {
					tracked[pth] = mt
					changed = true
				}
			}
		}
		if changed {
			// Re-read and re-parse main file and imports
			content, err := os.ReadFile(filePath)
			if err != nil {
				fmt.Printf("Error reading file: %v\n", err)
				continue
			}
			program, err = p.Parse(filePath, string(content))
			if err != nil {
				fmt.Printf("Parser Error: %v\n", err)
				continue
			}
			// Import re-merge skipped in updated architecture model
			compileOnce()
		}
		time.Sleep(500 * time.Millisecond)
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
	validator.RegisterRule(&engine.CycleDetectionRule{})
	validator.RegisterRule(&engine.OrphanDetectionRule{})
	validator.RegisterRule(&engine.ExternalBestPracticeRule{})

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

func runNotebook(filePath string) {
	runner := notebook.NewRunner()
	err := runner.ProcessMarkdown(filePath)
	if err != nil {
		fmt.Printf("Error processing notebook: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("Successfully processed notebook: %s\n", filePath)
}

func runMCP() {
	server := mcp.NewServer()
	server.Serve()
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

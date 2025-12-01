package main

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	"github.com/sruja-ai/sruja/pkg/export/svg"
	"github.com/sruja-ai/sruja/pkg/language"
)

// runExportFolder exports all .sruja files in a folder to SVG
// It handles both independent architectures and inter-linked ones via imports
// Each file is treated as its own architecture, and imports are resolved within the folder context
func runExportFolder(folderPath, outputDir string, stdout, stderr io.Writer) int {
	// Validate folder exists
	info, err := os.Stat(folderPath)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error: folder not found: %s\n", folderPath)
		return 1
	}
	if !info.IsDir() {
		_, _ = fmt.Fprintf(stderr, "Error: %s is not a directory\n", folderPath)
		return 1
	}

	// Create output directory if it doesn't exist
	if outputDir == "" {
		outputDir = filepath.Join(folderPath, "svg-output")
	}
	if err := os.MkdirAll(outputDir, 0755); err != nil {
		_, _ = fmt.Fprintf(stderr, "Error creating output directory: %v\n", err)
		return 1
	}

	// Find all .sruja files
	var srujaFiles []string
	err = filepath.Walk(folderPath, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && strings.HasSuffix(path, ".sruja") {
			srujaFiles = append(srujaFiles, path)
		}
		return nil
	})

	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error walking directory: %v\n", err)
		return 1
	}

	if len(srujaFiles) == 0 {
		_, _ = fmt.Fprintf(stderr, "No .sruja files found in %s\n", folderPath)
		return 1
	}

	parser, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Error creating parser: %v\n", err)
		return 1
	}

	exporter := svg.NewExporter()
	success := 0
	failed := 0

	// Cache for parsed files to avoid re-parsing when resolving imports
	// This helps with inter-linked architectures
	parsedCache := make(map[string]*language.Program)

	_, _ = fmt.Fprintf(stdout, "Found %d .sruja file(s). Generating SVGs...\n\n", len(srujaFiles))

	for _, filePath := range srujaFiles {
		// Read file
		content, err := os.ReadFile(filePath)
		if err != nil {
			_, _ = fmt.Fprintf(stderr, "Error reading %s: %v\n", filePath, err)
			failed++
			continue
		}

		// Parse with import resolution (using cache for inter-linked files)
		program, err := parseWithImportsCached(parser, filePath, string(content), "", parsedCache)
		if err != nil {
			_, _ = fmt.Fprintf(stderr, "Error parsing %s: %v\n", filePath, err)
			failed++
			continue
		}

		// Cache the parsed program for potential reuse
		parsedCache[filePath] = program

		// Generate SVG - each file gets its own SVG even if it imports others
		svgOutput, err := exporter.Export(program.Architecture)
		if err != nil {
			_, _ = fmt.Fprintf(stderr, "Error exporting %s: %v\n", filePath, err)
			failed++
			continue
		}

		// Determine output filename
		relPath, _ := filepath.Rel(folderPath, filePath)
		outputFile := filepath.Join(outputDir, strings.TrimSuffix(relPath, ".sruja")+".svg")

		// Create subdirectories if needed
		if err := os.MkdirAll(filepath.Dir(outputFile), 0755); err != nil {
			_, _ = fmt.Fprintf(stderr, "Error creating output directory: %v\n", err)
			failed++
			continue
		}

		// Write SVG
		if err := os.WriteFile(outputFile, []byte(svgOutput), 0644); err != nil {
			_, _ = fmt.Fprintf(stderr, "Error writing %s: %v\n", outputFile, err)
			failed++
			continue
		}

		size := len(svgOutput)
		importsInfo := ""
		if len(program.Architecture.Imports) > 0 {
			importsInfo = fmt.Sprintf(" (imports: %d)", len(program.Architecture.Imports))
		}
		_, _ = fmt.Fprintf(stdout, "✓ %s -> %s (%d bytes)%s\n", relPath, outputFile, size, importsInfo)
		success++
	}

	_, _ = fmt.Fprintf(stdout, "\nSummary: %d succeeded, %d failed\n", success, failed)
	_, _ = fmt.Fprintf(stdout, "SVG files saved to: %s\n", outputDir)
	_, _ = fmt.Fprintf(stdout, "\nNote: Each file is exported as an independent architecture.\n")
	_, _ = fmt.Fprintf(stdout, "      Imports are resolved and included in the SVG visualization.\n")

	if failed > 0 {
		return 1
	}
	return 0
}

// parseWithImportsCached parses a file and resolves imports with caching
// This handles both independent architectures and inter-linked ones
func parseWithImportsCached(parser *language.Parser, filePath, content string, _ string, cache map[string]*language.Program) (*language.Program, error) {
	// Check cache first (for inter-linked files)
	if cached, ok := cache[filePath]; ok {
		return cached, nil
	}

	// Parse the main file
	program, err := parser.Parse(filePath, content)
	if err != nil {
		return nil, err
	}

	// Resolve imports with caching
	visited := make(map[string]bool) // Track visited files to prevent circular import loops
	if err := resolveImportsCached(program.Architecture, filepath.Dir(filePath), "", parser, cache, visited); err != nil {
		return nil, fmt.Errorf("failed to resolve imports: %w", err)
	}

	return program, nil
}

// resolveImportsCached resolves import statements with caching and circular import detection
// This allows inter-linked architectures to reference each other
func resolveImportsCached(arch *language.Architecture, currentDir string, _ string, parser *language.Parser, cache map[string]*language.Program, visited map[string]bool) error {
	if arch == nil {
		return nil
	}

	// Resolve imports for this architecture
	for _, imp := range arch.Imports {
		// Resolve import path relative to current file's directory
		importPath := imp.Path
		if !filepath.IsAbs(importPath) {
			// Remove quotes if present
			importPath = strings.Trim(importPath, `"`)
			// Resolve relative to current file's directory
			importPath = filepath.Join(currentDir, importPath)
		}

		// Normalize path for comparison
		absPath, err := filepath.Abs(importPath)
		if err != nil {
			return fmt.Errorf("cannot resolve import path %s: %w", imp.Path, err)
		}

		// Check for circular imports
		if visited[absPath] {
			// Circular import detected - this is okay, just skip to avoid infinite loop
			// The imported architecture is already being processed
			continue
		}

		// Check cache first
		var importedProgram *language.Program
		if cached, ok := cache[absPath]; ok {
			importedProgram = cached
		} else {
			// Read imported file
			content, err := os.ReadFile(importPath)
			if err != nil {
				return fmt.Errorf("cannot resolve import %s: %w", imp.Path, err)
			}

			// Parse imported file
			importedProgram, err = parser.Parse(importPath, string(content))
			if err != nil {
				return fmt.Errorf("error parsing imported file %s: %w", imp.Path, err)
			}

			// Mark as visited and recursively resolve its imports
			visited[absPath] = true
			if err := resolveImportsCached(importedProgram.Architecture, filepath.Dir(importPath), "", parser, cache, visited); err != nil {
				return err
			}
			visited[absPath] = false // Unmark after processing

			// Cache the parsed program
			cache[absPath] = importedProgram
		}

		// Add to ResolvedImports
		alias := imp.Path
		if imp.Alias != nil {
			alias = *imp.Alias
		}
		arch.ResolvedImports = append(arch.ResolvedImports, &language.ImportedArchitecture{
			Alias:        alias,
			Architecture: importedProgram.Architecture,
		})
	}

	return nil
}

package language

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/stdlib"
)

// ParseWorkspace recursively finds and parses all .sruja files in the given directory.
func (p *Parser) ParseWorkspace(rootPath string) (*Workspace, error) {
	ws := NewWorkspace()

	err := filepath.Walk(rootPath, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && strings.HasPrefix(filepath.Base(path), ".") {
			return nil
		}
		if !info.IsDir() && filepath.Ext(path) == ".sruja" {
			content, err := os.ReadFile(path)
			if err != nil {
				return fmt.Errorf("failed to read %s: %w", path, err)
			}

			prog, diags, err := p.Parse(path, string(content))
			if err != nil {
				// We still add it to workspace to collect diags, but the error might be critical
				ws.AddProgram(path, prog, diags)
				return nil // Continue walking to find other files
			}
			ws.AddProgram(path, prog, diags)

			// Recursively resolve imports found in this file
			_ = p.resolveImports(ws, prog, path)
		}
		return nil
	})

	if err != nil {
		return nil, fmt.Errorf("failed to walk workspace: %w", err)
	}

	return ws, nil
}

// ResolveImports recursively loads files referenced by import statements.
func (p *Parser) ResolveImports(ws *Workspace, prog *Program, filename string) error {
	return p.resolveImports(ws, prog, filename)
}

// resolveImports recursively loads files referenced by import statements.
func (p *Parser) resolveImports(ws *Workspace, prog *Program, filename string) error {
	if prog == nil || prog.Model == nil {
		return nil
	}

	for _, item := range prog.Model.Items {
		if item.Import != nil {
			importPath := item.Import.From

			// Resolve alias or relative path
			resolvedPath := importPath
			if alias, ok := ws.Aliases[importPath]; ok {
				resolvedPath = alias
			} else if strings.HasPrefix(importPath, ".") {
				resolvedPath = filepath.Join(filepath.Dir(filename), importPath)
			}

			// Special case: Standard Library (can be embedded or local)
			if ws.IsStdLib(importPath) {
				// Try loading from embedded FS first (works in WASM)
				p.loadFromStdLibFS(ws, importPath)
				// Continue to next import if loaded successfully
				// (function is best-effort and always succeeds)
				continue
			}

			// Capture info to decide how to load from OS
			info, err := os.Stat(resolvedPath)
			if err != nil {
				continue // Skip invalid paths for now
			}

			if info.IsDir() {
				// Load all files in directory
				subWS, err := p.ParseWorkspace(resolvedPath)
				if err == nil {
					for f, p := range subWS.Programs {
						ws.AddProgram(f, p, nil) // Diagnostics already in subWS
					}
					ws.Diags = append(ws.Diags, subWS.Diags...)
				}
			} else if filepath.Ext(resolvedPath) == ".sruja" {
				// Load single file if not already loaded
				if _, ok := ws.Programs[resolvedPath]; !ok {
					fileContent, err := os.ReadFile(resolvedPath)
					if err == nil {
						newProg, diags, err := p.Parse(resolvedPath, string(fileContent))
						if err == nil {
							ws.AddProgram(resolvedPath, newProg, diags)
							// Recursively resolve imports of the new program
							_ = p.resolveImports(ws, newProg, resolvedPath)
						}
					}
				}
			}
		}
	}
	return nil
}

// loadFromStdLibFS loads the standard library from the embedded filesystem.
// This is a best-effort operation that silently continues on errors.
func (p *Parser) loadFromStdLibFS(ws *Workspace, _ string) {
	// stdlib files are core.sruja, styles.sruja
	files := []string{"core.sruja", "styles.sruja"}
	for _, f := range files {
		content, err := stdlib.FS.ReadFile(f)
		if err != nil {
			continue
		}
		path := "sruja.ai/stdlib/" + f
		if _, ok := ws.Programs[path]; !ok {
			prog, diags, err := p.Parse(path, string(content))
			if err == nil {
				ws.AddProgram(path, prog, diags)
			}
		}
	}
}

// ParseFile parses a DSL file into an AST.
//
// This reads the file from disk and parses it. Use this for parsing files,
// or use Parse() if you already have the text in memory.
//
// Parameters:
//   - filename: Path to the .sruja file
//
// Returns:
//   - *Program: The parsed program (root of AST)
//   - []diagnostics.Diagnostic: List of diagnostics
//   - error: Critical error if file cannot be read
//
// Example:
//
//	parser, _ := language.NewParser()
//	program, diags, err := parser.ParseFile("example.sruja")
//	if err != nil {
//	    log.Fatal(err)
//	}
func (p *Parser) ParseFile(filename string) (*Program, []diagnostics.Diagnostic, error) {
	data, err := os.ReadFile(filename) // #nosec G304 // user defined path
	if err != nil {
		return nil, nil, fmt.Errorf("read error: %w", err)
	}
	return p.Parse(filename, string(data))
}

func baseName(filename string) string {
	// Extract base name without directories; if empty, use "Untitled"
	idx := strings.LastIndex(filename, "/")
	name := filename
	if idx != -1 && idx+1 < len(filename) {
		name = filename[idx+1:]
	}
	if name == "" {
		name = "Untitled"
	}
	// strip extension
	if dot := strings.LastIndex(name, "."); dot > 0 {
		name = name[:dot]
	}
	return name
}

package main

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
	"github.com/sruja-ai/sruja/pkg/export/html"
	"github.com/sruja-ai/sruja/pkg/language"
)

// htmlExamplesCmd exports all .sruja files in the examples directory to HTML
var htmlExamplesCmd = &cobra.Command{
	Use:   "html-examples",
	Short: "Export all examples to self-contained HTML",
	Long:  "Generate self-contained HTML for all .sruja files under the examples directory (uses SVG mode by default).",
	RunE: func(cmd *cobra.Command, _ []string) error {
		outDir, _ := cmd.Flags().GetString("out-dir")
		mode, _ := cmd.Flags().GetString("mode")

		// Ensure output directory exists
		if err := os.MkdirAll(outDir, 0o755); err != nil {
			return fmt.Errorf("failed to create output directory: %w", err)
		}

		// Collect example files recursively under ./examples
		exampleRoot := "examples"
		var files []string
		if err := filepath.WalkDir(exampleRoot, func(path string, d os.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if d.IsDir() {
				return nil
			}
			if filepath.Ext(path) == ".sruja" {
				files = append(files, path)
			}
			return nil
		}); err != nil {
			return fmt.Errorf("failed to scan examples: %w", err)
		}

		if len(files) == 0 {
			fmt.Fprintf(cmd.OutOrStdout(), "No .sruja examples found in %s\n", exampleRoot)
			return nil
		}

		// Prepare parser and exporter
		parser, err := language.NewParser()
		if err != nil {
			return fmt.Errorf("failed to create parser: %w", err)
		}

		exporter := html.NewExporter()
		// Set mode based on flag
		switch mode {
		case "single":
			exporter.Mode = html.ModeSingleFile
		case "svg":
			exporter.Mode = html.ModeSVG
		default:
			exporter.Mode = html.ModeSVG // Default to SVG
		}

		// Export each file
		var success, failed int
		for _, f := range files {
			content, err := os.ReadFile(f)
			if err != nil {
				fmt.Fprintf(cmd.ErrOrStderr(), "[skip] read error %s: %v\n", f, err)
				failed++
				continue
			}
			program, _, err := parser.Parse(f, string(content))
			if err != nil {
				fmt.Fprintf(cmd.ErrOrStderr(), "[skip] parse error %s: %v\n", f, err)
				failed++
				continue
			}

			base := filepath.Base(f)
			name := base[:len(base)-len(filepath.Ext(base))]
			outPath := filepath.Join(outDir, name+".html")

			if err := exporter.Export(program.Architecture, outPath); err != nil {
				fmt.Fprintf(cmd.ErrOrStderr(), "[fail] export %s: %v\n", f, err)
				failed++
				continue
			}
			fmt.Fprintf(cmd.OutOrStdout(), "[ok] %s -> %s\n", f, outPath)
			success++
		}

		fmt.Fprintf(cmd.OutOrStdout(), "Done. success=%d failed=%d\n", success, failed)
		return nil
	},
}

func init() {
	rootCmd.AddCommand(htmlExamplesCmd)
	htmlExamplesCmd.Flags().String("out-dir", "examples-html", "Output directory for generated HTML files")
	htmlExamplesCmd.Flags().String("mode", "svg", "Export mode: svg (default, lightweight) or single (full viewer bundle)")
}

package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/spf13/cobra"
	"github.com/sruja-ai/sruja/pkg/export/html"
	"github.com/sruja-ai/sruja/pkg/language"
)

var exportHTMLCmd = &cobra.Command{
	Use:   "html [file]",
	Short: "Export architecture to HTML",
	Long:  `Export the architecture definition to a self-contained HTML file.`,
	Args:  cobra.MaximumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		// Determine input file
		var inputFile string
		if len(args) > 0 {
			inputFile = args[0]
		} else {
			// Try to find a .sruja file in current directory
			files, err := filepath.Glob("*.sruja")
			if err != nil {
				return err
			}
			if len(files) == 0 {
				return fmt.Errorf("no .sruja file found in current directory")
			}
			inputFile = files[0]
			if len(files) > 1 {
				fmt.Printf("Multiple .sruja files found, using %s\n", inputFile)
			}
		}

		// Parse input file
		content, err := os.ReadFile(inputFile) // #nosec G304 // user defined input file
		if err != nil {
			return fmt.Errorf("failed to read file %s: %w", inputFile, err)
		}

		// Parse DSL
		parser, err := language.NewParser()
		if err != nil {
			return fmt.Errorf("failed to create parser: %w", err)
		}
		program, _, err := parser.Parse(inputFile, string(content))
		if err != nil {
			return fmt.Errorf("failed to parse DSL: %w", err)
		}

		// Determine output file
		outputFile, _ := cmd.Flags().GetString("output")
		if outputFile == "" {
			base := filepath.Base(inputFile)
			ext := filepath.Ext(base)
			outputFile = strings.TrimSuffix(base, ext) + ".html"
		}

		// Configure exporter
		exporter := html.NewExporter()
		mode, _ := cmd.Flags().GetString("mode")
		switch mode {
		case "svg":
			exporter.Mode = html.ModeSVG
		case "single":
			exporter.Mode = html.ModeSingleFile
		case "v2":
			exporter.Mode = html.ModeV2
		default:
			// Default to SVG mode (lightweight)
			exporter.Mode = html.ModeSVG
			mode = "svg"
		}
		minify, _ := cmd.Flags().GetBool("minify")
		exporter.Minify = minify

		// Export
		fmt.Printf("Exporting %s to %s (mode: %s)...\n", inputFile, outputFile, mode)
		if err := exporter.Export(program.Architecture, outputFile); err != nil {
			return fmt.Errorf("export failed: %w", err)
		}

		fmt.Println("Export successful!")
		return nil
	},
}

func init() {
	cmdExport.AddCommand(exportHTMLCmd)

	exportHTMLCmd.Flags().StringP("output", "o", "", "Output HTML file path")
	exportHTMLCmd.Flags().String("mode", "v2", "Export mode: 'legacy' (SVG-based) or 'v2' (Cytoscape-based)")

	// Add completion for flags
	_ = exportHTMLCmd.RegisterFlagCompletionFunc("mode", func(_ *cobra.Command, _ []string, _ string) ([]string, cobra.ShellCompDirective) {
		return []string{"legacy", "v2"}, cobra.ShellCompDirectiveNoFileComp
	})
}

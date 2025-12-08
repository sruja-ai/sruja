package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/export/html"
	jexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/export/svg"
	"github.com/sruja-ai/sruja/pkg/language"
)

//nolint:funlen,gocyclo,goconst // Export logic is complex, distinct strings needed
func runExport(args []string, stdout, stderr io.Writer) int {
	exportCmd := flag.NewFlagSet("export", flag.ContinueOnError)
	exportCmd.SetOutput(stderr)

	// Define flags
	singleFile := exportCmd.Bool("single-file", false, "Generate single file (full viewer bundle)")
	outDir := exportCmd.String("out", "", "Output directory")
	view := exportCmd.String("view", "system", "Diagram view for svg/pdf: system | container:<systemId> | component:<systemId>/<containerId> | c4:l1 | c4:l2:<systemId> | c4:l3:<systemId>/<containerId> | deployment[:<deploymentId>] ")
	legend := exportCmd.Bool("legend", false, "Show legend in SVG outputs")
	extended := exportCmd.Bool("extended", false, "Include pre-computed views in JSON output (for viewer apps)")
	_ = exportCmd.Bool("local", false, "Use local assets")

	if err := exportCmd.Parse(args); err != nil {
		_, _ = fmt.Fprintf(stderr, "Error parsing export flags: %v\n", err)
		return 1
	}

	if exportCmd.NArg() < 2 {
		_, _ = fmt.Fprintln(stderr, "Usage: sruja export <format> <file>")
		return 1
	}

	format := exportCmd.Arg(0)
	filePath := exportCmd.Arg(1)

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

	program, _, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Parser Error: %v\n", err)
		return 1
	}

	// Resolve references (smart resolution for short names)
	engine.RunResolution(program)

	var output string
	switch format {
	case "json":
		exporter := jexport.NewExporter()
		exporter.Extended = *extended
		output, err = exporter.Export(program.Architecture)
	case "markdown":
		exporter := markdown.NewExporter()
		output, err = exporter.Export(program.Architecture)
	case "html":
		exporter := html.NewExporter()
		exporter.EmbedJSON = true // Embed JSON for self-contained HTML

		// Set export mode based on flags
		if *singleFile {
			exporter.Mode = html.ModeSingleFile
		} else {
			// Default to SVG mode (lightweight)
			exporter.Mode = html.ModeSVG
		}

		// Set output directory if provided
		if *outDir != "" {
			exporter.OutputDir = *outDir
		}

		output, err = exporter.ExportFromArchitecture(program.Architecture)
	case "svg":
		exporter := svg.NewExporter()
		exporter.ShowLegend = *legend
		// Support multiple views
		// Note: If views block is defined in DSL, it can be used for filtering/styling
		// For now, CLI view flags take precedence, but DSL views provide styling hints
		//nolint:gocritic // if-else chain preferred for clarity here
		if strings.HasPrefix(*view, "container:") || strings.HasPrefix(*view, "c4:l2:") {
			parts := strings.Split(strings.TrimPrefix(*view, "container:"), "/")
			if strings.HasPrefix(*view, "c4:l2:") {
				parts = strings.Split(strings.TrimPrefix(*view, "c4:l2:"), "/")
			}
			sysID := parts[0]
			var sys *language.System
			for _, s := range program.Architecture.Systems {
				if s.ID == sysID {
					sys = s
					break
				}
			}
			if sys == nil {
				_, _ = fmt.Fprintf(stderr, "System not found: %s\n", sysID)
				return 1
			}
			output, err = exporter.ExportSystemContainer(program.Architecture, sys)
		} else if strings.HasPrefix(*view, "component:") || strings.HasPrefix(*view, "c4:l3:") {
			parts := strings.Split(strings.TrimPrefix(*view, "component:"), "/")
			if strings.HasPrefix(*view, "c4:l3:") {
				parts = strings.Split(strings.TrimPrefix(*view, "c4:l3:"), "/")
			}
			if len(parts) < 2 {
				_, _ = fmt.Fprintln(stderr, "Invalid component view. Use component:<systemId>/<containerId>")
				return 1
			}
			sysID := parts[0]
			contID := parts[1]
			var sys *language.System
			var cont *language.Container
			for _, s := range program.Architecture.Systems {
				if s.ID == sysID {
					sys = s
					break
				}
			}
			if sys != nil {
				for _, c := range sys.Containers {
					if c.ID == contID {
						cont = c
						break
					}
				}
			}
			if sys == nil || cont == nil {
				_, _ = fmt.Fprintf(stderr, "Component view not found: system=%s container=%s\n", sysID, contID)
				return 1
			}
			output, err = exporter.ExportContainerComponent(program.Architecture, sys, cont)
		} else if strings.HasPrefix(*view, "c4:l1") || strings.EqualFold(*view, "system") {
			output, err = exporter.Export(program.Architecture)
		} else if strings.HasPrefix(*view, "deployment") {
			// deployment[:<deploymentId>]
			depID := strings.TrimPrefix(*view, "deployment")
			depID = strings.TrimPrefix(depID, ":")
			var dep *language.DeploymentNode
			if depID == "" {
				if len(program.Architecture.DeploymentNodes) > 0 {
					dep = program.Architecture.DeploymentNodes[0]
				}
			} else {
				for _, d := range program.Architecture.DeploymentNodes {
					if d.ID == depID {
						dep = d
						break
					}
				}
			}
			if dep == nil {
				_, _ = fmt.Fprintln(stderr, "Deployment node not found")
				return 1
			}
			output, err = exporter.ExportDeployment(program.Architecture, dep)
		} else if strings.HasPrefix(*view, "all") {
			output, err = exporter.ExportAll(program.Architecture)
		} else {
			output, err = exporter.Export(program.Architecture)
		}
	default:
		_, _ = fmt.Fprintf(stderr, "Unsupported export format: %s. Supported formats: json, markdown, html, svg\n", format)
		return 1
	}

	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Export Error: %v\n", err)
		return 1
	}

	_, _ = fmt.Fprint(stdout, output)
	return 0
}

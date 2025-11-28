// apps/cli/cmd/diff.go
package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/language"
)

func runDiff(stdout, stderr io.Writer) int {
	diffCmd := flag.NewFlagSet("diff", flag.ContinueOnError)
	diffCmd.SetOutput(stderr)
	diffJSON := diffCmd.Bool("json", false, "output as JSON")
	diffFormat := diffCmd.String("format", "text", "output format: text, json")

	if err := diffCmd.Parse(os.Args[2:]); err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing diff flags: %v", err)))
		return 1
	}

	if diffCmd.NArg() < 2 {
		_, _ = fmt.Fprintln(stderr, dx.Error("Usage: sruja diff <file1> <file2>"))
		_, _ = fmt.Fprintln(stderr, dx.Info("Example: sruja diff architecture-v1.sruja architecture-v2.sruja"))
		return 1
	}

	file1 := diffCmd.Arg(0)
	file2 := diffCmd.Arg(1)

	// Parse both files
	program1, err := parseFile(file1)
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing %s: %v", file1, err)))
		return 1
	}

	program2, err := parseFile(file2)
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing %s: %v", file2, err)))
		return 1
	}

	// Generate diff
	diff := computeDiff(program1, program2, file1, file2)

	// Output diff
	if *diffJSON || *diffFormat == "json" {
		outputDiffJSON(diff, stdout)
	} else {
		outputDiffText(diff, stdout)
	}
	return 0
}

type Diff struct {
	AddedSystems      []string
	RemovedSystems    []string
	AddedContainers   map[string][]string // system -> containers
	RemovedContainers map[string][]string
	AddedComponents   map[string][]string // system.container -> components
	RemovedComponents map[string][]string
	ModifiedSystems   []string
}

func parseFile(filePath string) (*language.Program, error) {
	content, err := os.ReadFile(filePath)
	if err != nil {
		return nil, err
	}

	p, err := language.NewParser()
	if err != nil {
		return nil, err
	}

	return p.Parse(filePath, string(content))
}

func computeDiff(program1, program2 *language.Program, file1, file2 string) *Diff {
	diff := &Diff{
		AddedContainers:   make(map[string][]string),
		RemovedContainers: make(map[string][]string),
		AddedComponents:   make(map[string][]string),
		RemovedComponents: make(map[string][]string),
	}

	arch1 := program1.Architecture
	arch2 := program2.Architecture

	if arch1 == nil || arch2 == nil {
		return diff
	}

	// Build element maps for comparison
	systems1 := make(map[string]*language.System)
	systems2 := make(map[string]*language.System)

	for _, sys := range arch1.Systems {
		systems1[sys.ID] = sys
	}
	for _, sys := range arch2.Systems {
		systems2[sys.ID] = sys
	}

	// Find added and removed systems
	for id := range systems2 {
		if _, exists := systems1[id]; !exists {
			diff.AddedSystems = append(diff.AddedSystems, id)
		}
	}
	for id := range systems1 {
		if _, exists := systems2[id]; !exists {
			diff.RemovedSystems = append(diff.RemovedSystems, id)
		}
	}

	// Find modified systems (existing in both but changed)
	for id, sys2 := range systems2 {
		if sys1, exists := systems1[id]; exists {
			// Compare containers
			containers1 := make(map[string]bool)
			containers2 := make(map[string]bool)

			for _, cont := range sys1.Containers {
				containers1[cont.ID] = true
			}
			for _, cont := range sys2.Containers {
				containers2[cont.ID] = true
			}

			// Added containers
			for contID := range containers2 {
				if !containers1[contID] {
					diff.AddedContainers[id] = append(diff.AddedContainers[id], contID)
					diff.ModifiedSystems = appendIfNotExists(diff.ModifiedSystems, id)
				}
			}

			// Removed containers
			for contID := range containers1 {
				if !containers2[contID] {
					diff.RemovedContainers[id] = append(diff.RemovedContainers[id], contID)
					diff.ModifiedSystems = appendIfNotExists(diff.ModifiedSystems, id)
				}
			}

			// Compare components (simplified - check if container has new/removed components)
			for _, cont2 := range sys2.Containers {
				var cont1 *language.Container
				for _, c := range sys1.Containers {
					if c.ID == cont2.ID {
						cont1 = c
						break
					}
				}
				if cont1 != nil {
					comps1 := make(map[string]bool)
					comps2 := make(map[string]bool)

					for _, comp := range cont1.Components {
						comps1[comp.ID] = true
					}
					for _, comp := range cont2.Components {
						comps2[comp.ID] = true
					}

					for compID := range comps2 {
						if !comps1[compID] {
							key := fmt.Sprintf("%s.%s", id, cont2.ID)
							diff.AddedComponents[key] = append(diff.AddedComponents[key], compID)
							diff.ModifiedSystems = appendIfNotExists(diff.ModifiedSystems, id)
						}
					}
					for compID := range comps1 {
						if !comps2[compID] {
							key := fmt.Sprintf("%s.%s", id, cont1.ID)
							diff.RemovedComponents[key] = append(diff.RemovedComponents[key], compID)
							diff.ModifiedSystems = appendIfNotExists(diff.ModifiedSystems, id)
						}
					}
				}
			}
		}
	}

	return diff
}

func appendIfNotExists(slice []string, item string) []string {
	for _, s := range slice {
		if s == item {
			return slice
		}
	}
	return append(slice, item)
}

func outputDiffText(diff *Diff, w io.Writer) {
	_, _ = fmt.Fprintln(w, dx.Header("Architecture Diff"))

	if len(diff.AddedSystems) == 0 && len(diff.RemovedSystems) == 0 &&
		len(diff.AddedContainers) == 0 && len(diff.RemovedContainers) == 0 &&
		len(diff.AddedComponents) == 0 && len(diff.RemovedComponents) == 0 {
		_, _ = fmt.Fprintln(w, dx.Success("No differences found. Architectures are identical."))
		return
	}

	useColor := dx.SupportsColor()

	// Added systems
	if len(diff.AddedSystems) > 0 {
		_, _ = fmt.Fprintln(w, dx.Section("Added Systems"))
		for _, sys := range diff.AddedSystems {
			_, _ = fmt.Fprintf(w, "  %s %s\n", dx.Colorize(dx.ColorGreen, "+", useColor), dx.Colorize(dx.ColorBold, sys, useColor))
		}
		_, _ = fmt.Fprintln(w)
	}

	// Removed systems
	if len(diff.RemovedSystems) > 0 {
		_, _ = fmt.Fprintln(w, dx.Section("Removed Systems"))
		for _, sys := range diff.RemovedSystems {
			_, _ = fmt.Fprintf(w, "  %s %s\n", dx.Colorize(dx.ColorRed, "-", useColor), dx.Colorize(dx.ColorBold, sys, useColor))
		}
		_, _ = fmt.Fprintln(w)
	}

	// Modified systems
	if len(diff.ModifiedSystems) > 0 {
		_, _ = fmt.Fprintln(w, dx.Section("Modified Systems"))
		for _, sys := range diff.ModifiedSystems {
			_, _ = fmt.Fprintf(w, "  %s %s\n", dx.Colorize(dx.ColorYellow, "~", useColor), dx.Colorize(dx.ColorBold, sys, useColor))

			// Added containers
			if containers, ok := diff.AddedContainers[sys]; ok && len(containers) > 0 {
				for _, cont := range containers {
					_, _ = fmt.Fprintf(w, "    %s container %s\n", dx.Colorize(dx.ColorGreen, "+", useColor), cont)
				}
			}

			// Removed containers
			if containers, ok := diff.RemovedContainers[sys]; ok && len(containers) > 0 {
				for _, cont := range containers {
					_, _ = fmt.Fprintf(w, "    %s container %s\n", dx.Colorize(dx.ColorRed, "-", useColor), cont)
				}
			}

			// Added components
			for key, components := range diff.AddedComponents {
				if strings.HasPrefix(key, sys+".") {
					cont := strings.TrimPrefix(key, sys+".")
					for _, comp := range components {
						_, _ = fmt.Fprintf(w, "    %s component %s.%s\n", dx.Colorize(dx.ColorGreen, "+", useColor), cont, comp)
					}
				}
			}

			// Removed components
			for key, components := range diff.RemovedComponents {
				if strings.HasPrefix(key, sys+".") {
					cont := strings.TrimPrefix(key, sys+".")
					for _, comp := range components {
						_, _ = fmt.Fprintf(w, "    %s component %s.%s\n", dx.Colorize(dx.ColorRed, "-", useColor), cont, comp)
					}
				}
			}
		}
		_, _ = fmt.Fprintln(w)
	}
}

func outputDiffJSON(diff *Diff, w io.Writer) {
	_, _ = fmt.Fprintln(w, "{")
	_, _ = fmt.Fprintln(w, "  \"added_systems\": [")
	for i, sys := range diff.AddedSystems {
		_, _ = fmt.Fprintf(w, "    \"%s\"", sys)
		if i < len(diff.AddedSystems)-1 {
			_, _ = fmt.Fprint(w, ",")
		}
		_, _ = fmt.Fprintln(w)
	}
	_, _ = fmt.Fprintln(w, "  ],")
	_, _ = fmt.Fprintln(w, "  \"removed_systems\": [")
	for i, sys := range diff.RemovedSystems {
		_, _ = fmt.Fprintf(w, "    \"%s\"", sys)
		if i < len(diff.RemovedSystems)-1 {
			_, _ = fmt.Fprint(w, ",")
		}
		_, _ = fmt.Fprintln(w)
	}
	_, _ = fmt.Fprintln(w, "  ],")
	_, _ = fmt.Fprintln(w, "  \"modified_systems\": [")
	for i, sys := range diff.ModifiedSystems {
		_, _ = fmt.Fprintf(w, "    \"%s\"", sys)
		if i < len(diff.ModifiedSystems)-1 {
			_, _ = fmt.Fprint(w, ",")
		}
		_, _ = fmt.Fprintln(w)
	}
	_, _ = fmt.Fprintln(w, "  ]")
	_, _ = fmt.Fprintln(w, "}")
}

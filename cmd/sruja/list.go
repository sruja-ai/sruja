// apps/cli/cmd/list.go
package main

import (
	"flag"
	"fmt"
	"io"
	"strings"

	"github.com/sruja-ai/sruja/internal/lister"
	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/language"
)

func runList(args []string, stdout, stderr io.Writer) int {
	listCmd := flag.NewFlagSet("list", flag.ContinueOnError)
	listCmd.SetOutput(stderr)
	listJSON := listCmd.Bool("json", false, "output as JSON")
	listFile := listCmd.String("file", "", "architecture file path")

	if err := listCmd.Parse(args); err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing list flags: %v", err)))
		return 1
	}

	if listCmd.NArg() < 1 {
		_, _ = fmt.Fprintln(stderr, "Usage: sruja list <type> [--file <path>] [--json]")
		_, _ = fmt.Fprintln(stderr, "Types: systems, containers, components, persons, datastores, queues, scenarios, adrs")
		_, _ = fmt.Fprintln(stderr, "Example: sruja list systems --file architecture.sruja")
		return 1
	}

	elementType := strings.ToLower(listCmd.Arg(0))
	filePath := findSrujaFile(*listFile)

	if filePath == "" {
		_, _ = fmt.Fprintln(stderr, "Error: no architecture file found. Use --file to specify.")
		return 1
	}

	program, err := parseArchitectureFile(filePath, stderr)
	if err != nil {
		return 1
	}

	if program.Architecture == nil {
		_, _ = fmt.Fprintln(stderr, "Error: no architecture found in file")
		return 1
	}

	arch := program.Architecture

	if err := listElementType(arch, elementType, *listJSON, stdout, stderr); err != nil {
		return 1
	}
	return 0
}

func listElementType(arch *language.Architecture, elementType string, jsonOutput bool, stdout, stderr io.Writer) error {
	switch elementType {
	case "systems":
		listSystems(arch, jsonOutput, stdout)
	case "containers":
		listContainers(arch, jsonOutput, stdout)
	case "components":
		listComponents(arch, jsonOutput, stdout)
	case "persons":
		listPersons(arch, jsonOutput, stdout)
	case "datastores":
		listDataStores(arch, jsonOutput, stdout)
	case "queues":
		listQueues(arch, jsonOutput, stdout)
	case "scenarios":
		listScenarios(arch, jsonOutput, stdout)
	case "adrs":
		listADRs(arch, jsonOutput, stdout)

	default:
		_, _ = fmt.Fprintf(stderr, "Error: unknown type '%s'\n", elementType)
		_, _ = fmt.Fprintln(stderr, "Types: systems, containers, components, persons, datastores, queues, scenarios, adrs")
		return fmt.Errorf("unknown type: %s", elementType)
	}
	return nil
}

func listSystems(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	items := lister.ListSystems(arch)

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, sys := range items {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s"}`, sys.ID, sys.Label)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Systems (%d):\n\n", len(items))
		for _, sys := range items {
			desc := ""
			if sys.Description != "" {
				desc = fmt.Sprintf(" - %s", sys.Description)
			}
			_, _ = fmt.Fprintf(w, "  • %s: %s%s\n", sys.ID, sys.Label, desc)
		}
	}
}

func listContainers(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	items := lister.ListContainers(arch)

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, cont := range items {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s", "system": "%s"}`, cont.ID, cont.Label, cont.SystemID)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Containers (%d):\n\n", len(items))
		for _, cont := range items {
			desc := ""
			if cont.Description != "" {
				desc = fmt.Sprintf(" - %s", cont.Description)
			}
			_, _ = fmt.Fprintf(w, "  • %s.%s: %s%s\n", cont.SystemID, cont.ID, cont.Label, desc)
		}
	}
}

func listComponents(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	items := lister.ListComponents(arch)

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, comp := range items {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			if comp.ContainerID != "" {
				_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s", "system": "%s", "container": "%s"}`, comp.ID, comp.Label, comp.SystemID, comp.ContainerID)
			} else {
				_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s", "system": "%s"}`, comp.ID, comp.Label, comp.SystemID)
			}
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Components (%d):\n\n", len(items))
		for _, comp := range items {
			desc := ""
			if comp.Description != "" {
				desc = fmt.Sprintf(" - %s", comp.Description)
			}
			if comp.ContainerID != "" {
				_, _ = fmt.Fprintf(w, "  • %s.%s.%s: %s%s\n", comp.SystemID, comp.ContainerID, comp.ID, comp.Label, desc)
			} else {
				_, _ = fmt.Fprintf(w, "  • %s.%s: %s%s\n", comp.SystemID, comp.ID, comp.Label, desc)
			}
		}
	}
}

func listPersons(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	items := lister.ListPersons(arch)

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, person := range items {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s"}`, person.ID, person.Label)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Persons (%d):\n\n", len(items))
		for _, person := range items {
			_, _ = fmt.Fprintf(w, "  • %s: %s\n", person.ID, person.Label)
		}
	}
}

func listDataStores(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	items := lister.ListDataStores(arch)

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, ds := range items {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s", "system": "%s"}`, ds.ID, ds.Label, ds.SystemID)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Data Stores (%d):\n\n", len(items))
		for _, ds := range items {
			_, _ = fmt.Fprintf(w, "  • %s.%s: %s\n", ds.SystemID, ds.ID, ds.Label)
		}
	}
}

func listQueues(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	items := lister.ListQueues(arch)

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, q := range items {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s", "system": "%s"}`, q.ID, q.Label, q.SystemID)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Queues (%d):\n\n", len(items))
		for _, q := range items {
			_, _ = fmt.Fprintf(w, "  • %s.%s: %s\n", q.SystemID, q.ID, q.Label)
		}
	}
}

func listScenarios(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	items := lister.ListScenarios(arch)

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, scenario := range items {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "title": "%s"}`, scenario.ID, scenario.Title)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Scenarios (%d):\n\n", len(items))
		for _, scenario := range items {
			_, _ = fmt.Fprintf(w, "  • %s: %s\n", scenario.ID, scenario.Title)
		}
	}
}

func listADRs(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	items := lister.ListADRs(arch)

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, adr := range items {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "title": "%s"}`, adr.ID, adr.Title)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "ADRs (%d):\n\n", len(items))
		for _, adr := range items {
			title := adr.Title
			if title == "" {
				title = "(no title)"
			}
			_, _ = fmt.Fprintf(w, "  • %s: %s\n", adr.ID, title)
		}
	}
}

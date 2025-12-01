// apps/cli/cmd/list.go
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

func runList(stdout, stderr io.Writer) int {
	listCmd := flag.NewFlagSet("list", flag.ContinueOnError)
	listCmd.SetOutput(stderr)
	listJSON := listCmd.Bool("json", false, "output as JSON")
	listFile := listCmd.String("file", "", "architecture file path")

	if err := listCmd.Parse(os.Args[2:]); err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing list flags: %v", err)))
		return 1
	}

	if listCmd.NArg() < 1 {
		_, _ = fmt.Fprintln(stderr, "Usage: sruja list <type> [--file <path>] [--json]")
		_, _ = fmt.Fprintln(stderr, "Types: systems, containers, components, persons, datastores, queues, scenarios, adrs, entities, events")
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
	case "entities":
		listEntities(arch, jsonOutput, stdout)
	case "events":
		listEvents(arch, jsonOutput, stdout)
	default:
		_, _ = fmt.Fprintf(stderr, "Error: unknown type '%s'\n", elementType)
		_, _ = fmt.Fprintln(stderr, "Types: systems, containers, components, persons, datastores, queues, scenarios, adrs, entities, events")
		return fmt.Errorf("unknown type: %s", elementType)
	}
	return nil
}

func listSystems(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, sys := range arch.Systems {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s"}`, sys.ID, sys.Label)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Systems (%d):\n\n", len(arch.Systems))
		for _, sys := range arch.Systems {
			desc := ""
			if sys.Description != nil {
				desc = fmt.Sprintf(" - %s", *sys.Description)
			}
			_, _ = fmt.Fprintf(w, "  • %s: %s%s\n", sys.ID, sys.Label, desc)
		}
	}
}

func listContainers(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	count := 0
	for _, sys := range arch.Systems {
		count += len(sys.Containers)
	}

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		first := true
		for _, sys := range arch.Systems {
			for _, cont := range sys.Containers {
				if !first {
					_, _ = fmt.Fprint(w, ", ")
				}
				first = false
				_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s", "system": "%s"}`, cont.ID, cont.Label, sys.ID)
			}
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Containers (%d):\n\n", count)
		for _, sys := range arch.Systems {
			for _, cont := range sys.Containers {
				desc := ""
				if cont.Description != nil {
					desc = fmt.Sprintf(" - %s", *cont.Description)
				}
				_, _ = fmt.Fprintf(w, "  • %s.%s: %s%s\n", sys.ID, cont.ID, cont.Label, desc)
			}
		}
	}
}

func listComponents(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	count := 0
	for _, sys := range arch.Systems {
		count += len(sys.Components)
		for _, cont := range sys.Containers {
			count += len(cont.Components)
		}
	}

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		first := true
		for _, sys := range arch.Systems {
			for _, comp := range sys.Components {
				if !first {
					_, _ = fmt.Fprint(w, ", ")
				}
				first = false
				_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s", "system": "%s"}`, comp.ID, comp.Label, sys.ID)
			}
			for _, cont := range sys.Containers {
				for _, comp := range cont.Components {
					if !first {
						_, _ = fmt.Fprint(w, ", ")
					}
					first = false
					_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s", "system": "%s", "container": "%s"}`, comp.ID, comp.Label, sys.ID, cont.ID)
				}
			}
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Components (%d):\n\n", count)
		for _, sys := range arch.Systems {
			for _, comp := range sys.Components {
				desc := ""
				if comp.Description != nil {
					desc = fmt.Sprintf(" - %s", *comp.Description)
				}
				_, _ = fmt.Fprintf(w, "  • %s.%s: %s%s\n", sys.ID, comp.ID, comp.Label, desc)
			}
			for _, cont := range sys.Containers {
				for _, comp := range cont.Components {
					desc := ""
					if comp.Description != nil {
						desc = fmt.Sprintf(" - %s", *comp.Description)
					}
					_, _ = fmt.Fprintf(w, "  • %s.%s.%s: %s%s\n", sys.ID, cont.ID, comp.ID, comp.Label, desc)
				}
			}
		}
	}
}

func listPersons(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, person := range arch.Persons {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s"}`, person.ID, person.Label)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Persons (%d):\n\n", len(arch.Persons))
		for _, person := range arch.Persons {
			_, _ = fmt.Fprintf(w, "  • %s: %s\n", person.ID, person.Label)
		}
	}
}

func listDataStores(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	count := 0
	for _, sys := range arch.Systems {
		count += len(sys.DataStores)
	}

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		first := true
		for _, sys := range arch.Systems {
			for _, ds := range sys.DataStores {
				if !first {
					_, _ = fmt.Fprint(w, ", ")
				}
				first = false
				_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s", "system": "%s"}`, ds.ID, ds.Label, sys.ID)
			}
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Data Stores (%d):\n\n", count)
		for _, sys := range arch.Systems {
			for _, ds := range sys.DataStores {
				desc := ""
				if ds.Description != nil {
					desc = fmt.Sprintf(" - %s", *ds.Description)
				}
				_, _ = fmt.Fprintf(w, "  • %s.%s: %s%s\n", sys.ID, ds.ID, ds.Label, desc)
			}
		}
	}
}

func listQueues(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	count := 0
	for _, sys := range arch.Systems {
		count += len(sys.Queues)
	}

	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		first := true
		for _, sys := range arch.Systems {
			for _, q := range sys.Queues {
				if !first {
					_, _ = fmt.Fprint(w, ", ")
				}
				first = false
				_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s", "system": "%s"}`, q.ID, q.Label, sys.ID)
			}
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Queues (%d):\n\n", count)
		for _, sys := range arch.Systems {
			for _, q := range sys.Queues {
				desc := ""
				if q.Description != nil {
					desc = fmt.Sprintf(" - %s", *q.Description)
				}
				_, _ = fmt.Fprintf(w, "  • %s.%s: %s%s\n", sys.ID, q.ID, q.Label, desc)
			}
		}
	}
}

func listScenarios(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, scenario := range arch.Scenarios {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			id := scenario.ID
			if id == "" {
				id = "unnamed"
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "title": "%s", "steps": %d}`, id, scenario.Title, len(scenario.Steps))
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Scenarios (%d):\n\n", len(arch.Scenarios))
		for _, scenario := range arch.Scenarios {
			id := scenario.ID
			if id == "" {
				id = "unnamed"
			}
			_, _ = fmt.Fprintf(w, "  • %s: %s (%d steps)\n", id, scenario.Title, len(scenario.Steps))
		}
	}
}

func listADRs(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, adr := range arch.ADRs {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			title := ""
			if adr.Title != nil {
				title = *adr.Title
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "title": "%s"}`, adr.ID, title)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "ADRs (%d):\n\n", len(arch.ADRs))
		for _, adr := range arch.ADRs {
			title := ""
			if adr.Title != nil {
				title = *adr.Title
			}
			_, _ = fmt.Fprintf(w, "  • %s: %s\n", adr.ID, title)
		}
	}
}

func listEntities(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	// Collect all entities from architecture and systems/containers
	var entities []*language.Entity
	entities = append(entities, arch.Entities...)
	for _, sys := range arch.Systems {
		entities = append(entities, sys.Entities...)
		for _, cont := range sys.Containers {
			entities = append(entities, cont.Entities...)
		}
	}
	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, e := range entities {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s"}`, e.ID, e.Label)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Entities (%d):\n\n", len(entities))
		for _, e := range entities {
			_, _ = fmt.Fprintf(w, "  • %s: %s\n", e.ID, e.Label)
		}
	}
}

func listEvents(arch *language.Architecture, jsonOutput bool, w io.Writer) {
	// Collect events from architecture and systems/containers
	var events []*language.DomainEvent
	events = append(events, arch.Events...)
	for _, sys := range arch.Systems {
		events = append(events, sys.Events...)
		for _, cont := range sys.Containers {
			events = append(events, cont.Events...)
		}
	}
	if jsonOutput {
		_, _ = fmt.Fprint(w, "[")
		for i, ev := range events {
			if i > 0 {
				_, _ = fmt.Fprint(w, ", ")
			}
			_, _ = fmt.Fprintf(w, `{"id": "%s", "label": "%s"}`, ev.ID, ev.Label)
		}
		_, _ = fmt.Fprintln(w, "]")
	} else {
		_, _ = fmt.Fprintf(w, "Events (%d):\n\n", len(events))
		for _, ev := range events {
			_, _ = fmt.Fprintf(w, "  • %s: %s\n", ev.ID, ev.Label)
		}
	}
}

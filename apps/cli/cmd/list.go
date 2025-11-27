// apps/cli/cmd/list.go
package main

import (
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

func runList() {
	listCmd := flag.NewFlagSet("list", flag.ExitOnError)
	listJSON := listCmd.Bool("json", false, "output as JSON")
	listFile := listCmd.String("file", "", "architecture file path")

	listCmd.Parse(os.Args[2:])

	if listCmd.NArg() < 1 {
		fmt.Println("Usage: sruja list <type> [--file <path>] [--json]")
		fmt.Println("Types: systems, containers, components, persons, datastores, queues, journeys, adrs, entities, events")
		fmt.Println("Example: sruja list systems --file architecture.sruja")
		os.Exit(1)
	}

	elementType := strings.ToLower(listCmd.Arg(0))
	filePath := *listFile

	if filePath == "" {
		// Try to find .sruja files in current directory
		files, err := os.ReadDir(".")
		if err == nil {
			for _, file := range files {
				if !file.IsDir() && len(file.Name()) > 6 && file.Name()[len(file.Name())-6:] == ".sruja" {
					filePath = file.Name()
					break
				}
			}
		}
	}

	if filePath == "" {
		fmt.Println("Error: no architecture file found. Use --file to specify.")
		os.Exit(1)
	}

	// Parse the architecture file
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

	if program.Architecture == nil {
		fmt.Println("Error: no architecture found in file")
		os.Exit(1)
	}

	arch := program.Architecture

	switch elementType {
	case "systems":
		listSystems(arch, *listJSON)
	case "containers":
		listContainers(arch, *listJSON)
	case "components":
		listComponents(arch, *listJSON)
	case "persons":
		listPersons(arch, *listJSON)
	case "datastores":
		listDataStores(arch, *listJSON)
	case "queues":
		listQueues(arch, *listJSON)
	case "journeys":
		listJourneys(arch, *listJSON)
	case "adrs":
		listADRs(arch, *listJSON)
	case "entities":
		listEntities(arch, *listJSON)
	case "events":
		listEvents(arch, *listJSON)
	default:
		fmt.Printf("Error: unknown type '%s'\n", elementType)
		fmt.Println("Types: systems, containers, components, persons, datastores, queues, journeys, adrs, entities, events")
		os.Exit(1)
	}
}

func listSystems(arch *language.Architecture, jsonOutput bool) {
	if jsonOutput {
		fmt.Print("[")
		for i, sys := range arch.Systems {
			if i > 0 {
				fmt.Print(", ")
			}
			fmt.Printf(`{"id": "%s", "label": "%s"}`, sys.ID, sys.Label)
		}
		fmt.Println("]")
	} else {
		fmt.Printf("Systems (%d):\n\n", len(arch.Systems))
		for _, sys := range arch.Systems {
			desc := ""
			if sys.Description != nil {
				desc = fmt.Sprintf(" - %s", *sys.Description)
			}
			fmt.Printf("  • %s: %s%s\n", sys.ID, sys.Label, desc)
		}
	}
}

func listContainers(arch *language.Architecture, jsonOutput bool) {
	count := 0
	for _, sys := range arch.Systems {
		count += len(sys.Containers)
	}

	if jsonOutput {
		fmt.Print("[")
		first := true
		for _, sys := range arch.Systems {
			for _, cont := range sys.Containers {
				if !first {
					fmt.Print(", ")
				}
				first = false
				fmt.Printf(`{"id": "%s", "label": "%s", "system": "%s"}`, cont.ID, cont.Label, sys.ID)
			}
		}
		fmt.Println("]")
	} else {
		fmt.Printf("Containers (%d):\n\n", count)
		for _, sys := range arch.Systems {
			for _, cont := range sys.Containers {
				desc := ""
				if cont.Description != nil {
					desc = fmt.Sprintf(" - %s", *cont.Description)
				}
				fmt.Printf("  • %s.%s: %s%s\n", sys.ID, cont.ID, cont.Label, desc)
			}
		}
	}
}

func listComponents(arch *language.Architecture, jsonOutput bool) {
	count := 0
	for _, sys := range arch.Systems {
		count += len(sys.Components)
		for _, cont := range sys.Containers {
			count += len(cont.Components)
		}
	}

	if jsonOutput {
		fmt.Print("[")
		first := true
		for _, sys := range arch.Systems {
			for _, comp := range sys.Components {
				if !first {
					fmt.Print(", ")
				}
				first = false
				fmt.Printf(`{"id": "%s", "label": "%s", "system": "%s"}`, comp.ID, comp.Label, sys.ID)
			}
			for _, cont := range sys.Containers {
				for _, comp := range cont.Components {
					if !first {
						fmt.Print(", ")
					}
					first = false
					fmt.Printf(`{"id": "%s", "label": "%s", "system": "%s", "container": "%s"}`, comp.ID, comp.Label, sys.ID, cont.ID)
				}
			}
		}
		fmt.Println("]")
	} else {
		fmt.Printf("Components (%d):\n\n", count)
		for _, sys := range arch.Systems {
			for _, comp := range sys.Components {
				desc := ""
				if comp.Description != nil {
					desc = fmt.Sprintf(" - %s", *comp.Description)
				}
				fmt.Printf("  • %s.%s: %s%s\n", sys.ID, comp.ID, comp.Label, desc)
			}
			for _, cont := range sys.Containers {
				for _, comp := range cont.Components {
					desc := ""
					if comp.Description != nil {
						desc = fmt.Sprintf(" - %s", *comp.Description)
					}
					fmt.Printf("  • %s.%s.%s: %s%s\n", sys.ID, cont.ID, comp.ID, comp.Label, desc)
				}
			}
		}
	}
}

func listPersons(arch *language.Architecture, jsonOutput bool) {
	if jsonOutput {
		fmt.Print("[")
		for i, person := range arch.Persons {
			if i > 0 {
				fmt.Print(", ")
			}
			fmt.Printf(`{"id": "%s", "label": "%s"}`, person.ID, person.Label)
		}
		fmt.Println("]")
	} else {
		fmt.Printf("Persons (%d):\n\n", len(arch.Persons))
		for _, person := range arch.Persons {
			fmt.Printf("  • %s: %s\n", person.ID, person.Label)
		}
	}
}

func listDataStores(arch *language.Architecture, jsonOutput bool) {
	count := 0
	for _, sys := range arch.Systems {
		count += len(sys.DataStores)
	}

	if jsonOutput {
		fmt.Print("[")
		first := true
		for _, sys := range arch.Systems {
			for _, ds := range sys.DataStores {
				if !first {
					fmt.Print(", ")
				}
				first = false
				fmt.Printf(`{"id": "%s", "label": "%s", "system": "%s"}`, ds.ID, ds.Label, sys.ID)
			}
		}
		fmt.Println("]")
	} else {
		fmt.Printf("Data Stores (%d):\n\n", count)
		for _, sys := range arch.Systems {
			for _, ds := range sys.DataStores {
				desc := ""
				if ds.Description != nil {
					desc = fmt.Sprintf(" - %s", *ds.Description)
				}
				fmt.Printf("  • %s.%s: %s%s\n", sys.ID, ds.ID, ds.Label, desc)
			}
		}
	}
}

func listQueues(arch *language.Architecture, jsonOutput bool) {
	count := 0
	for _, sys := range arch.Systems {
		count += len(sys.Queues)
	}

	if jsonOutput {
		fmt.Print("[")
		first := true
		for _, sys := range arch.Systems {
			for _, q := range sys.Queues {
				if !first {
					fmt.Print(", ")
				}
				first = false
				fmt.Printf(`{"id": "%s", "label": "%s", "system": "%s"}`, q.ID, q.Label, sys.ID)
			}
		}
		fmt.Println("]")
	} else {
		fmt.Printf("Queues (%d):\n\n", count)
		for _, sys := range arch.Systems {
			for _, q := range sys.Queues {
				desc := ""
				if q.Description != nil {
					desc = fmt.Sprintf(" - %s", *q.Description)
				}
				fmt.Printf("  • %s.%s: %s%s\n", sys.ID, q.ID, q.Label, desc)
			}
		}
	}
}

func listJourneys(arch *language.Architecture, jsonOutput bool) {
	if jsonOutput {
		fmt.Print("[")
		for i, journey := range arch.Journeys {
			if i > 0 {
				fmt.Print(", ")
			}
			fmt.Printf(`{"id": "%s", "title": "%s", "steps": %d}`, journey.ID, journey.Title, len(journey.Steps))
		}
		fmt.Println("]")
	} else {
		fmt.Printf("Journeys (%d):\n\n", len(arch.Journeys))
		for _, journey := range arch.Journeys {
			fmt.Printf("  • %s: %s (%d steps)\n", journey.ID, journey.Title, len(journey.Steps))
		}
	}
}

func listADRs(arch *language.Architecture, jsonOutput bool) {
	if jsonOutput {
		fmt.Print("[")
		for i, adr := range arch.ADRs {
			if i > 0 {
				fmt.Print(", ")
			}
			fmt.Printf(`{"id": "%s", "title": "%s"}`, adr.ID, adr.Title)
		}
		fmt.Println("]")
	} else {
		fmt.Printf("ADRs (%d):\n\n", len(arch.ADRs))
		for _, adr := range arch.ADRs {
			fmt.Printf("  • %s: %s\n", adr.ID, adr.Title)
		}
	}
}

func listEntities(arch *language.Architecture, jsonOutput bool) {
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
		fmt.Print("[")
		for i, e := range entities {
			if i > 0 {
				fmt.Print(", ")
			}
			fmt.Printf(`{"name": "%s"}`, e.Name)
		}
		fmt.Println("]")
	} else {
		fmt.Printf("Entities (%d):\n\n", len(entities))
		for _, e := range entities {
			fmt.Printf("  • %s\n", e.Name)
		}
	}
}

func listEvents(arch *language.Architecture, jsonOutput bool) {
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
		fmt.Print("[")
		for i, ev := range events {
			if i > 0 {
				fmt.Print(", ")
			}
			ver := ""
			if ev.Body != nil && ev.Body.Version != nil {
				ver = *ev.Body.Version
			}
			fmt.Printf(`{"name": "%s", "version": "%s"}`, ev.Name, ver)
		}
		fmt.Println("]")
	} else {
		fmt.Printf("Events (%d):\n\n", len(events))
		for _, ev := range events {
			extra := ""
			if ev.Body != nil && ev.Body.Version != nil {
				extra = fmt.Sprintf(" v%s", *ev.Body.Version)
			}
			fmt.Printf("  • %s%s\n", ev.Name, extra)
		}
	}
}

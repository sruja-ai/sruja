// apps/cli/cmd/list.go
package main

import (
	"flag"
	"fmt"
	"io"
	"strings"

	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/language"
	"golang.org/x/text/cases"
	lang "golang.org/x/text/language"
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

	rawType := strings.ToLower(listCmd.Arg(0))
	elementType := rawType

	// Normalize plural to singular
	switch rawType {
	case "systems":
		elementType = "system"
	case "containers":
		elementType = "container"
	case "components":
		elementType = "component"
	case "persons":
		elementType = "person"
	case "datastores":
		elementType = "datastore"
	case "queues":
		elementType = "queue"
	case "scenarios":
		elementType = "scenario"
	case "adrs":
		elementType = "adr"
	}

	// Validate type
	validTypes := map[string]bool{
		"system": true, "container": true, "component": true,
		"person": true, "datastore": true, "queue": true,
		"scenario": true, "adr": true,
	}
	if !validTypes[elementType] {
		_, _ = fmt.Fprintf(stderr, "Error: unknown type '%s'\n", rawType)
		_, _ = fmt.Fprintln(stderr, "Types: systems, containers, components, persons, datastores, queues, scenarios, adrs")
		return 1
	}

	filePath := findSrujaFile(*listFile)

	if filePath == "" {
		_, _ = fmt.Fprintln(stderr, "Error: no architecture file found. Use --file to specify.")
		return 1
	}

	program, err := parseArchitectureFile(filePath, stderr)
	if err != nil {
		return 1
	}

	if program.Model == nil {
		_, _ = fmt.Fprintln(stderr, "Error: no model found in file")
		return 1
	}

	if err := listElementTypeFromModel(program.Model, elementType, *listJSON, stdout, stderr); err != nil {
		return 1
	}
	return 0
}

//nolint:unparam
func listElementTypeFromModel(model *language.ModelBlock, elementType string, jsonOutput bool, stdout, _ io.Writer) error {
	// Extract elements from Model block
	var elements []map[string]string
	var collectElements func(elem *language.LikeC4ElementDef, parentID string)
	collectElements = func(elem *language.LikeC4ElementDef, parentID string) {
		if elem == nil {
			return
		}
		id := elem.GetID()
		if id == "" {
			return
		}
		kind := elem.GetKind()
		title := ""
		titlePtr := elem.GetTitle()
		if titlePtr != nil {
			title = *titlePtr
		}
		if kind == elementType || (elementType == "datastore" && kind == "database") {
			info := map[string]string{
				"id":    id,
				"label": title,
				"kind":  kind,
			}
			if parentID != "" {
				info["parent"] = parentID
			}
			elements = append(elements, info)
		}
		// Recursively process nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					fqn := id
					if parentID != "" {
						fqn = parentID + "." + id
					}
					collectElements(bodyItem.Element, fqn)
				}
			}
		}
	}

	// Also check for scenarios and ADRs in model items
	for _, item := range model.Items {
		if item.ElementDef != nil {
			collectElements(item.ElementDef, "")
		}
		if elementType == "scenario" && item.Scenario != nil {
			title := ""
			if item.Scenario.Title != nil {
				title = *item.Scenario.Title
			}
			elements = append(elements, map[string]string{
				"id":    item.Scenario.ID,
				"label": title,
				"kind":  "scenario",
			})
		}
		if elementType == "adr" && item.ADR != nil {
			title := ""
			if item.ADR.Title != nil {
				title = *item.ADR.Title
			}
			elements = append(elements, map[string]string{
				"id":    item.ADR.ID,
				"label": title,
				"kind":  "adr",
			})
		}
	}

	if jsonOutput {
		_, _ = fmt.Fprint(stdout, "[")
		for i, elem := range elements {
			if i > 0 {
				_, _ = fmt.Fprint(stdout, ", ")
			}
			_, _ = fmt.Fprintf(stdout, `{"id": "%s", "label": "%s"`, elem["id"], elem["label"])
			if parent, ok := elem["parent"]; ok {
				_, _ = fmt.Fprintf(stdout, `, "parent": "%s"`, parent)
			}
			_, _ = fmt.Fprint(stdout, "}")
		}
		_, _ = fmt.Fprintln(stdout, "]")
	} else {
		_, _ = fmt.Fprintf(stdout, "%s (%d):\n\n", cases.Title(lang.Und).String(elementType), len(elements))
		for _, elem := range elements {
			parent := ""
			if p, ok := elem["parent"]; ok {
				parent = p + "."
			}
			_, _ = fmt.Fprintf(stdout, "  • %s%s: %s\n", parent, elem["id"], elem["label"])
		}
	}
	return nil
}

// Legacy list functions removed - Architecture struct removed (old syntax no longer supported)
// Use listElementTypeFromModel instead which works with LikeC4 ModelBlock

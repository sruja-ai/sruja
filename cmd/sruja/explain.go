// apps/cli/cmd/explain.go
package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/language"
)

func runExplain() {
	explainCmd := flag.NewFlagSet("explain", flag.ExitOnError)
	explainJSON := explainCmd.Bool("json", false, "output as JSON")
	explainFile := explainCmd.String("file", "", "architecture file path")

	if err := explainCmd.Parse(os.Args[2:]); err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error parsing explain flags: %v", err)))
		os.Exit(1)
	}

	if explainCmd.NArg() < 1 {
		fmt.Println("Usage: sruja explain <element-id> [--file <path>] [--json]")
		fmt.Println("Example: sruja explain BillingAPI --file architecture.sruja")
		os.Exit(1)
	}

	elementID := explainCmd.Arg(0)
	filePath := *explainFile

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

	// Create explainer
	explainer := dx.NewExplainer(program)
	explanation, err := explainer.ExplainElement(elementID)
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("%v", err)))
		fmt.Println(dx.Dim("\nTip: Use 'sruja list systems' or 'sruja list containers' to see available elements."))
		os.Exit(1)
	}

	// Output
	if *explainJSON {
		// JSON output (simplified for now)
		fmt.Printf(`{
  "id": "%s",
  "description": "%s",
  "incoming_relations": %d,
  "outgoing_relations": %d,
  "dependencies": %d,
  "adrs": %d,
  "journeys": %d
}
`, explanation.ID, explanation.Description, len(explanation.Relations.Incoming), len(explanation.Relations.Outgoing), len(explanation.Dependencies), len(explanation.ADRs), len(explanation.Journeys))
	} else {
		// Human-readable output
		fmt.Println(explanation.Format())
	}
}

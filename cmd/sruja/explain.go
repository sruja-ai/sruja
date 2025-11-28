// apps/cli/cmd/explain.go
package main

import (
	"flag"
	"fmt"
	"io"
	"os"

	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/language"
)

func runExplain(stdout, stderr io.Writer) int {
	explainCmd := flag.NewFlagSet("explain", flag.ContinueOnError)
	explainCmd.SetOutput(stderr)
	explainJSON := explainCmd.Bool("json", false, "output as JSON")
	explainFile := explainCmd.String("file", "", "architecture file path")

	if err := explainCmd.Parse(os.Args[2:]); err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing explain flags: %v", err)))
		return 1
	}

	if explainCmd.NArg() < 1 {
		_, _ = fmt.Fprintln(stderr, "Usage: sruja explain <element-id> [--file <path>] [--json]")
		_, _ = fmt.Fprintln(stderr, "Example: sruja explain BillingAPI --file architecture.sruja")
		return 1
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
		_, _ = fmt.Fprintln(stderr, "Error: no architecture file found. Use --file to specify.")
		return 1
	}

	// Parse the architecture file
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

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Parser Error: %v\n", err)
		return 1
	}

	// Create explainer
	explainer := dx.NewExplainer(program)
	explanation, err := explainer.ExplainElement(elementID)
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("%v", err)))
		_, _ = fmt.Fprintln(stderr, dx.Dim("\nTip: Use 'sruja list systems' or 'sruja list containers' to see available elements."))
		return 1
	}

	// Output
	if *explainJSON {
		// JSON output (simplified for now)
		_, _ = fmt.Fprintf(stdout, `{
  "id": "%s",
  "description": "%s",
  "incoming_relations": %d,
  "outgoing_relations": %d,
  "dependencies": %d,
  "adrs": %d,
  "scenarios": %d
}
`, explanation.ID, explanation.Description, len(explanation.Relations.Incoming), len(explanation.Relations.Outgoing), len(explanation.Dependencies), len(explanation.ADRs), len(explanation.Scenarios))
	} else {
		// Human-readable output
		_, _ = fmt.Fprintln(stdout, explanation.Format())
	}
	return 0
}

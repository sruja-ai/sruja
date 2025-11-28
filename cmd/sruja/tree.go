// apps/cli/cmd/tree.go
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

func runTree(stdout, stderr io.Writer) int {
	treeCmd := flag.NewFlagSet("tree", flag.ContinueOnError)
	treeCmd.SetOutput(stderr)
	treeFile := treeCmd.String("file", "", "architecture file path")
	treeSystem := treeCmd.String("system", "", "show tree for specific system only")
	treeJSON := treeCmd.Bool("json", false, "output as JSON")

	if err := treeCmd.Parse(os.Args[2:]); err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error parsing tree flags: %v", err)))
		return 1
	}

	filePath := *treeFile
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
		_, _ = fmt.Fprintln(stderr, dx.Error("No architecture file found. Use --file to specify."))
		return 1
	}

	// Parse the architecture file
	content, err := os.ReadFile(filePath)
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error reading file: %v", err)))
		return 1
	}

	p, err := language.NewParser()
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Error creating parser: %v", err)))
		return 1
	}

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("Parser Error: %v", err)))
		return 1
	}

	if program.Architecture == nil {
		_, _ = fmt.Fprintln(stderr, dx.Error("No architecture found in file"))
		return 1
	}

	arch := program.Architecture

	if *treeSystem != "" {
		// Show tree for specific system
		var found *language.System
		for _, sys := range arch.Systems {
			if sys.ID == *treeSystem {
				found = sys
				break
			}
		}
		if found == nil {
			_, _ = fmt.Fprintln(stderr, dx.Error(fmt.Sprintf("System '%s' not found", *treeSystem)))
			return 1
		}
		printSystemTree(found, *treeJSON, 0, stdout)
	} else {
		// Show tree for entire architecture
		_, _ = fmt.Fprintln(stdout, dx.Header(fmt.Sprintf("Architecture: %s", arch.Name)))
		for i, sys := range arch.Systems {
			printSystemTree(sys, *treeJSON, 0, stdout)
			if i < len(arch.Systems)-1 {
				_, _ = fmt.Fprintln(stdout)
			}
		}
	}
	return 0
}

func printSystemTree(sys *language.System, jsonOutput bool, indent int, w io.Writer) {
	indentStr := strings.Repeat("  ", indent)

	if jsonOutput {
		// JSON output
		_, _ = fmt.Fprintf(w, "%s{\n", indentStr)
		_, _ = fmt.Fprintf(w, "%s  \"type\": \"system\",\n", indentStr)
		_, _ = fmt.Fprintf(w, "%s  \"id\": \"%s\",\n", indentStr, sys.ID)
		_, _ = fmt.Fprintf(w, "%s  \"label\": \"%s\",\n", indentStr, sys.Label)
		_, _ = fmt.Fprintf(w, "%s  \"containers\": [\n", indentStr)
		for i, cont := range sys.Containers {
			_, _ = fmt.Fprintf(w, "%s    {\n", indentStr)
			_, _ = fmt.Fprintf(w, "%s      \"type\": \"container\",\n", indentStr)
			_, _ = fmt.Fprintf(w, "%s      \"id\": \"%s\",\n", indentStr, cont.ID)
			_, _ = fmt.Fprintf(w, "%s      \"label\": \"%s\",\n", indentStr, cont.Label)
			_, _ = fmt.Fprintf(w, "%s      \"components\": [\n", indentStr)
			for j, comp := range cont.Components {
				_, _ = fmt.Fprintf(w, "%s        {\"type\": \"component\", \"id\": \"%s\", \"label\": \"%s\"}", indentStr, comp.ID, comp.Label)
				if j < len(cont.Components)-1 {
					_, _ = fmt.Fprint(w, ",")
				}
				_, _ = fmt.Fprintln(w)
			}
			_, _ = fmt.Fprintf(w, "%s      ]\n", indentStr)
			_, _ = fmt.Fprintf(w, "%s    }", indentStr)
			if i < len(sys.Containers)-1 {
				_, _ = fmt.Fprint(w, ",")
			}
			_, _ = fmt.Fprintln(w)
		}
		_, _ = fmt.Fprintf(w, "%s  ]\n", indentStr)
		_, _ = fmt.Fprintf(w, "%s}", indentStr)
		if indent == 0 {
			_, _ = fmt.Fprintln(w)
		}
	} else {
		// Human-readable tree output
		useColor := dx.SupportsColor()

		// System
		sysLine := indentStr + dx.Code("system")
		sysLine += " " + dx.Bold(sys.ID)
		if sys.Label != sys.ID {
			sysLine += " \"" + sys.Label + "\""
		}
		_, _ = fmt.Fprintln(w, sysLine)

		// Containers
		for i, cont := range sys.Containers {
			isLastContainer := i == len(sys.Containers)-1 && len(sys.Components) == 0 && len(sys.DataStores) == 0 && len(sys.Queues) == 0
			connector := "├─"
			if isLastContainer {
				connector = "└─"
			}
			contLine := fmt.Sprintf("%s  %s ", indentStr, connector)
			contLine += dx.Code("container") + " "
			contLine += dx.Bold(cont.ID)
			if cont.Label != cont.ID {
				contLine += " \"" + cont.Label + "\""
			}
			_, _ = fmt.Fprintln(w, contLine)

			// Components in container
			totalItems := len(cont.Components) + len(cont.DataStores) + len(cont.Queues)
			itemIdx := 0
			for _, comp := range cont.Components {
				itemIdx++
				isLast := itemIdx == totalItems
				connector := "├─"
				if isLast {
					connector = "└─"
				}
				compLine := fmt.Sprintf("%s    %s %s ", indentStr, connector, dx.Code("component"))
				compLine += comp.ID
				if comp.Label != comp.ID {
					compLine += " \"" + comp.Label + "\""
				}
				_, _ = fmt.Fprintln(w, compLine)
			}

			// Data stores in container
			for _, ds := range cont.DataStores {
				itemIdx++
				isLast := itemIdx == totalItems
				connector := "├─"
				if isLast {
					connector = "└─"
				}
				dsLine := fmt.Sprintf("%s    %s %s ", indentStr, connector, dx.Code("datastore"))
				dsLine += ds.ID
				if ds.Label != ds.ID {
					dsLine += " \"" + ds.Label + "\""
				}
				_, _ = fmt.Fprintln(w, dsLine)
			}

			// Queues in container
			for _, q := range cont.Queues {
				itemIdx++
				isLast := itemIdx == totalItems
				connector := "├─"
				if isLast {
					connector = "└─"
				}
				qLine := fmt.Sprintf("%s    %s %s ", indentStr, connector, dx.Code("queue"))
				qLine += q.ID
				if q.Label != q.ID {
					qLine += " \"" + q.Label + "\""
				}
				_, _ = fmt.Fprintln(w, qLine)
			}
		}

		// Components directly in system
		if len(sys.Components) > 0 {
			for i, comp := range sys.Components {
				isLast := i == len(sys.Components)-1 && len(sys.DataStores) == 0 && len(sys.Queues) == 0
				connector := "├─"
				if isLast {
					connector = "└─"
				}
				compLine := fmt.Sprintf("%s  %s %s ", indentStr, connector, dx.Colorize(dx.ColorGreen, "component", useColor))
				compLine += comp.ID
				if comp.Label != comp.ID {
					compLine += " \"" + comp.Label + "\""
				}
				_, _ = fmt.Fprintln(w, compLine)
			}
		}

		// Data stores directly in system
		if len(sys.DataStores) > 0 {
			for i, ds := range sys.DataStores {
				isLast := i == len(sys.DataStores)-1 && len(sys.Queues) == 0
				connector := "├─"
				if isLast {
					connector = "└─"
				}
				dsLine := fmt.Sprintf("%s  %s %s ", indentStr, connector, dx.Colorize(dx.ColorPurple, "datastore", useColor))
				dsLine += ds.ID
				if ds.Label != ds.ID {
					dsLine += " \"" + ds.Label + "\""
				}
				_, _ = fmt.Fprintln(w, dsLine)
			}
		}

		// Queues directly in system
		if len(sys.Queues) > 0 {
			for i, q := range sys.Queues {
				isLast := i == len(sys.Queues)-1
				connector := "├─"
				if isLast {
					connector = "└─"
				}
				qLine := fmt.Sprintf("%s  %s %s ", indentStr, connector, dx.Colorize(dx.ColorPurple, "queue", useColor))
				qLine += q.ID
				if q.Label != q.ID {
					qLine += " \"" + q.Label + "\""
				}
				_, _ = fmt.Fprintln(w, qLine)
			}
		}
	}
}

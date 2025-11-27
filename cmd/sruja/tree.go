// apps/cli/cmd/tree.go
package main

import (
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/language"
)

func runTree() {
	treeCmd := flag.NewFlagSet("tree", flag.ExitOnError)
	treeFile := treeCmd.String("file", "", "architecture file path")
	treeSystem := treeCmd.String("system", "", "show tree for specific system only")
	treeJSON := treeCmd.Bool("json", false, "output as JSON")

	if err := treeCmd.Parse(os.Args[2:]); err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error parsing tree flags: %v", err)))
		os.Exit(1)
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
		fmt.Println(dx.Error("No architecture file found. Use --file to specify."))
		os.Exit(1)
	}

	// Parse the architecture file
	content, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error reading file: %v", err)))
		os.Exit(1)
	}

	p, err := language.NewParser()
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error creating parser: %v", err)))
		os.Exit(1)
	}

	program, err := p.Parse(filePath, string(content))
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Parser Error: %v", err)))
		os.Exit(1)
	}

	if program.Architecture == nil {
		fmt.Println(dx.Error("No architecture found in file"))
		os.Exit(1)
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
			fmt.Println(dx.Error(fmt.Sprintf("System '%s' not found", *treeSystem)))
			os.Exit(1)
		}
		printSystemTree(found, *treeJSON, 0)
	} else {
		// Show tree for entire architecture
		fmt.Println(dx.Header(fmt.Sprintf("Architecture: %s", arch.Name)))
		for i, sys := range arch.Systems {
			printSystemTree(sys, *treeJSON, 0)
			if i < len(arch.Systems)-1 {
				fmt.Println()
			}
		}
	}
}

func printSystemTree(sys *language.System, jsonOutput bool, indent int) {
	indentStr := strings.Repeat("  ", indent)

	if jsonOutput {
		// JSON output
		fmt.Printf("%s{\n", indentStr)
		fmt.Printf("%s  \"type\": \"system\",\n", indentStr)
		fmt.Printf("%s  \"id\": \"%s\",\n", indentStr, sys.ID)
		fmt.Printf("%s  \"label\": \"%s\",\n", indentStr, sys.Label)
		fmt.Printf("%s  \"containers\": [\n", indentStr)
		for i, cont := range sys.Containers {
			fmt.Printf("%s    {\n", indentStr)
			fmt.Printf("%s      \"type\": \"container\",\n", indentStr)
			fmt.Printf("%s      \"id\": \"%s\",\n", indentStr, cont.ID)
			fmt.Printf("%s      \"label\": \"%s\",\n", indentStr, cont.Label)
			fmt.Printf("%s      \"components\": [\n", indentStr)
			for j, comp := range cont.Components {
				fmt.Printf("%s        {\"type\": \"component\", \"id\": \"%s\", \"label\": \"%s\"}", indentStr, comp.ID, comp.Label)
				if j < len(cont.Components)-1 {
					fmt.Print(",")
				}
				fmt.Println()
			}
			fmt.Printf("%s      ]\n", indentStr)
			fmt.Printf("%s    }", indentStr)
			if i < len(sys.Containers)-1 {
				fmt.Print(",")
			}
			fmt.Println()
		}
		fmt.Printf("%s  ]\n", indentStr)
		fmt.Printf("%s}", indentStr)
		if indent == 0 {
			fmt.Println()
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
		fmt.Println(sysLine)

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
			fmt.Println(contLine)

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
				fmt.Println(compLine)
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
				fmt.Println(dsLine)
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
				fmt.Println(qLine)
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
				fmt.Println(compLine)
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
				fmt.Println(dsLine)
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
				fmt.Println(qLine)
			}
		}
	}
}

package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/sruja-ai/sruja/pkg/language"
)

func runTree(args []string, stdout, stderr io.Writer) int {
	treeCmd := flag.NewFlagSet("tree", flag.ContinueOnError)
	treeCmd.SetOutput(stderr)

	if err := treeCmd.Parse(args); err != nil {
		_, _ = fmt.Fprintf(stderr, "Error parsing tree flags: %v\n", err)
		return 1
	}

	if treeCmd.NArg() < 1 {
		_, _ = fmt.Fprintln(stderr, "Usage: sruja tree <file>")
		return 1
	}

	filePath := treeCmd.Arg(0)

	content, err := os.ReadFile(filepath.Clean(filePath))
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

	if program.Model == nil {
		_, _ = fmt.Fprintf(stderr, "Error: no model found in file\n")
		return 1
	}

	printTreeFromModel(stdout, program.Model, "")
	return 0
}

func printTreeFromModel(w io.Writer, model *language.Model, prefix string) {
	_, _ = fmt.Fprintf(w, "%sModel\n", prefix)
	var printElement func(elem *language.ElementDef, indent string)
	printElement = func(elem *language.ElementDef, indent string) {
		if elem == nil {
			return
		}
		id := elem.GetID()
		if id == "" {
			return
		}
		title := id
		titlePtr := elem.GetTitle()
		if titlePtr != nil {
			title = *titlePtr
		}
		_, _ = fmt.Fprintf(w, "%s%s (%s)\n", indent, title, id)
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					printElement(bodyItem.Element, indent+"  ")
				}
			}
		}
	}
	for _, item := range model.Items {
		if item.ElementDef != nil {
			printElement(item.ElementDef, prefix+"  ")
		}
	}
}

// Legacy printTree functions removed - Architecture struct removed (old syntax no longer supported)
// Use printTreeFromModel instead

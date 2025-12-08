package main

import (
	"flag"
	"fmt"
	"io"
	"os"

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

	program, _, err := p.Parse(filePath, string(content))
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "Parser Error: %v\n", err)
		return 1
	}

	printTree(stdout, program.Architecture, "")
	return 0
}

func printTree(w io.Writer, arch *language.Architecture, prefix string) {
	_, _ = fmt.Fprintf(w, "%s%s\n", prefix, arch.Name)
	//nolint:gocritic // Copying item struct is acceptable
	for _, item := range arch.Items {
		if item.System != nil {
			printSystemTree(w, item.System, prefix+"  ")
		}
	}
}

func printSystemTree(w io.Writer, sys *language.System, prefix string) {
	_, _ = fmt.Fprintf(w, "%s%s (%s)\n", prefix, sys.Label, sys.ID)
	for _, item := range sys.Items {
		if item.Container != nil {
			printContainerTree(w, item.Container, prefix+"  ")
		}
	}
}

func printContainerTree(w io.Writer, cont *language.Container, prefix string) {
	_, _ = fmt.Fprintf(w, "%s%s (%s)\n", prefix, cont.Label, cont.ID)
	//nolint:gocritic // Copying item struct is acceptable
	for _, item := range cont.Items {
		if item.Component != nil {
			_, _ = fmt.Fprintf(w, "%s  %s (%s)\n", prefix, item.Component.Label, item.Component.ID)
		}
	}
}

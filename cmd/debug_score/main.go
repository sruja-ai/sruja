package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: go run main.go <file>")
		os.Exit(1)
	}
	file := os.Args[1]
	cleanFile := filepath.Clean(file)
	if filepath.IsAbs(cleanFile) || cleanFile != file || strings.Contains(file, "..") {
		fmt.Println("Error: absolute paths and path traversal are not allowed")
		os.Exit(1)
	}
	data, err := os.ReadFile(file)
	if err != nil {
		panic(err)
	}

	p, err := language.NewParser()
	if err != nil {
		panic(err)
	}

	program, _, err := p.Parse(file, string(data))
	if err != nil {
		panic(err)
	}

	scorer := engine.NewScorer()
	card := scorer.CalculateScore(program)

	fmt.Printf("File: %s\n", file)
	fmt.Printf("Score: %d\n", card.Score)
	fmt.Printf("Grade: %s\n", card.Grade)
	fmt.Printf("Categories:\n")
	fmt.Printf("  Structural: %d\n", card.Categories.Structural)
	fmt.Printf("  Documentation: %d\n", card.Categories.Documentation)
	fmt.Printf("  Traceability: %d\n", card.Categories.Traceability)
	fmt.Printf("  Complexity: %d\n", card.Categories.Complexity)
	fmt.Printf("  Standardization: %d\n", card.Categories.Standardization)

	fmt.Println("\nDeductions:")
	for _, d := range card.Deductions {
		fmt.Printf("- [%s] %s (%d pts): %s\n", d.Category, d.Rule, d.Points, d.Message)
	}

	fmt.Println("\nDebug Traceability:")
	if program.Model != nil {
		for _, item := range program.Model.Items {
			if item.ElementDef != nil {
				elem := item.ElementDef
				fmt.Printf("Element %s: Kind=%s Tags=%v\n", elem.GetID(), elem.GetKind(), elem.GetTagRefs())
				if elem.GetBody() != nil {
					for _, sub := range elem.GetBody().Items {
						if sub.Element != nil {
							fmt.Printf("  SubElement %s: Kind=%s Tags=%v\n", sub.Element.GetID(), sub.Element.GetKind(), sub.Element.GetTagRefs())
							if sub.Element.GetBody() != nil {
								for _, sub2 := range sub.Element.GetBody().Items {
									if sub2.Element != nil {
										fmt.Printf("    SubSubElement %s: Kind=%s Tags=%v\n", sub2.Element.GetID(), sub2.Element.GetKind(), sub2.Element.GetTagRefs())
									}
								}
							}
						}
					}
				}
			}
		}
	}
}

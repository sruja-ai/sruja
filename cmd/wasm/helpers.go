// cmd/wasm/helpers.go
// Helper functions for WASM LSP operations
package main

import (
	"regexp"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// extractSymbols extracts all symbols from the architecture
func extractSymbols(arch *language.Architecture) []map[string]interface{} {
	estimatedSize := len(arch.Systems) + len(arch.Containers) + len(arch.Components) + len(arch.DataStores) + len(arch.Persons)
	symbols := make([]map[string]interface{}, 0, estimatedSize)

	for _, sys := range arch.Systems {
		symbols = append(symbols, map[string]interface{}{
			"name": sys.ID,
			"kind": "system",
			"line": sys.Pos.Line,
		})
	}

	for _, cont := range arch.Containers {
		symbols = append(symbols, map[string]interface{}{
			"name": cont.ID,
			"kind": "container",
			"line": cont.Pos.Line,
		})
	}

	for _, comp := range arch.Components {
		symbols = append(symbols, map[string]interface{}{
			"name": comp.ID,
			"kind": "component",
			"line": comp.Pos.Line,
		})
	}

	for _, ds := range arch.DataStores {
		symbols = append(symbols, map[string]interface{}{
			"name": ds.ID,
			"kind": "datastore",
			"line": ds.Pos.Line,
		})
	}

	for _, person := range arch.Persons {
		symbols = append(symbols, map[string]interface{}{
			"name": person.ID,
			"kind": "person",
			"line": person.Pos.Line,
		})
	}

	return symbols
}

// findHoverInfo finds hover information at the given position
func findHoverInfo(arch *language.Architecture, text string, line, column int) map[string]interface{} {
	lines := strings.Split(text, "\n")
	if line < 1 || line > len(lines) {
		return nil
	}

	lineText := lines[line-1]
	if column < 1 || column > len(lineText) {
		return nil
	}

	start := column - 1
	end := column - 1
	for start > 0 && isIdentChar(lineText[start-1]) {
		start--
	}
	for end < len(lineText) && isIdentChar(lineText[end]) {
		end++
	}
	if start >= end {
		return nil
	}

	word := lineText[start:end]

	for _, sys := range arch.Systems {
		if sys.ID == word {
			var contents strings.Builder
			contents.Grow(len(sys.ID) + len(sys.Label) + 20)
			contents.WriteString("System: ")
			contents.WriteString(sys.ID)
			if sys.Label != "" {
				contents.WriteString("\n")
				contents.WriteString(sys.Label)
			}
			return map[string]interface{}{
				"contents": contents.String(),
			}
		}
	}

	return nil
}

// isIdentChar checks if a character is a valid identifier character
func isIdentChar(c byte) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_'
}

// findDefinition finds the definition location for a symbol at the given position
func findDefinition(arch *language.Architecture, text string, line, column int) map[string]interface{} {
	lines := strings.Split(text, "\n")
	if line < 1 || line > len(lines) {
		return nil
	}

	lineText := lines[line-1]

	start := column - 1
	for start > 0 {
		prevChar := lineText[start-1]
		if isIdentChar(prevChar) || prevChar == '.' {
			start--
		} else {
			break
		}
	}

	end := column - 1
	for end < len(lineText) {
		currChar := lineText[end]
		if isIdentChar(currChar) || currChar == '.' {
			end++
		} else {
			break
		}
	}

	if start >= end {
		return nil
	}

	qualifiedName := lineText[start:end]
	parts := strings.Split(qualifiedName, ".")
	if len(parts) == 0 {
		return nil
	}

	if len(parts) == 1 {
		for _, sys := range arch.Systems {
			if sys.ID == parts[0] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   sys.Pos.Line,
					"column": sys.Pos.Column,
				}
			}
		}
		for _, cont := range arch.Containers {
			if cont.ID == parts[0] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   cont.Pos.Line,
					"column": cont.Pos.Column,
				}
			}
		}
		for _, comp := range arch.Components {
			if comp.ID == parts[0] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   comp.Pos.Line,
					"column": comp.Pos.Column,
				}
			}
		}
		for _, ds := range arch.DataStores {
			if ds.ID == parts[0] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   ds.Pos.Line,
					"column": ds.Pos.Column,
				}
			}
		}
		return nil
	}

	var currentSystem *language.System
	for _, sys := range arch.Systems {
		if sys.ID == parts[0] {
			currentSystem = sys
			break
		}
	}
	if currentSystem == nil {
		return nil
	}

	if len(parts) == 2 {
		for _, cont := range currentSystem.Containers {
			if cont.ID == parts[1] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   cont.Pos.Line,
					"column": cont.Pos.Column,
				}
			}
		}
		for _, comp := range currentSystem.Components {
			if comp.ID == parts[1] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   comp.Pos.Line,
					"column": comp.Pos.Column,
				}
			}
		}
		for _, ds := range currentSystem.DataStores {
			if ds.ID == parts[1] {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   ds.Pos.Line,
					"column": ds.Pos.Column,
				}
			}
		}
		return nil
	}

	if len(parts) >= 3 {
		var currentContainer *language.Container
		for _, cont := range currentSystem.Containers {
			if cont.ID == parts[1] {
				currentContainer = cont
				break
			}
		}
		if currentContainer == nil {
			return nil
		}

		childID := parts[2]
		for _, comp := range currentContainer.Components {
			if comp.ID == childID {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   comp.Pos.Line,
					"column": comp.Pos.Column,
				}
			}
		}
		for _, ds := range currentContainer.DataStores {
			if ds.ID == childID {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   ds.Pos.Line,
					"column": ds.Pos.Column,
				}
			}
		}
		for _, q := range currentContainer.Queues {
			if q.ID == childID {
				return map[string]interface{}{
					"file":   "input.sruja",
					"line":   q.Pos.Line,
					"column": q.Pos.Column,
				}
			}
		}
	}

	return nil
}

// findSymbolReferences finds all references to a symbol in the architecture
func findSymbolReferences(arch *language.Architecture, text string, symbolName string) []map[string]interface{} {
	references := []map[string]interface{}{}
	lines := strings.Split(text, "\n")

	matchesSymbol := func(qname string) bool {
		if qname == symbolName {
			return true
		}
		if strings.HasPrefix(qname, symbolName+".") {
			return true
		}
		if strings.HasSuffix(qname, "."+symbolName) {
			return true
		}
		parts := strings.Split(qname, ".")
		for _, part := range parts {
			if part == symbolName {
				return true
			}
		}
		return false
	}

	findInRelations := func(rels []*language.Relation) {
		for _, rel := range rels {
			fromStr := rel.From.String()
			toStr := rel.To.String()

			if matchesSymbol(fromStr) {
				loc := rel.Location()
				lineNum := findLineForRelation(lines, fromStr, loc.Line)
				if lineNum > 0 {
					colNum := findColumnForSymbol(lines[lineNum-1], fromStr)
					if colNum > 0 {
						references = append(references, map[string]interface{}{
							"line":   lineNum,
							"column": colNum,
						})
					}
				}
			}

			if matchesSymbol(toStr) {
				loc := rel.Location()
				lineNum := findLineForRelation(lines, toStr, loc.Line)
				if lineNum > 0 {
					colNum := findColumnForSymbol(lines[lineNum-1], toStr)
					if colNum > 0 {
						references = append(references, map[string]interface{}{
							"line":   lineNum,
							"column": colNum,
						})
					}
				}
			}
		}
	}

	findInRelations(arch.Relations)

	for _, sys := range arch.Systems {
		findInRelations(sys.Relations)
		for _, cont := range sys.Containers {
			findInRelations(cont.Relations)
		}
	}

	return references
}

// findLineForRelation finds the line number for a relation symbol
func findLineForRelation(lines []string, symbol string, hintLine int) int {
	for i := hintLine - 1; i < len(lines); i++ {
		if strings.Contains(lines[i], symbol) {
			return i + 1
		}
	}
	for i := hintLine - 2; i >= 0; i-- {
		if strings.Contains(lines[i], symbol) {
			return i + 1
		}
	}
	return 0
}

// findColumnForSymbol finds the column number for a symbol in a line
func findColumnForSymbol(lineText string, symbol string) int {
	idx := strings.Index(lineText, symbol)
	if idx >= 0 {
		return idx + 1
	}
	return 0
}

// performRename performs the actual rename operation
func performRename(input, oldName, newName string) string {
	result := input

	wordBoundaryPattern := `\b` + regexp.QuoteMeta(oldName) + `\b`
	result = regexp.MustCompile(wordBoundaryPattern).ReplaceAllString(result, newName)

	qualifiedAfterDot := `\.` + regexp.QuoteMeta(oldName) + `(\s|->|"|\.|{|}|,|\[|\]|$)`
	result = regexp.MustCompile(qualifiedAfterDot).ReplaceAllStringFunc(result, func(match string) string {
		return "." + newName + match[len(oldName)+1:]
	})

	qualifiedBeforeArrow := regexp.QuoteMeta(oldName) + `\s*->`
	result = regexp.MustCompile(qualifiedBeforeArrow).ReplaceAllString(result, newName+" ->")

	return result
}

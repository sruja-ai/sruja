//go:build js && wasm

package main

import (
	"encoding/json"
	"fmt"
	"strings"
	"syscall/js"

	"github.com/sruja-ai/sruja/pkg/export/likec4"
	"github.com/sruja-ai/sruja/pkg/language"
)

// format formats the DSL text
func format(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 1 {
		return lspResult(false, nil, "invalid arguments")
	}
	input := args[0].String()

	_, program, err := parseToWorkspace(input, "input.sruja")
	if err != nil || program == nil {
		return lspResult(true, input, "")
	}

	// Printer works directly with Program AST (no conversion needed)
	// Use LikeC4 DSL exporter for formatting
	dslExporter := likec4.NewDSLExporter()
	formatted := dslExporter.ExportDSL(program)
	ret = lspResult(true, formatted, "")
	return
}

// semanticTokens returns semantic tokens for syntax highlighting
func semanticTokens(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 1 {
		return lspResult(false, nil, "invalid arguments")
	}
	input := args[0].String()

	lines := strings.Split(input, "\n")
	data := []uint32{}
	ti := map[string]uint32{
		"keyword": 0, "class": 1, "module": 2, "function": 3, "struct": 4, "enum": 5, "variable": 6, "operator": 7, "string": 8,
	}

	keywords := []string{
		"specification", "model", "views", "view",
		"element", "system", "component", "container", "datastore", "queue", "person", "workspace",
		"relationship", "extend", "include", "exclude",
		"metadata", "properties", "style",
		"requirement", "adr", "policy", "domain",
		"library", "owner", "import", "from",
		"title", "description", "technology", "tech", "link", "icon", "shape", "color",
		"status", "context", "decision", "consequences",
	}

	prevLine := uint32(0)
	for li, line := range lines {
		// Strings
		start := -1
		for i := 0; i < len(line); i++ {
			if line[i] == '"' {
				if start == -1 {
					start = i
				} else {
					deltaLine := uint32(li) - prevLine
					deltaStart := uint32(start)
					if deltaLine == 0 {
						deltaStart = uint32(start)
					}
					prevLine = uint32(li)
					data = append(data, deltaLine, deltaStart, uint32(i-start+1), ti["string"], 0)
					start = -1
				}
			}
		}

		// Arrow operator
		if idx := strings.Index(line, "->"); idx >= 0 {
			deltaLine := uint32(li) - prevLine
			deltaStart := uint32(idx)
			if deltaLine == 0 {
				deltaStart = uint32(idx)
			}
			prevLine = uint32(li)
			data = append(data, deltaLine, deltaStart, 2, ti["operator"], 0)
		}

		// Keywords
		for _, k := range keywords {
			if idx := strings.Index(line, k); idx >= 0 {
				if isBoundary(line, idx, len(k)) {
					deltaLine := uint32(li) - prevLine
					deltaStart := uint32(idx)
					if deltaLine == 0 {
						deltaStart = uint32(idx)
					}
					prevLine = uint32(li)
					data = append(data, deltaLine, deltaStart, uint32(len(k)), ti["keyword"], 0)
				}
			}
		}

		// Element declarations
		markDeclID(&data, uint32(li), line, "system ", ti["class"], &prevLine)
		markDeclID(&data, uint32(li), line, "container ", ti["module"], &prevLine)
		markDeclID(&data, uint32(li), line, "component ", ti["function"], &prevLine)
		markDeclID(&data, uint32(li), line, "datastore ", ti["struct"], &prevLine)
		markDeclID(&data, uint32(li), line, "queue ", ti["enum"], &prevLine)
		markDeclID(&data, uint32(li), line, "person ", ti["variable"], &prevLine)
	}

	jsonBytes, _ := json.Marshal(data)
	ret = lspResult(true, string(jsonBytes), "")
	return
}

// documentLinks returns document links for file references
func documentLinks(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 1 {
		return lspResult(false, nil, "invalid arguments")
	}
	input := args[0].String()

	links := []map[string]interface{}{}
	lines := strings.Split(input, "\n")

	for i, line := range lines {
		// Check for file:// URLs
		if idx := strings.Index(line, "file://"); idx >= 0 {
			endIdx := idx + 7
			for endIdx < len(line) && !strings.ContainsAny(line[endIdx:endIdx+1], " \t\n\r\"'`") {
				endIdx++
			}
			if endIdx > idx+7 {
				filePath := line[idx+7 : endIdx]
				links = append(links, map[string]interface{}{
					"range": map[string]interface{}{
						"start": map[string]interface{}{"line": i, "character": idx},
						"end":   map[string]interface{}{"line": i, "character": endIdx},
					},
					"target": "file://" + filePath,
				})
			}
		}

		// Check for .sruja file references
		if idx := strings.Index(line, ".sruja"); idx >= 0 {
			startIdx := idx
			for startIdx > 0 && !strings.ContainsAny(string(line[startIdx-1]), " \t\n\r\"'`") {
				startIdx--
			}
			endIdx := idx + 6
			for endIdx < len(line) && !strings.ContainsAny(line[endIdx:endIdx+1], " \t\n\r\"'`") {
				endIdx++
			}
			if startIdx < idx && (strings.Contains(line[startIdx:endIdx], "/") || strings.Contains(line[startIdx:endIdx], "\\")) {
				links = append(links, map[string]interface{}{
					"range": map[string]interface{}{
						"start": map[string]interface{}{"line": i, "character": startIdx},
						"end":   map[string]interface{}{"line": i, "character": endIdx},
					},
					"target": "file://" + line[startIdx:endIdx],
				})
			}
		}
	}

	jsonBytes, _ := json.Marshal(links)
	ret = lspResult(true, string(jsonBytes), "")
	return
}

// foldingRanges returns folding ranges for code blocks
func foldingRanges(this js.Value, args []js.Value) (ret interface{}) {
	defer func() {
		if r := recover(); r != nil {
			ret = lspResult(false, nil, fmt.Sprint(r))
		}
	}()

	if len(args) < 1 {
		return lspResult(false, nil, "invalid arguments")
	}
	input := args[0].String()

	_, program, err := parseToWorkspace(input, "input.sruja")
	ranges := []map[string]interface{}{}

	if err == nil && program != nil && program.Model != nil {
		// Add folding ranges from AST
		var addElementRanges func(elem *language.LikeC4ElementDef)
		addElementRanges = func(elem *language.LikeC4ElementDef) {
			if elem == nil {
				return
			}
			loc := elem.Location()
			body := elem.GetBody()
			if loc.Line > 0 && body != nil {
				endLine := findBlockEnd(input, loc.Line-1, loc.Column-1)
				if endLine > loc.Line {
					startCol := uint32(loc.Column - 1)
					kind := "region"
					ranges = append(ranges, map[string]interface{}{
						"startLine":      uint32(loc.Line - 1),
						"startCharacter": startCol,
						"endLine":        uint32(endLine - 1),
						"kind":           kind,
					})
				}
				for _, bodyItem := range body.Items {
					if bodyItem.Element != nil {
						addElementRanges(bodyItem.Element)
					}
				}
			}
		}
		for _, item := range program.Model.Items {
			if item.ElementDef != nil {
				addElementRanges(item.ElementDef)
			}
		}
	} else {
		// Fallback to text-based folding
		ranges = foldingRangesFromText(input)
	}

	jsonBytes, _ := json.Marshal(ranges)
	ret = lspResult(true, string(jsonBytes), "")
	return
}

// Helper functions for semantic tokens and folding
func isBoundary(line string, start int, length int) bool {
	before := start - 1
	after := start + length
	if before >= 0 && before < len(line) {
		if isIdentChar(line[before]) {
			return false
		}
	}
	if after >= 0 && after < len(line) {
		if isIdentChar(line[after]) {
			return false
		}
	}
	return true
}

func markDeclID(data *[]uint32, line uint32, s string, prefix string, ttype uint32, prevLine *uint32) {
	if !strings.HasPrefix(strings.TrimSpace(s), prefix) {
		return
	}
	idx := strings.Index(s, prefix)
	if idx < 0 {
		return
	}
	rest := strings.TrimSpace(s[idx+len(prefix):])
	if rest == "" {
		return
	}
	end := 0
	for end < len(rest) {
		c := rest[end]
		if c == ' ' || c == '"' || c == '{' {
			break
		}
		end++
	}
	col := idx + len(prefix)
	deltaLine := line - *prevLine
	deltaStart := uint32(col)
	if deltaLine == 0 {
		deltaStart = uint32(col)
	}
	*prevLine = line
	*data = append(*data, deltaLine, deltaStart, uint32(end), ttype, 1)
}

func findBlockEnd(text string, startLine, startCol int) int {
	lines := strings.Split(text, "\n")
	if startLine >= len(lines) {
		return 0
	}
	braceCol := -1
	searchLine := startLine
	for searchLine < len(lines) {
		line := lines[searchLine]
		if searchLine == startLine {
			if idx := strings.Index(line[startCol:], "{"); idx >= 0 {
				braceCol = startCol + idx
				break
			}
		} else {
			if idx := strings.Index(line, "{"); idx >= 0 {
				braceCol = idx
				break
			}
		}
		searchLine++
	}
	if braceCol < 0 {
		return 0
	}
	depth := 0
	for i := searchLine; i < len(lines); i++ {
		line := lines[i]
		startIdx := 0
		if i == searchLine {
			startIdx = braceCol + 1
		}
		for j := startIdx; j < len(line); j++ {
			char := line[j]
			if char == '{' {
				depth++
			} else if char == '}' {
				depth--
				if depth == 0 {
					return i + 1
				}
			}
		}
	}
	return 0
}

func foldingRangesFromText(text string) []map[string]interface{} {
	lines := strings.Split(text, "\n")
	ranges := []map[string]interface{}{}
	openBraces := []struct {
		line   int
		column int
	}{}
	for i, line := range lines {
		for j, char := range line {
			if char == '{' {
				openBraces = append(openBraces, struct {
					line   int
					column int
				}{line: i, column: j})
			} else if char == '}' && len(openBraces) > 0 {
				open := openBraces[len(openBraces)-1]
				openBraces = openBraces[:len(openBraces)-1]
				if i > open.line {
					col := uint32(open.column)
					kind := "region"
					ranges = append(ranges, map[string]interface{}{
						"startLine":      uint32(open.line),
						"startCharacter": col,
						"endLine":        uint32(i),
						"kind":           kind,
					})
				}
			}
		}
	}
	return ranges
}

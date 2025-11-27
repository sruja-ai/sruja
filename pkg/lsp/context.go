// pkg/lsp/context.go
package lsp

import (
	"strings"
)

// CompletionContext represents the context where autocomplete is being requested.
//
// The context determines what types of suggestions should be provided.
type CompletionContext struct {
	// Position information
	Line      int
	Character int
	
	// Document text (lines)
	Lines []string
	
	// Current line text
	CurrentLine string
	
	// Text before cursor on current line
	BeforeCursor string
	
	// Text after cursor on current line
	AfterCursor string
	
	// Context type
	ContextType ContextType
	
	// Additional context-specific data
	ParentElement string // If inside an element block (e.g., "system", "container")
	Scope         string // Current scope: "system", "container", "component", "architecture"
	InMetadata    bool   // Whether cursor is inside a metadata block
	MetadataKey   string // If in metadata, the key being typed
	InRelation    bool   // Whether cursor is in a relation (after ->)
	ImportAlias   string // If after import alias (e.g., "Billing.")
	KeywordPrefix string // Partial keyword being typed
}

// ContextType indicates what kind of completion context we're in.
type ContextType int

const (
	ContextUnknown ContextType = iota
	ContextTopLevel            // At architecture top level
	ContextKeyword             // Typing a keyword (system, container, etc.)
	ContextElementID           // Typing an element ID
	ContextElementBlock        // Inside an element block (system { ... })
	ContextMetadata            // Inside metadata { ... }
	ContextMetadataKey         // Typing a metadata key
	ContextMetadataValue       // Typing a metadata value
	ContextRelation            // In a relation (element -> element)
	ContextQualifiedReference  // Typing a qualified reference (Billing.API)
	ContextAttribute           // Typing an attribute (technology, tags, etc.)
	ContextImport              // In an import statement
	ContextRequirement         // In a requirement statement
	ContextADR                 // In an ADR statement
	ContextJourney             // In a journey statement
	ContextJourneyStep         // In a journey step
)

// AnalyzeContext analyzes the document text and cursor position to determine completion context.
func AnalyzeContext(lines []string, line, character int) CompletionContext {
	if line < 0 || line >= len(lines) {
		return CompletionContext{
			Lines:        lines,
			Line:         line,
			Character:    character,
			ContextType:  ContextUnknown,
			CurrentLine:  "",
			BeforeCursor: "",
			AfterCursor:  "",
		}
	}
	
	currentLine := lines[line]
	beforeCursor := currentLine[:min(character, len(currentLine))]
	afterCursor := currentLine[min(character, len(currentLine)):]
	
	ctx := CompletionContext{
		Lines:        lines,
		Line:         line,
		Character:    character,
		CurrentLine:  currentLine,
		BeforeCursor: beforeCursor,
		AfterCursor:  afterCursor,
	}
	
	// Analyze context
	ctx.detectContext()
	
	return ctx
}

// detectContext determines the completion context type based on cursor position.
func (ctx *CompletionContext) detectContext() {
	before := strings.TrimSpace(ctx.BeforeCursor)
	
	// Check if we're in a metadata block
	if strings.Contains(before, "metadata") && strings.Contains(before, "{") {
		// Check if we're after a colon (typing value)
		if strings.Contains(before, ":") && !strings.HasSuffix(before, ":") {
			ctx.ContextType = ContextMetadataValue
			ctx.InMetadata = true
			// Extract metadata key
			parts := strings.Split(before, ":")
			if len(parts) > 0 {
				lastPart := strings.TrimSpace(parts[len(parts)-1])
				if lastPart != "" {
					keyPart := strings.TrimSpace(parts[len(parts)-2])
					if idx := strings.LastIndex(keyPart, " "); idx >= 0 {
						ctx.MetadataKey = strings.TrimSpace(keyPart[idx:])
					} else {
						ctx.MetadataKey = keyPart
					}
				}
			}
			return
		}
		
		// Typing metadata key
		if !strings.Contains(before, ":") || strings.HasSuffix(before, ":") {
			ctx.ContextType = ContextMetadataKey
			ctx.InMetadata = true
			return
		}
	}
	
	// Check if we're in a relation (after ->)
	if strings.Contains(before, "->") {
		ctx.ContextType = ContextRelation
		ctx.InRelation = true
		return
	}
	
	// Check if we're typing a qualified reference (ImportAlias.)
	if idx := strings.LastIndex(before, "."); idx >= 0 {
		beforeDot := strings.TrimSpace(before[:idx])
		// Check if this looks like an import alias
		if isLikelyAlias(beforeDot) {
			ctx.ContextType = ContextQualifiedReference
			ctx.ImportAlias = beforeDot
			return
		}
	}
	
	// Check if we're inside an element block
	parent := findParentElement(ctx.Lines, ctx.Line)
	if parent != "" {
		ctx.ParentElement = parent
		ctx.Scope = parent
		ctx.ContextType = ContextElementBlock
		
		// Check if we're typing an attribute
		if strings.Contains(before, " ") && !strings.Contains(before, "{") {
			lastWord := lastWord(before)
			ctx.ContextType = ContextAttribute
			ctx.KeywordPrefix = lastWord
			return
		}
		return
	}
	
	// Check if we're typing a keyword
	if isTypingKeyword(before) {
		ctx.ContextType = ContextKeyword
		ctx.KeywordPrefix = lastWord(before)
		return
	}
	
	// Check if we're at top level (architecture level)
	if ctx.isAtTopLevel() {
		ctx.ContextType = ContextTopLevel
		return
	}
	
	// Default: typing an element ID
	ctx.ContextType = ContextElementID
}

// isAtTopLevel checks if cursor is at architecture top level.
func (ctx *CompletionContext) isAtTopLevel() bool {
	// Count braces to see if we're at top level
	depth := 0
	for i := 0; i < ctx.Line; i++ {
		line := ctx.Lines[i]
		for _, ch := range line {
			if ch == '{' {
				depth++
			} else if ch == '}' {
				depth--
			}
		}
	}
	
	// Count braces on current line before cursor
	for _, ch := range ctx.BeforeCursor {
		if ch == '{' {
			depth++
		} else if ch == '}' {
			depth--
		}
	}
	
	return depth <= 1 // 1 = inside architecture block, 0 = outside
}

// findParentElement finds the parent element type (system, container, component).
func findParentElement(lines []string, currentLine int) string {
	depth := 0
	targetDepth := -1
	
	for i := currentLine; i >= 0; i-- {
		line := strings.TrimSpace(lines[i])
		
		// Count braces
		for _, ch := range line {
			if ch == '{' {
				depth--
				if depth == 1 && targetDepth == -1 {
					targetDepth = i
				}
			} else if ch == '}' {
				depth++
			}
		}
		
		// Check for element keywords
		if strings.HasPrefix(line, "system ") || strings.HasPrefix(line, "system\t") {
			return "system"
		}
		if strings.HasPrefix(line, "container ") || strings.HasPrefix(line, "container\t") {
			return "container"
		}
		if strings.HasPrefix(line, "component ") || strings.HasPrefix(line, "component\t") {
			return "component"
		}
	}
	
	return ""
}

// isTypingKeyword checks if the user is typing a keyword.
func isTypingKeyword(text string) bool {
	text = strings.TrimSpace(text)
	if text == "" {
		return true // Start of line
	}
	
	// Check if last word is a partial keyword
	keywords := []string{"system", "container", "component", "person", "datastore", "queue",
		"import", "journey", "requirement", "adr", "shared", "library"}
	
	last := lastWord(text)
	for _, kw := range keywords {
		if strings.HasPrefix(kw, strings.ToLower(last)) {
			return true
		}
	}
	
	return false
}

// lastWord returns the last word in a string.
func lastWord(s string) string {
	s = strings.TrimSpace(s)
	if s == "" {
		return ""
	}
	
	parts := strings.Fields(s)
	if len(parts) == 0 {
		return ""
	}
	return parts[len(parts)-1]
}

// isLikelyAlias checks if a string looks like an import alias.
func isLikelyAlias(s string) bool {
	// Simple heuristic: if it's a single word starting with capital letter
	s = strings.TrimSpace(s)
	if s == "" {
		return false
	}
	
	parts := strings.Fields(s)
	if len(parts) != 1 {
		return false
	}
	
	first := parts[0]
	if len(first) == 0 {
		return false
	}
	
	// Check if starts with capital letter (common for aliases)
	return first[0] >= 'A' && first[0] <= 'Z'
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}


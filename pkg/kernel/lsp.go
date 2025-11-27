// pkg/kernel/lsp.go
// Package kernel provides LSP-like features for IDE integration.
package kernel

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/model"
)

// CompletionItem represents an autocomplete suggestion.
type CompletionItem struct {
	Label         string `json:"label"`
	Kind          string `json:"kind"`                    // "system", "component", etc.
	Detail        string `json:"detail"`                  // Additional information
	Documentation string `json:"documentation,omitempty"` // Help text
}

// HoverInfo represents information shown when hovering over a symbol.
type HoverInfo struct {
	Content    string         `json:"content"`  // Markdown content
	Location   model.Location `json:"location"` // Definition location
	SymbolKind SymbolKind     `json:"symbolKind"`
}

// GetCompletions returns autocomplete suggestions for a given prefix.
func (k *Kernel) GetCompletions(prefix string, position int) []CompletionItem {
	var completions []CompletionItem

	// Get all symbols
	symbols := k.symbols.GetAllSymbols()

	// Filter by prefix
	for id, entry := range symbols {
		if len(id) >= len(prefix) && id[:len(prefix)] == prefix {
			completions = append(completions, CompletionItem{
				Label:  id,
				Kind:   string(entry.Kind),
				Detail: entry.Name,
			})
		}
	}

	// Also add DSL keywords
	keywords := []string{
		"system", "container", "component", "entity", "event",
		"requirement", "adr", "domain", "architecture",
	}
	for _, keyword := range keywords {
		if len(keyword) >= len(prefix) && keyword[:len(prefix)] == prefix {
			completions = append(completions, CompletionItem{
				Label: keyword,
				Kind:  "keyword",
			})
		}
	}

	return completions
}

// GetHover returns hover information for a symbol at a given position.
func (k *Kernel) GetHover(symbolID string) (*HoverInfo, bool) {
	entry, ok := k.symbols.GetSymbol(symbolID)
	if !ok {
		return nil, false
	}

	content := fmt.Sprintf("**%s** (%s)\n\n%s",
		entry.Name,
		entry.Kind,
		formatSymbolInfo(entry),
	)

	return &HoverInfo{
		Content:    content,
		Location:   entry.Location,
		SymbolKind: entry.Kind,
	}, true
}

// GetDefinition returns the definition location for a symbol.
func (k *Kernel) GetDefinition(symbolID string) (*model.Location, bool) {
	entry, ok := k.symbols.GetSymbol(symbolID)
	if !ok {
		return nil, false
	}

	loc := entry.Location
	return &loc, true
}

// GetReferences returns all references to a symbol.
func (k *Kernel) GetReferences(symbolID string) []SymbolReference {
	entry, ok := k.symbols.GetSymbol(symbolID)
	if !ok {
		return nil
	}

	return entry.References
}

// formatSymbolInfo formats symbol information for hover display.
func formatSymbolInfo(entry *SymbolEntry) string {
	info := fmt.Sprintf("Defined in: %s:%d:%d",
		entry.Location.File,
		entry.Location.Line,
		entry.Location.Column,
	)

	if len(entry.References) > 0 {
		info += fmt.Sprintf("\n\nReferenced %d time(s)", len(entry.References))
	}

	return info
}

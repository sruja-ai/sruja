// pkg/lsp/completion.go
package lsp

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// CompletionProvider provides intelligent autocomplete for the Sruja DSL.
//
// It uses:
// - SemanticIndex: for element references
// - MetadataRegistry: for metadata key suggestions
// - CompletionContext: to understand where the user is typing
// - Plugins: for plugin-contributed completions
type CompletionProvider struct {
	index    *SemanticIndex
	registry *MetadataRegistry
	plugins  []CompletionPlugin
}

// NewCompletionProvider creates a new completion provider.
func NewCompletionProvider(index *SemanticIndex, registry *MetadataRegistry) *CompletionProvider {
	return &CompletionProvider{
		index:    index,
		registry: registry,
		plugins:  []CompletionPlugin{},
	}
}

// RegisterPlugin registers a plugin that can contribute completions.
func (cp *CompletionProvider) RegisterPlugin(plugin CompletionPlugin) {
	cp.plugins = append(cp.plugins, plugin)
}

// ProvideCompletions returns LSP completion items based on context.
func (cp *CompletionProvider) ProvideCompletions(
	filePath string,
	text string,
	line, character int,
) ([]map[string]any, error) {
	lines := strings.Split(text, "\n")
	if line < 0 || line >= len(lines) {
		return nil, nil
	}

	ctx := AnalyzeContext(lines, line, character)

	var items []CompletionItem

	switch ctx.ContextType {
	case ContextKeyword:
		items = cp.provideKeywordCompletions(ctx)
	case ContextElementID:
		items = cp.provideElementIDCompletions(ctx, filePath)
	case ContextElementBlock:
		items = cp.provideElementBlockCompletions(ctx, filePath)
	case ContextMetadataKey:
		items = cp.provideMetadataKeyCompletions(ctx)
	case ContextMetadataValue:
		items = cp.provideMetadataValueCompletions(ctx)
	case ContextRelation:
		items = cp.provideRelationCompletions(ctx, filePath)
	case ContextQualifiedReference:
		items = cp.provideQualifiedReferenceCompletions(ctx, filePath)
	case ContextAttribute:
		items = cp.provideAttributeCompletions(ctx)
	case ContextTopLevel:
		items = cp.provideTopLevelCompletions(ctx)
	default:
		// Fallback: provide keywords and element IDs
		items = append(items, cp.provideKeywordCompletions(ctx)...)
		items = append(items, cp.provideElementIDCompletions(ctx, filePath)...)
	}

	// Add plugin-contributed completions
	for _, plugin := range cp.plugins {
		pluginItems := plugin.ProvideCompletions(ctx)
		items = append(items, pluginItems...)
	}

	// Convert to LSP format
	result := make([]map[string]any, len(items))
	for i, item := range items {
		result[i] = map[string]any{
			"label":         item.Label,
			"kind":          item.Kind,
			"detail":        item.Detail,
			"documentation": item.Documentation,
			"insertText":    item.InsertText,
		}
	}

	return result, nil
}

// provideKeywordCompletions provides keyword completions.
func (cp *CompletionProvider) provideKeywordCompletions(ctx CompletionContext) []CompletionItem {
	keywords := []struct {
		label      string
		kind       int
		insertText string
		detail     string
	}{
		{"system", 14, "system ${1:ID} \"${2:Label}\" {", "Define a system"},
		{"container", 14, "container ${1:ID} \"${2:Label}\" {", "Define a container"},
		{"component", 14, "component ${1:ID} \"${2:Label}\" {", "Define a component"},
		{"person", 14, "person ${1:ID} \"${2:Label}\"", "Define a person"},
		{"datastore", 14, "datastore ${1:ID} \"${2:Label}\"", "Define a datastore"},
		{"queue", 14, "queue ${1:ID} \"${2:Label}\"", "Define a queue"},
		{"import", 14, "import \"${1:path}\" as ${2:Alias}", "Import an architecture"},
		{"journey", 14, "journey ${1:ID} \"${2:Label}\" {", "Define a journey"},
		{"requirement", 14, "requirement ${1:ID} \"${2:Label}\"", "Define a requirement"},
		{"adr", 14, "adr ${1:ID} \"${2:Title}\"", "Define an ADR"},
		{"shared", 14, "shared ${1:ID} \"${2:Label}\"", "Define a shared artifact"},
		{"library", 14, "library ${1:ID} \"${2:Label}\"", "Define a library"},
	}

	var items []CompletionItem
	prefix := strings.ToLower(ctx.KeywordPrefix)

	for _, kw := range keywords {
		if prefix == "" || strings.HasPrefix(kw.label, prefix) {
			items = append(items, CompletionItem{
				Label:         kw.label,
				Kind:          kw.kind,
				Detail:        kw.detail,
				InsertText:    kw.insertText,
				Documentation: kw.detail,
			})
		}
	}

	return items
}

// provideElementIDCompletions provides element ID completions.
func (cp *CompletionProvider) provideElementIDCompletions(ctx CompletionContext, filePath string) []CompletionItem {
	// Get prefix from before cursor
	prefix := lastWord(ctx.BeforeCursor)

	// Get matching elements from semantic index
	elements := cp.index.GetByPrefix(prefix)

	var items []CompletionItem
	for _, elem := range elements {
		// Only suggest elements from current file or visible imports
		if elem.File == filePath || cp.isVisibleImport(elem, filePath) {
			items = append(items, elem.ToCompletionItem())
		}
	}

	return items
}

// provideElementBlockCompletions provides completions inside an element block.
func (cp *CompletionProvider) provideElementBlockCompletions(ctx CompletionContext, filePath string) []CompletionItem {
	var items []CompletionItem

	// Suggest child elements based on parent type
	switch ctx.Scope {
	case "system":
		items = append(items, CompletionItem{
			Label:      "container",
			Kind:       14,
			Detail:     "Define a container",
			InsertText: "container ${1:ID} \"${2:Label}\" {",
		})
		items = append(items, CompletionItem{
			Label:      "component",
			Kind:       14,
			Detail:     "Define a component",
			InsertText: "component ${1:ID} \"${2:Label}\" {",
		})
		items = append(items, CompletionItem{
			Label:      "datastore",
			Kind:       14,
			Detail:     "Define a datastore",
			InsertText: "datastore ${1:ID} \"${2:Label}\"",
		})
		items = append(items, CompletionItem{
			Label:      "queue",
			Kind:       14,
			Detail:     "Define a queue",
			InsertText: "queue ${1:ID} \"${2:Label}\"",
		})
		items = append(items, CompletionItem{
			Label:      "metadata",
			Kind:       14,
			Detail:     "Add metadata",
			InsertText: "metadata {\n  ",
		})

	case "container":
		items = append(items, CompletionItem{
			Label:      "component",
			Kind:       14,
			Detail:     "Define a component",
			InsertText: "component ${1:ID} \"${2:Label}\" {",
		})
		items = append(items, CompletionItem{
			Label:      "metadata",
			Kind:       14,
			Detail:     "Add metadata",
			InsertText: "metadata {\n  ",
		})
	}

	return items
}

// provideMetadataKeyCompletions provides metadata key completions.
func (cp *CompletionProvider) provideMetadataKeyCompletions(ctx CompletionContext) []CompletionItem {
	scope := ctx.Scope
	if scope == "" {
		scope = "system" // Default scope
	}

	descriptors := cp.registry.ForScope(scope)

	var items []CompletionItem
	prefix := strings.ToLower(lastWord(ctx.BeforeCursor))

	for _, desc := range descriptors {
		if prefix == "" || strings.HasPrefix(strings.ToLower(desc.Key), prefix) {
			items = append(items, desc.ToCompletionItem())
		}
	}

	// Also check for plugin-contributed metadata
	for _, plugin := range cp.plugins {
		pluginDescs := plugin.MetadataDescriptors(scope)
		for _, desc := range pluginDescs {
			if prefix == "" || strings.HasPrefix(strings.ToLower(desc.Key), prefix) {
				items = append(items, desc.ToCompletionItem())
			}
		}
	}

	return items
}

// provideMetadataValueCompletions provides metadata value completions.
func (cp *CompletionProvider) provideMetadataValueCompletions(ctx CompletionContext) []CompletionItem {
	if ctx.MetadataKey == "" {
		return nil
	}

	desc, ok := cp.registry.Get(ctx.MetadataKey)
	if !ok {
		// Check plugins
		for _, plugin := range cp.plugins {
			desc, ok = plugin.GetMetadataDescriptor(ctx.MetadataKey)
			if ok {
				break
			}
		}
		if !ok {
			return nil
		}
	}

	var items []CompletionItem

	// If enum values are defined, suggest them
	if len(desc.Enum) > 0 {
		for _, val := range desc.Enum {
			items = append(items, CompletionItem{
				Label:      val,
				Kind:       1, // Text
				InsertText: "\"" + val + "\"",
			})
		}
	}

	return items
}

// provideRelationCompletions provides element ID completions for relations.
func (cp *CompletionProvider) provideRelationCompletions(ctx CompletionContext, filePath string) []CompletionItem {
	// Same as element ID completions, but filtered by what makes sense in a relation
	elements := cp.index.GetByPrefix("")

	var items []CompletionItem
	for _, elem := range elements {
		// Relations typically point to systems, containers, components, persons, datastores, queues
		if elem.Type == "system" || elem.Type == "container" || elem.Type == "component" ||
			elem.Type == "person" || elem.Type == "datastore" || elem.Type == "queue" {
			if elem.File == filePath || cp.isVisibleImport(elem, filePath) {
				items = append(items, elem.ToCompletionItem())
			}
		}
	}

	return items
}

// provideQualifiedReferenceCompletions provides completions for qualified references.
func (cp *CompletionProvider) provideQualifiedReferenceCompletions(ctx CompletionContext, filePath string) []CompletionItem {
	if ctx.ImportAlias == "" {
		return nil
	}

	// Get elements from the imported architecture
	elements := cp.index.GetQualifiedElements(ctx.ImportAlias + ".")

	var items []CompletionItem
	for _, elem := range elements {
		items = append(items, elem.ToCompletionItem())
	}

	return items
}

// provideAttributeCompletions provides attribute completions.
func (cp *CompletionProvider) provideAttributeCompletions(ctx CompletionContext) []CompletionItem {
	var items []CompletionItem

	// Suggest common attributes
	attributes := []struct {
		label      string
		insertText string
		detail     string
	}{
		{"technology", "technology \"${1:Technology}\"", "Specify technology stack"},
		{"tags", "tags [\"${1:tag}\"]", "Add tags"},
		{"metadata", "metadata {\n  ", "Add metadata"},
		{"description", "description \"${1:Description}\"", "Add description"},
	}

	prefix := strings.ToLower(ctx.KeywordPrefix)

	for _, attr := range attributes {
		if prefix == "" || strings.HasPrefix(attr.label, prefix) {
			items = append(items, CompletionItem{
				Label:      attr.label,
				Kind:       5, // Property
				Detail:     attr.detail,
				InsertText: attr.insertText,
			})
		}
	}

	return items
}

// provideTopLevelCompletions provides completions at architecture top level.
func (cp *CompletionProvider) provideTopLevelCompletions(ctx CompletionContext) []CompletionItem {
	// Combine keyword and element ID completions
	items := cp.provideKeywordCompletions(ctx)

	// Also suggest existing elements that can be referenced
	// (This is less common at top level, but useful for relations)

	return items
}

// isVisibleImport checks if an element is visible via imports in the current file.
func (cp *CompletionProvider) isVisibleImport(elem *ElementReference, filePath string) bool {
	// Check if element's architecture is imported in current file
	aliases := cp.index.GetImportAliases(filePath)
	for _, alias := range aliases {
		if elem.ArchName == alias {
			return true
		}
	}
	return false
}

// ParseText attempts to parse text and extract completion hints.
// Returns nil if parsing fails (tolerant parsing).
func ParseText(text string) *language.Program {
	parser, err := language.NewParser()
	if err != nil {
		return nil
	}

	// Try to parse (may fail for incomplete code)
	program, err := parser.Parse("", text)
	if err != nil {
		return nil
	}

	return program
}

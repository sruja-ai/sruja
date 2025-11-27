// pkg/lsp/metadata_registry.go
package lsp

import (
	"fmt"
	"strings"
)

// MetadataDescriptor describes a metadata key that can be autocompleted.
//
// Plugins register metadata descriptors to extend the DSL with domain-specific
// metadata keys. The LSP uses these descriptors to provide intelligent autocomplete.
type MetadataDescriptor struct {
	Key         string   // Metadata key name (e.g., "rate_limit", "team")
	Type        string   // Value type: "string", "number", "boolean"
	Description string   // Human-readable description
	Enum        []string // Optional: valid values for this key
	Scope       []string // Element types this metadata applies to: "system", "container", "component", "relation", "person", "datastore", "queue"
	Default     string   // Optional: default value
	Required    bool     // Whether this metadata key is required
	Plugin      string   // Plugin that registered this metadata key
}

// MetadataRegistry stores all registered metadata descriptors from core DSL and plugins.
type MetadataRegistry struct {
	descriptors map[string]*MetadataDescriptor // key -> descriptor
	byScope     map[string][]*MetadataDescriptor // scope -> descriptors
}

// NewMetadataRegistry creates a new metadata registry with core DSL metadata keys.
func NewMetadataRegistry() *MetadataRegistry {
	reg := &MetadataRegistry{
		descriptors: make(map[string]*MetadataDescriptor),
		byScope:     make(map[string][]*MetadataDescriptor),
	}
	
	// Register core DSL metadata keys
	reg.registerCoreMetadata()
	
	return reg
}

// registerCoreMetadata registers built-in core metadata keys.
func (r *MetadataRegistry) registerCoreMetadata() {
	coreKeys := []*MetadataDescriptor{
		{
			Key:         "team",
			Type:        "string",
			Description: "Team or organization responsible for this element",
			Scope:       []string{"system", "container", "component"},
			Plugin:      "core",
		},
		{
			Key:         "owner",
			Type:        "string",
			Description: "Owner or contact person for this element",
			Scope:       []string{"system", "container", "component"},
			Plugin:      "core",
		},
		{
			Key:         "tier",
			Type:        "string",
			Description: "Criticality tier (gold, silver, bronze)",
			Enum:        []string{"gold", "silver", "bronze"},
			Scope:       []string{"system", "container", "component"},
			Plugin:      "core",
		},
		{
			Key:         "criticality",
			Type:        "string",
			Description: "Criticality level",
			Enum:        []string{"critical", "high", "medium", "low"},
			Scope:       []string{"system", "container", "component"},
			Plugin:      "core",
		},
		{
			Key:         "tags",
			Type:        "string",
			Description: "Comma-separated tags for categorization",
			Scope:       []string{"system", "container", "component", "person", "datastore", "queue"},
			Plugin:      "core",
		},
		{
			Key:         "cost_center",
			Type:        "string",
			Description: "Cost center code for billing",
			Scope:       []string{"system", "container"},
			Plugin:      "core",
		},
		{
			Key:         "documentation",
			Type:        "string",
			Description: "Link to documentation",
			Scope:       []string{"system", "container", "component"},
			Plugin:      "core",
		},
	}
	
	for _, desc := range coreKeys {
		r.Register(desc)
	}
}

// Register registers a metadata descriptor from a plugin.
//
// If a descriptor with the same key already exists, it will be overwritten.
// This allows plugins to override or extend core metadata keys.
func (r *MetadataRegistry) Register(desc *MetadataDescriptor) {
	r.descriptors[desc.Key] = desc
	
	// Index by scope
	for _, scope := range desc.Scope {
		r.byScope[scope] = append(r.byScope[scope], desc)
	}
}

// Get returns a metadata descriptor by key.
func (r *MetadataRegistry) Get(key string) (*MetadataDescriptor, bool) {
	desc, ok := r.descriptors[key]
	return desc, ok
}

// ForScope returns all metadata descriptors applicable to the given scope.
//
// Scope can be: "system", "container", "component", "relation", "person", "datastore", "queue"
func (r *MetadataRegistry) ForScope(scope string) []*MetadataDescriptor {
	return r.byScope[scope]
}

// All returns all registered metadata descriptors.
func (r *MetadataRegistry) All() []*MetadataDescriptor {
	result := make([]*MetadataDescriptor, 0, len(r.descriptors))
	for _, desc := range r.descriptors {
		result = append(result, desc)
	}
	return result
}

// GetValueSuggestions returns suggested values for a metadata key.
//
// Returns enum values if the descriptor defines them, otherwise returns empty slice.
func (r *MetadataRegistry) GetValueSuggestions(key string) []string {
	desc, ok := r.descriptors[key]
	if !ok {
		return nil
	}
	if len(desc.Enum) > 0 {
		return desc.Enum
	}
	return nil
}

// FilterByPrefix returns metadata descriptors whose keys start with the given prefix.
//
// Useful for filtering autocomplete suggestions as the user types.
func (r *MetadataRegistry) FilterByPrefix(prefix string) []*MetadataDescriptor {
	var result []*MetadataDescriptor
	prefixLower := strings.ToLower(prefix)
	for key, desc := range r.descriptors {
		if strings.HasPrefix(strings.ToLower(key), prefixLower) {
			result = append(result, desc)
		}
	}
	return result
}

// CompletionItem represents an LSP completion item for metadata.
type CompletionItem struct {
	Label         string `json:"label"`
	Kind          int    `json:"kind"` // LSP CompletionItemKind
	Detail        string `json:"detail,omitempty"`
	Documentation string `json:"documentation,omitempty"`
	InsertText    string `json:"insertText,omitempty"`
}

// ToCompletionItem converts a metadata descriptor to an LSP completion item.
func (desc *MetadataDescriptor) ToCompletionItem() CompletionItem {
	detail := fmt.Sprintf("%s: %s", desc.Type, desc.Description)
	if len(desc.Enum) > 0 {
		detail += fmt.Sprintf(" (options: %s)", strings.Join(desc.Enum, ", "))
	}
	
	insertText := desc.Key + ": "
	if desc.Type == "boolean" {
		insertText += "\"true\""
	} else if len(desc.Enum) > 0 {
		insertText += "\"" + desc.Enum[0] + "\""
	} else {
		insertText += "\"\""
	}
	
	return CompletionItem{
		Label:         desc.Key,
		Kind:          5, // LSP CompletionItemKind.Property
		Detail:        detail,
		Documentation: desc.Description,
		InsertText:    insertText,
	}
}


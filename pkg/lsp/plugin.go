// pkg/lsp/plugin.go
package lsp

// CompletionPlugin is the interface that plugins implement to contribute to LSP completions.
//
// Plugins can:
// - Register metadata descriptors
// - Provide custom completion items
// - Define validation rules
// - Add hover documentation
//
// Example plugin:
//
//	type RateLimitPlugin struct{}
//
//	func (p *RateLimitPlugin) MetadataDescriptors(scope string) []*MetadataDescriptor {
//		if scope == "container" {
//			return []*MetadataDescriptor{
//				{
//					Key:         "rate_limit",
//					Type:        "string",
//					Description: "Requests per second",
//					Scope:       []string{"container"},
//					Plugin:      "rate-limit",
//				},
//			}
//		}
//		return nil
//	}
//
//	func (p *RateLimitPlugin) ProvideCompletions(ctx CompletionContext) []CompletionItem {
//		// Custom completion logic
//		return nil
//	}
type CompletionPlugin interface {
	// MetadataDescriptors returns metadata descriptors for a given scope.
	//
	// Scope can be: "system", "container", "component", "relation", "person", "datastore", "queue"
	MetadataDescriptors(scope string) []*MetadataDescriptor
	
	// GetMetadataDescriptor returns a metadata descriptor by key.
	GetMetadataDescriptor(key string) (*MetadataDescriptor, bool)
	
	// ProvideCompletions provides custom completion items based on context.
	ProvideCompletions(ctx CompletionContext) []CompletionItem
}

// BasePlugin provides a default implementation of CompletionPlugin.
//
// Plugins can embed this and override specific methods.
type BasePlugin struct {
	name            string
	metadataKeys    map[string]*MetadataDescriptor
	metadataByScope map[string][]*MetadataDescriptor
}

// NewBasePlugin creates a new base plugin.
func NewBasePlugin(name string) *BasePlugin {
	return &BasePlugin{
		name:            name,
		metadataKeys:    make(map[string]*MetadataDescriptor),
		metadataByScope: make(map[string][]*MetadataDescriptor),
	}
}

// RegisterMetadata registers a metadata descriptor.
func (p *BasePlugin) RegisterMetadata(desc *MetadataDescriptor) {
	desc.Plugin = p.name
	p.metadataKeys[desc.Key] = desc
	
	// Index by scope
	for _, scope := range desc.Scope {
		p.metadataByScope[scope] = append(p.metadataByScope[scope], desc)
	}
}

// MetadataDescriptors implements CompletionPlugin.
func (p *BasePlugin) MetadataDescriptors(scope string) []*MetadataDescriptor {
	return p.metadataByScope[scope]
}

// GetMetadataDescriptor implements CompletionPlugin.
func (p *BasePlugin) GetMetadataDescriptor(key string) (*MetadataDescriptor, bool) {
	desc, ok := p.metadataKeys[key]
	return desc, ok
}

// ProvideCompletions implements CompletionPlugin.
func (p *BasePlugin) ProvideCompletions(ctx CompletionContext) []CompletionItem {
	// Default: no custom completions
	return nil
}


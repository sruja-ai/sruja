// pkg/export/markdown/options.go
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/export/mermaid"
)

// ContextType defines the context for export formatting
type ContextType string

const (
	ContextDefault        ContextType = "default"
	ContextCodeGeneration ContextType = "code_generation"
	ContextReview         ContextType = "review"
	ContextAnalysis       ContextType = "analysis"
)

// Scope defines what part of the architecture to export
type Scope struct {
	Type string // "system", "container", "component", "full"
	ID   string // Element ID (e.g., "OrderService")
}

// ParseScope parses a scope string like "system:OrderService"
func ParseScope(scopeStr string) (*Scope, error) {
	if scopeStr == "" || scopeStr == "full" {
		return &Scope{Type: "full", ID: ""}, nil
	}

	parts := strings.SplitN(scopeStr, ":", 2)
	if len(parts) != 2 {
		return nil, fmt.Errorf("invalid scope format: %s (expected 'type:id')", scopeStr)
	}

	scopeType := strings.ToLower(strings.TrimSpace(parts[0]))
	scopeID := strings.TrimSpace(parts[1])

	switch scopeType {
	case "system", "container", "component":
		return &Scope{Type: scopeType, ID: scopeID}, nil
	default:
		return nil, fmt.Errorf("invalid scope type: %s (expected system, container, or component)", scopeType)
	}
}

// Options represents Markdown export options.
type Options struct {
	IncludeTOC          bool
	IncludeOverview     bool
	IncludeSystems      bool
	IncludeDeployments  bool
	IncludePersons      bool
	IncludeRequirements bool
	IncludeADRs         bool
	IncludeScenarios    bool
	IncludeMetadata     bool
	IncludeGlossary     bool
	MermaidConfig       mermaid.Config
	HeadingLevel        int

	// New AI-friendly options
	Scope      *Scope      // Scope to specific element
	TokenLimit int         // Maximum tokens (0 = no limit)
	Context    ContextType // Context type for formatting
}

// DefaultOptions returns the default Markdown export options.
func DefaultOptions() Options {
	return Options{
		IncludeTOC:          true,
		IncludeOverview:     true,
		IncludeSystems:      true,
		IncludeDeployments:  true,
		IncludePersons:      true,
		IncludeRequirements: true,
		IncludeADRs:         true,
		IncludeScenarios:    true,
		IncludeMetadata:     true,
		IncludeGlossary:     true,
		MermaidConfig:       mermaid.DefaultConfig(),
		HeadingLevel:        1,
		Scope:               &Scope{Type: "full", ID: ""},
		TokenLimit:          0,
		Context:             ContextDefault,
	}
}

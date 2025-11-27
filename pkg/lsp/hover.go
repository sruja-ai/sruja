// pkg/lsp/hover.go
package lsp

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// HoverProvider provides rich hover documentation for LSP.
type HoverProvider struct {
	semanticIndex    *SemanticIndex
	metadataRegistry *MetadataRegistry
}

// NewHoverProvider creates a new hover provider.
func NewHoverProvider(index *SemanticIndex, registry *MetadataRegistry) *HoverProvider {
	return &HoverProvider{
		semanticIndex:    index,
		metadataRegistry: registry,
	}
}

// HoverInfo contains the information to display in a hover.
type HoverInfo struct {
	Contents string // Markdown formatted content
	Range    *Range // Optional: range of text this hover applies to
}

// Range represents a text range in a document.
type Range struct {
	Start Position
	End   Position
}

// Position represents a position in a document.
type Position struct {
	Line      int
	Character int
}

// ProvideHover provides hover documentation for an element at the given position.
func (hp *HoverProvider) ProvideHover(
	filePath string,
	text string,
	line, character int,
) (*HoverInfo, error) {
	// First attempt policy hover (Approval Policy DSL) using heuristics
	if h := hp.ProvideApprovalHover(text, line, character); h != nil {
		return h, nil
	}

	// Parse the file to find the element under cursor
	parser, err := language.NewParser()
	if err != nil {
		return nil, err
	}

	program, err := parser.Parse(filePath, text)
	if err != nil {
		// Try to get basic info even if parse fails
		hoverInfo, err := hp.provideBasicHover(text, line, character)
		if err != nil {
			return nil, err
		}
		return hoverInfo, nil
	}

	// Get identifier at cursor position
	identifier := identifierAt(text, line, character)
	if identifier == "" {
		return nil, nil
	}

	// Find element in semantic index
	elem, ok := hp.semanticIndex.GetElement(identifier)
	if !ok {
		// Try to find in current file
		hoverInfo := hp.findElementInProgram(program, identifier, text, line, character)
		if hoverInfo != nil {
			return hoverInfo, nil
		}
		return nil, nil
	}

	// Generate rich hover documentation
	return hp.generateHoverForElement(elem, program), nil
}

// provideBasicHover provides basic hover info when parsing fails.
func (hp *HoverProvider) provideBasicHover(text string, line, character int) (*HoverInfo, error) {
	identifier := identifierAt(text, line, character)
	if identifier == "" {
		return nil, nil
	}

	// Try to find in semantic index
	if elem, ok := hp.semanticIndex.GetElement(identifier); ok {
		return hp.generateBasicHover(elem), nil
	}

	return nil, nil
}

// findElementInProgram finds an element in the parsed program.
func (hp *HoverProvider) findElementInProgram(
	program *language.Program,
	identifier string,
	text string,
	line, character int,
) *HoverInfo {
	if program == nil || program.Architecture == nil {
		return nil
	}

	arch := program.Architecture

	// Search systems
	for _, sys := range arch.Systems {
		if sys.ID == identifier {
			return hp.generateHoverForSystem(sys)
		}
		// Search containers
		for _, cont := range sys.Containers {
			if cont.ID == identifier {
				return hp.generateHoverForContainer(cont, sys)
			}
			// Search components
			for _, comp := range cont.Components {
				if comp.ID == identifier {
					return hp.generateHoverForComponent(comp, cont, sys)
				}
			}
		}
		// Search components directly in system
		for _, comp := range sys.Components {
			if comp.ID == identifier {
				return hp.generateHoverForComponent(comp, nil, sys)
			}
		}
		// Search datastores
		for _, ds := range sys.DataStores {
			if ds.ID == identifier {
				return hp.generateHoverForDataStore(ds, sys)
			}
		}
		// Search queues
		for _, q := range sys.Queues {
			if q.ID == identifier {
				return hp.generateHoverForQueue(q, sys)
			}
		}
	}

	// Search persons
	for _, person := range arch.Persons {
		if person.ID == identifier {
			return hp.generateHoverForPerson(person)
		}
	}

	return nil
}

// generateHoverForElement generates hover documentation for an element reference.
func (hp *HoverProvider) generateHoverForElement(elem *ElementReference, program *language.Program) *HoverInfo {
	var sb strings.Builder

	// Title
	sb.WriteString(fmt.Sprintf("## %s: %s\n\n", elem.Type, elem.ID))
	sb.WriteString(fmt.Sprintf("**Label:** %s\n\n", elem.Label))

	// Try to get full element details from program
	if program != nil {
		details := hp.findElementInProgram(program, elem.ID, "", 0, 0)
		if details != nil {
			return details
		}
	}

	// Basic info
	sb.WriteString(fmt.Sprintf("**File:** `%s`\n", elem.File))
	if elem.Line > 0 {
		sb.WriteString(fmt.Sprintf("**Location:** Line %d\n", elem.Line))
	}

	return &HoverInfo{
		Contents: sb.String(),
	}
}

// generateBasicHover generates basic hover info from element reference.
func (hp *HoverProvider) generateBasicHover(elem *ElementReference) *HoverInfo {
	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("**%s:** %s\n\n", elem.Type, elem.ID))
	sb.WriteString(fmt.Sprintf("**Label:** %s\n\n", elem.Label))
	sb.WriteString(fmt.Sprintf("Defined in `%s`", elem.File))
	return &HoverInfo{Contents: sb.String()}
}

// generateHoverForSystem generates rich hover documentation for a system.
func (hp *HoverProvider) generateHoverForSystem(sys *language.System) *HoverInfo {
	var sb strings.Builder

	// Title
	sb.WriteString(fmt.Sprintf("## System: %s\n\n", sys.Label))
	sb.WriteString(fmt.Sprintf("**ID:** `%s`\n\n", sys.ID))

	// Description
	if sys.Description != nil && *sys.Description != "" {
		sb.WriteString(fmt.Sprintf("%s\n\n", *sys.Description))
	}

	// Statistics
	sb.WriteString("### Overview\n\n")
	if len(sys.Containers) > 0 {
		sb.WriteString(fmt.Sprintf("- **Containers:** %d\n", len(sys.Containers)))
	}
	if len(sys.Components) > 0 {
		sb.WriteString(fmt.Sprintf("- **Components:** %d\n", len(sys.Components)))
	}
	if len(sys.DataStores) > 0 {
		sb.WriteString(fmt.Sprintf("- **Data Stores:** %d\n", len(sys.DataStores)))
	}
	if len(sys.Queues) > 0 {
		sb.WriteString(fmt.Sprintf("- **Queues:** %d\n", len(sys.Queues)))
	}
	if len(sys.Relations) > 0 {
		sb.WriteString(fmt.Sprintf("- **Relations:** %d\n", len(sys.Relations)))
	}
	sb.WriteString("\n")

	// Metadata
	if len(sys.Metadata) > 0 {
		sb.WriteString("### Metadata\n\n")
		for _, meta := range sys.Metadata {
			// Get metadata descriptor for description
			desc, ok := hp.metadataRegistry.Get(meta.Key)
			var detail string
			if ok && desc.Description != "" {
				detail = fmt.Sprintf(" - %s", desc.Description)
			}
			sb.WriteString(fmt.Sprintf("- **%s:** `%s`%s\n", meta.Key, meta.Value, detail))
		}
		sb.WriteString("\n")
	}

	// Relations summary
	if len(sys.Relations) > 0 {
		sb.WriteString("### Relations\n\n")
		for _, rel := range sys.Relations {
			verb := ""
			if rel.Verb != nil {
				verb = fmt.Sprintf(" (%s)", *rel.Verb)
			}
			label := ""
			if rel.Label != nil {
				label = fmt.Sprintf(" - %s", *rel.Label)
			}
			sb.WriteString(fmt.Sprintf("- `%s` → `%s`%s%s\n", rel.From, rel.To, verb, label))
		}
		sb.WriteString("\n")
	}

	return &HoverInfo{Contents: sb.String()}
}

// generateHoverForContainer generates rich hover documentation for a container.
func (hp *HoverProvider) generateHoverForContainer(cont *language.Container, sys *language.System) *HoverInfo {
	var sb strings.Builder

	sb.WriteString(fmt.Sprintf("## Container: %s\n\n", cont.Label))
	sb.WriteString(fmt.Sprintf("**ID:** `%s`\n", cont.ID))
	if sys != nil {
		sb.WriteString(fmt.Sprintf("**System:** `%s`\n", sys.ID))
	}
	sb.WriteString("\n")

	if cont.Description != nil && *cont.Description != "" {
		sb.WriteString(fmt.Sprintf("%s\n\n", *cont.Description))
	}

	// Technology
	// Note: Technology is in ContainerItem, so we'd need to extract it
	// For now, we'll skip it

	// Statistics
	if len(cont.Components) > 0 {
		sb.WriteString(fmt.Sprintf("- **Components:** %d\n", len(cont.Components)))
	}
	if len(cont.Relations) > 0 {
		sb.WriteString(fmt.Sprintf("- **Relations:** %d\n", len(cont.Relations)))
	}
	sb.WriteString("\n")

	// Metadata
	if len(cont.Metadata) > 0 {
		sb.WriteString("### Metadata\n\n")
		for _, meta := range cont.Metadata {
			desc, ok := hp.metadataRegistry.Get(meta.Key)
			var detail string
			if ok && desc.Description != "" {
				detail = fmt.Sprintf(" - %s", desc.Description)
			}
			sb.WriteString(fmt.Sprintf("- **%s:** `%s`%s\n", meta.Key, meta.Value, detail))
		}
		sb.WriteString("\n")
	}

	return &HoverInfo{Contents: sb.String()}
}

// generateHoverForComponent generates rich hover documentation for a component.
func (hp *HoverProvider) generateHoverForComponent(
	comp *language.Component,
	cont *language.Container,
	sys *language.System,
) *HoverInfo {
	var sb strings.Builder

	sb.WriteString(fmt.Sprintf("## Component: %s\n\n", comp.Label))
	sb.WriteString(fmt.Sprintf("**ID:** `%s`\n", comp.ID))
	if sys != nil {
		sb.WriteString(fmt.Sprintf("**System:** `%s`\n", sys.ID))
	}
	if cont != nil {
		sb.WriteString(fmt.Sprintf("**Container:** `%s`\n", cont.ID))
	}
	sb.WriteString("\n")

	if comp.Description != nil && *comp.Description != "" {
		sb.WriteString(fmt.Sprintf("%s\n\n", *comp.Description))
	}

	if comp.Technology != nil {
		sb.WriteString(fmt.Sprintf("**Technology:** `%s`\n\n", *comp.Technology))
	}

	// Metadata
	if len(comp.Metadata) > 0 {
		sb.WriteString("### Metadata\n\n")
		for _, meta := range comp.Metadata {
			desc, ok := hp.metadataRegistry.Get(meta.Key)
			var detail string
			if ok && desc.Description != "" {
				detail = fmt.Sprintf(" - %s", desc.Description)
			}
			sb.WriteString(fmt.Sprintf("- **%s:** `%s`%s\n", meta.Key, meta.Value, detail))
		}
		sb.WriteString("\n")
	}

	return &HoverInfo{Contents: sb.String()}
}

// generateHoverForDataStore generates hover documentation for a data store.
func (hp *HoverProvider) generateHoverForDataStore(ds *language.DataStore, sys *language.System) *HoverInfo {
	var sb strings.Builder

	sb.WriteString(fmt.Sprintf("## Data Store: %s\n\n", ds.Label))
	sb.WriteString(fmt.Sprintf("**ID:** `%s`\n", ds.ID))
	if sys != nil {
		sb.WriteString(fmt.Sprintf("**System:** `%s`\n", sys.ID))
	}
	sb.WriteString("\n")

	if ds.Description != nil && *ds.Description != "" {
		sb.WriteString(fmt.Sprintf("%s\n\n", *ds.Description))
	}

	// Metadata
	if len(ds.Metadata) > 0 {
		sb.WriteString("### Metadata\n\n")
		for _, meta := range ds.Metadata {
			sb.WriteString(fmt.Sprintf("- **%s:** `%s`\n", meta.Key, meta.Value))
		}
	}

	return &HoverInfo{Contents: sb.String()}
}

// generateHoverForQueue generates hover documentation for a queue.
func (hp *HoverProvider) generateHoverForQueue(q *language.Queue, sys *language.System) *HoverInfo {
	var sb strings.Builder

	sb.WriteString(fmt.Sprintf("## Queue: %s\n\n", q.Label))
	sb.WriteString(fmt.Sprintf("**ID:** `%s`\n", q.ID))
	if sys != nil {
		sb.WriteString(fmt.Sprintf("**System:** `%s`\n", sys.ID))
	}
	sb.WriteString("\n")

	if q.Description != nil && *q.Description != "" {
		sb.WriteString(fmt.Sprintf("%s\n\n", *q.Description))
	}

	return &HoverInfo{Contents: sb.String()}
}

// generateHoverForPerson generates hover documentation for a person.
func (hp *HoverProvider) generateHoverForPerson(person *language.Person) *HoverInfo {
	var sb strings.Builder

	sb.WriteString(fmt.Sprintf("## Person: %s\n\n", person.Label))
	sb.WriteString(fmt.Sprintf("**ID:** `%s`\n\n", person.ID))

	if len(person.Metadata) > 0 {
		sb.WriteString("### Metadata\n\n")
		for _, meta := range person.Metadata {
			sb.WriteString(fmt.Sprintf("- **%s:** `%s`\n", meta.Key, meta.Value))
		}
	}

	return &HoverInfo{Contents: sb.String()}
}

// generateHoverForMetadataKey generates hover documentation for a metadata key.
func (hp *HoverProvider) generateHoverForMetadataKey(key string, scope string) *HoverInfo {
	desc, ok := hp.metadataRegistry.Get(key)
	if !ok {
		return nil
	}

	var sb strings.Builder

	sb.WriteString(fmt.Sprintf("## Metadata: `%s`\n\n", key))
	sb.WriteString(fmt.Sprintf("**Type:** `%s`\n\n", desc.Type))

	if desc.Description != "" {
		sb.WriteString(fmt.Sprintf("%s\n\n", desc.Description))
	}

	if len(desc.Enum) > 0 {
		sb.WriteString("**Valid values:**\n")
		for _, val := range desc.Enum {
			sb.WriteString(fmt.Sprintf("- `%s`\n", val))
		}
		sb.WriteString("\n")
	}

	if desc.Plugin != "" && desc.Plugin != "core" {
		sb.WriteString(fmt.Sprintf("*Provided by plugin: %s*\n", desc.Plugin))
	}

	return &HoverInfo{Contents: sb.String()}
}

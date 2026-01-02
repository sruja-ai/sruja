package context

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Options defines configuration for the context export.
type Options struct {
	// Template is the instruction template to use (e.g., "proposal", "security")
	Template string
	// Scope defines the scope of the export ("full", "system", "container")
	Scope string
	// Format is the output format (default "markdown")
	Format string
}

// Exporter handles Context document generation.
type Exporter struct {
	Options Options
}

// NewExporter creates a new Context exporter.
func NewExporter(options Options) *Exporter {
	return &Exporter{Options: options}
}

// Export generates a Context document from a program.
func (e *Exporter) Export(prog *language.Program) string {
	if prog == nil || prog.Model == nil {
		return ""
	}

	f := NewFormatter()

	// Header
	instruction := GetTemplate(e.Options.Template)
	f.WriteHeader("Architecture Model", instruction)

	f.sb.WriteString("# Architecture Elements\n\n")

	// Recursive helper to visit elements
	var visitElement func(elDef *language.ElementDef, depth int)
	visitElement = func(elDef *language.ElementDef, depth int) {
		name := elDef.GetID()
		kind := elDef.GetKind()

		el := Element{
			Name:        name,
			Kind:        kind,
			Description: "",
			Metadata:    make(map[string]string),
		}

		// Extract properties from body
		body := elDef.GetBody()
		if body != nil {
			for _, item := range body.Items {
				// Description
				if item.Description != nil {
					el.Description = *item.Description
				}
				// Technology
				if item.Technology != nil {
					el.Metadata["technology"] = *item.Technology
				}
				// Tags
				if len(item.Tags) > 0 {
					el.Metadata["tags"] = strings.Join(item.Tags, ", ")
				}
				// Metadata block
				if item.Metadata != nil {
					for _, entry := range item.Metadata.Entries {
						if entry.Value != nil {
							if isRelevantMetadata(entry.Key) {
								el.Metadata[entry.Key] = *entry.Value
							}
						}
					}
				}

			}
		}

		f.WriteElement(el, depth)

		// Recurse for children
		if body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					visitElement(item.Element, depth+1)
				}
			}
		}
	}

	// top-level items
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil {
			visitElement(item.ElementDef, 0)
		}
	}

	f.sb.WriteString("\n# Relationships\n\n")

	// Helper to collect relations
	collectFn := func(rel *language.Relation) {
		r := Relation{
			Source: rel.From.String(),
			Target: rel.To.String(),
			Label:  "",
		}
		if rel.Label != nil {
			r.Label = *rel.Label
		} else if rel.Verb != nil { // Fallback to verb if no label
			r.Label = *rel.Verb
		}
		f.WriteRelationship(r)
	}

	// Recursive visitor for relations
	var visitRelations func(elDef *language.ElementDef)
	visitRelations = func(elDef *language.ElementDef) {
		body := elDef.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Relation != nil {
					collectFn(item.Relation)
				}
				if item.Element != nil {
					visitRelations(item.Element)
				}
			}
		}
	}

	// top-level relations and recurse
	for _, item := range prog.Model.Items {
		if item.Relation != nil {
			collectFn(item.Relation)
		}
		if item.ElementDef != nil {
			visitRelations(item.ElementDef)
		}
	}

	return f.String()
}

func isRelevantMetadata(key string) bool {
	keys := []string{"technology", "tech", "hosting", "cost", "tier", "replicas", "owner", "language", "framework"}
	for _, k := range keys {
		if strings.EqualFold(key, k) {
			return true
		}
	}
	return false
}

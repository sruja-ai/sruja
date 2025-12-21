package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

func (e *Exporter) writeSystems(sb *strings.Builder, arch interface{}, prog *language.Program) {
	archStruct := arch.(*struct {
		Name         string
		Description  *string
		Systems      []*language.System
		Persons      []*language.Person
		Requirements []*language.Requirement
		ADRs         []*language.ADR
	})
	if len(archStruct.Systems) == 0 {
		return
	}

	sb.WriteString("## System Architecture\n\n")
	sb.WriteString("This section details the internal structure of each system, including containers, data stores, queues, and their constituent components.\n\n")

	for _, sys := range archStruct.Systems {
		fmt.Fprintf(sb, "### %s\n\n", sys.Label)
		if sys.Description != nil {
			fmt.Fprintf(sb, "%s\n\n", *sys.Description)
		} else {
			fmt.Fprintf(sb, "The %s system provides core functionality for the architecture.\n\n", sys.Label)
		}

		// Display properties if available (tags may be in properties)
		if len(sys.Properties) > 0 {
			sb.WriteString("**Properties:**\n\n")
			for key, value := range sys.Properties {
				fmt.Fprintf(sb, "- `%s`: %s\n", key, value)
			}
			sb.WriteString("\n")
		}

		// Generate L2 diagram (Container View) for this system
		if len(sys.Containers) > 0 || len(sys.DataStores) > 0 || len(sys.Queues) > 0 {
			l2Diagram := e.generateL2Diagram(sys, prog)
			if l2Diagram != "" {
				sb.WriteString("#### Container Diagram (Level 2)\n\n")
				sb.WriteString("The Container diagram shows the high-level structure of the system, including application containers, data stores, and message queues.\n\n")
				sb.WriteString("```mermaid\n")
				sb.WriteString(l2Diagram)
				sb.WriteString("\n```\n\n")
			}

			// Generate L3 diagrams (Component View) for containers with components
			if len(sys.Containers) > 0 {
				hasComponents := false
				for _, cont := range sys.Containers {
					if len(cont.Components) > 0 {
						hasComponents = true
						break
					}
				}
				if hasComponents {
					sb.WriteString("#### Component Diagrams (Level 3)\n\n")
					sb.WriteString("The following diagrams detail the internal component structure of each container.\n\n")
				}
			}

			for _, cont := range sys.Containers {
				if len(cont.Components) > 0 {
					l3Diagram := e.generateL3Diagram(cont, sys.ID, prog)
					if l3Diagram != "" {
						fmt.Fprintf(sb, "##### %s Component Diagram\n\n", cont.Label)
						fmt.Fprintf(sb, "This diagram shows the internal components of the %s container.\n\n", cont.Label)
						sb.WriteString("```mermaid\n")
						sb.WriteString(l3Diagram)
						sb.WriteString("\n```\n\n")
					}
				}
			}
		}
	}
}

func (e *Exporter) writePersons(sb *strings.Builder, arch interface{}) {
	archStruct := arch.(*struct {
		Name         string
		Description  *string
		Systems      []*language.System
		Persons      []*language.Person
		Requirements []*language.Requirement
		ADRs         []*language.ADR
	})
	if len(archStruct.Persons) == 0 {
		return
	}

	sb.WriteString("## Actors and User Roles\n\n")
	sb.WriteString("This section identifies the external actors and user roles that interact with the system.\n\n")
	for _, p := range archStruct.Persons {
		desc := ""
		if p.Description != nil {
			desc = *p.Description
		} else {
			desc = "External actor interacting with the system"
		}
		fmt.Fprintf(sb, "- **%s**: %s\n", p.Label, desc)
	}
	sb.WriteString("\n")
}

// writeRelationships extracts and writes relationships with prioritization
func (e *Exporter) writeRelationships(sb *strings.Builder, prog *language.Program) {
	if prog == nil || prog.Model == nil {
		return
	}

	// Extract and prioritize relationships
	relations := extractRelationsFromModel(prog)
	if len(relations) == 0 {
		return
	}

	// Prioritize relationships (direct relationships first, then others)
	prioritized := prioritizeRelationships(relations)

	sb.WriteString("## System Relationships and Data Flow\n\n")
	sb.WriteString("This section describes the key relationships and data flows between system components.\n\n")

	// Limit relationships based on token budget if needed
	maxRelations := len(prioritized)
	if e.Options.TokenLimit > 0 {
		// Rough estimate: each relationship ~50 tokens
		availableTokens := e.Options.TokenLimit / 4 // Convert to chars
		estimatedUsed := len(sb.String())
		remaining := availableTokens - estimatedUsed
		maxRelationsForBudget := remaining / 200 // ~50 tokens per relation * 4 chars
		if maxRelationsForBudget < maxRelations {
			maxRelations = maxRelationsForBudget
		}
		if maxRelations < 1 {
			maxRelations = 1 // Always show at least one
		}
	}

	for i, rel := range prioritized {
		if i >= maxRelations {
			break
		}
		from := rel.From.String()
		to := rel.To.String()
		label := getString(rel.Label)
		verb := ""
		switch {
		case rel.Verb != nil:
			verb = *rel.Verb
		case rel.VerbRaw != nil:
			verb = rel.VerbRaw.Value
		}

		switch {
		case label != "":
			fmt.Fprintf(sb, "- **%s** → **%s**: %s\n", from, to, label)
		case verb != "":
			fmt.Fprintf(sb, "- **%s** → **%s**: %s\n", from, to, verb)
		default:
			fmt.Fprintf(sb, "- **%s** → **%s**\n", from, to)
		}
	}

	if len(prioritized) > maxRelations {
		fmt.Fprintf(sb, "\n*... and %d more relationships*\n", len(prioritized)-maxRelations)
	}

	sb.WriteString("\n")
}

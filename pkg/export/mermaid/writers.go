package mermaid

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Mermaid styles
const (
// Constants moved to constants.go
)

func (e *Exporter) writeHeader(sb *strings.Builder) {
	if e.Config.UseFrontmatter {
		sb.WriteString("---\nconfig:\n")
		if e.Config.Layout != "" {
			fmt.Fprintf(sb, "  layout: %s\n", e.Config.Layout)
		}
		if e.Config.Theme != "" && e.Config.Theme != DefaultTheme {
			fmt.Fprintf(sb, "  theme: %s\n", e.Config.Theme)
		}
		if e.Config.Direction != "" {
			fmt.Fprintf(sb, "  direction: %s\n", strings.ToLower(e.Config.Direction))
		}
		sb.WriteString("---\n")
	} else {
		theme := e.Config.Theme
		if theme == "" {
			theme = DefaultTheme
		}
		fmt.Fprintf(sb, "%%%%{init: { \"theme\": \"%s\", \"flowchart\": { \"htmlLabels\": true } }}%%%%\n", theme)
	}

	dir := e.Config.Direction
	if dir == "" {
		dir = DefaultDirection
	}
	fmt.Fprintf(sb, "graph %s\n\n", dir)
}

func (e *Exporter) writeStyles(sb *strings.Builder) {
	fmt.Fprintf(sb, "    classDef %s %s\n", ClassPerson, StylePerson)
	fmt.Fprintf(sb, "    classDef %s %s\n", ClassSystem, StyleSystem)
	fmt.Fprintf(sb, "    classDef %s %s\n", ClassContainer, StyleContainer)
	fmt.Fprintf(sb, "    classDef %s %s\n", ClassDatabase, StyleDatabase)
	fmt.Fprintf(sb, "    classDef %s %s\n", ClassQueue, StyleQueue)
	fmt.Fprintf(sb, "    classDef %s %s\n", ClassExternal, StyleExternal)
	fmt.Fprintf(sb, "    classDef %s %s\n\n", ClassComponent, StyleComponent)
}

func (e *Exporter) writePerson(sb *strings.Builder, p *language.Person) {
	id := sanitizeID(p.ID)
	label := escapeQuotes(formatLabel(p.Label, p.ID, getString(p.Description), ""))
	fmt.Fprintf(sb, "    %s[\"%s\"]\n", id, label)
	fmt.Fprintf(sb, "    class %s %s\n", id, ClassPerson)
}

func (e *Exporter) writeSystem(sb *strings.Builder, sys *language.System, _ *indexedArchitecture) {
	id := sanitizeID(sys.ID)

	// If system has containers, use subgraph
	if len(sys.Containers) > 0 || len(sys.DataStores) > 0 || len(sys.Queues) > 0 {
		label := escapeQuotes(sys.Label)
		if label == "" {
			label = sys.ID
		}
		fmt.Fprintf(sb, "    subgraph %s[\"%s\"]\n", id, label)
		for _, cont := range sys.Containers {
			e.writeContainer(sb, cont, sys.ID, "        ")
		}
		for _, ds := range sys.DataStores {
			e.writeDataStore(sb, ds, sys.ID, "        ")
		}
		for _, q := range sys.Queues {
			e.writeQueue(sb, q, sys.ID, "        ")
		}
		sb.WriteString("    end\n")
	} else {
		label := escapeQuotes(formatLabel(sys.Label, sys.ID, getString(sys.Description), ""))
		fmt.Fprintf(sb, "    %s[\"%s\"]\n", id, label)
		fmt.Fprintf(sb, "    class %s %s\n", id, ClassSystem)
	}
}

func (e *Exporter) writeContainer(sb *strings.Builder, cont *language.Container, parentID string, indent string) {
	fullID := cont.ID
	if parentID != "" && !strings.Contains(cont.ID, parentID) {
		fullID = parentID + "." + cont.ID
	}
	id := sanitizeID(fullID)

	// Container does not have direct Technology field in AST element struct, but we can look for it
	tech := ""
	if t, ok := cont.Properties["technology"]; ok {
		tech = t
	}
	label := escapeQuotes(formatLabel(cont.Label, cont.ID, getString(cont.Description), tech))

	// If container has components, use subgraph
	if len(cont.Components) > 0 {
		fmt.Fprintf(sb, "%ssubgraph %s[\"%s\"]\n", indent, id, label)
		for _, comp := range cont.Components {
			e.writeComponent(sb, comp, fullID, indent+"    ")
		}
		fmt.Fprintf(sb, "%send\n", indent)
	} else {
		fmt.Fprintf(sb, "%s%s[\"%s\"]\n", indent, id, label)
		fmt.Fprintf(sb, "%sclass %s %s\n", indent, id, ClassContainer)
	}
}

func (e *Exporter) writeDataStore(sb *strings.Builder, ds *language.DataStore, parentID string, indent string) {
	fullID := ds.ID
	if parentID != "" && !strings.Contains(ds.ID, parentID) {
		fullID = parentID + "." + ds.ID
	}
	id := sanitizeID(fullID)
	label := escapeQuotes(formatLabel(ds.Label, ds.ID, getString(ds.Description), getString(ds.Technology)))
	fmt.Fprintf(sb, "%s%s[(\"%s\")]\n", indent, id, label)
	fmt.Fprintf(sb, "%sclass %s %s\n", indent, id, ClassDatabase)
}

func (e *Exporter) writeQueue(sb *strings.Builder, q *language.Queue, parentID string, indent string) {
	fullID := q.ID
	if parentID != "" && !strings.Contains(q.ID, parentID) {
		fullID = parentID + "." + q.ID
	}
	id := sanitizeID(fullID)
	label := escapeQuotes(formatLabel(q.Label, q.ID, getString(q.Description), getString(q.Technology)))
	fmt.Fprintf(sb, "%s%s(\"%s\")\n", indent, id, label)
	fmt.Fprintf(sb, "%sclass %s %s\n", indent, id, ClassQueue)
}

func (e *Exporter) writeComponent(sb *strings.Builder, comp *language.Component, parentID string, indent string) {
	fullID := comp.ID
	if parentID != "" && !strings.Contains(comp.ID, parentID) {
		fullID = parentID + "." + comp.ID
	}
	id := sanitizeID(fullID)
	label := escapeQuotes(formatLabel(comp.Label, comp.ID, getString(comp.Description), getString(comp.Technology)))
	fmt.Fprintf(sb, "%s%s[\"%s\"]\n", indent, id, label)
	fmt.Fprintf(sb, "%sclass %s %s\n", indent, id, ClassComponent)
}

func (e *Exporter) writeRelation(sb *strings.Builder, rel *language.Relation, _ *indexedArchitecture) {
	from := sanitizeID(rel.From.String())
	to := sanitizeID(rel.To.String())

	label := getString(rel.Label)
	if label == "" {
		label = getString(rel.Verb)
	}

	if label != "" {
		fmt.Fprintf(sb, "    %s -->|\"%s\"| %s\n", from, escapeQuotes(label), to)
	} else {
		fmt.Fprintf(sb, "    %s --> %s\n", from, to)
	}
}

// Helpers

func getString(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

func sanitizeID(id string) string {
	return strings.Map(func(r rune) rune {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') {
			return r
		}
		return '_'
	}, id)
}

func escapeQuotes(s string) string {
	return strings.ReplaceAll(s, "\"", "#quot;")
}

func formatLabel(label, id, description, technology string) string {
	main := label
	if main == "" {
		main = id
	}

	res := main
	if technology != "" {
		res += fmt.Sprintf("\n(%s)", technology)
	}
	if description != "" {
		// Truncate description for Mermaid
		desc := description
		if len(desc) > 50 {
			desc = desc[:47] + "..."
		}
		res += fmt.Sprintf("\n%s", desc)
	}
	return res
}

func indexProgram(prog *language.Program) *indexedArchitecture {
	idx := &indexedArchitecture{
		ids: make(map[string]bool),
	}
	// Index all element IDs from the model
	if prog != nil && prog.Model != nil {
		var indexElement func(elem *language.ElementDef, prefix string)
		indexElement = func(elem *language.ElementDef, prefix string) {
			if elem == nil {
				return
			}
			id := elem.GetID()
			if id != "" {
				fullID := id
				if prefix != "" {
					fullID = prefix + "." + id
				}
				idx.ids[fullID] = true
				body := elem.GetBody()
				if body != nil {
					for _, bodyItem := range body.Items {
						if bodyItem.Element != nil {
							indexElement(bodyItem.Element, fullID)
						}
					}
				}
			}
		}
		for _, item := range prog.Model.Items {
			if item.ElementDef != nil {
				indexElement(item.ElementDef, "")
			}
		}
	}
	return idx
}

type indexedArchitecture struct {
	ids map[string]bool
}

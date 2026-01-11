package mermaid

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/export/views"
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

func (e *Exporter) writePerson(sb *strings.Builder, p *views.Element) {
	id := sanitizeID(p.ID)
	label := escapeQuotes(formatLabel(p.Title, p.ID, p.Description, p.Technology))
	fmt.Fprintf(sb, "    %s[\"%s\"]\n", id, label)
	fmt.Fprintf(sb, "    class %s %s\n", id, ClassPerson)
}

func (e *Exporter) writeContainer(sb *strings.Builder, cont *views.Element, indent string) {
	id := sanitizeID(cont.ID)
	label := escapeQuotes(formatLabel(cont.Title, cont.ID, cont.Description, cont.Technology))

	fmt.Fprintf(sb, "%s%s[\"%s\"]\n", indent, id, label)
	fmt.Fprintf(sb, "%sclass %s %s\n", indent, id, ClassContainer)
}

func (e *Exporter) writeDataStore(sb *strings.Builder, ds *views.Element, indent string) {
	id := sanitizeID(ds.ID)
	label := escapeQuotes(formatLabel(ds.Title, ds.ID, ds.Description, ds.Technology))
	fmt.Fprintf(sb, "%s%s[(\"%s\")]\n", indent, id, label)
	fmt.Fprintf(sb, "%sclass %s %s\n", indent, id, ClassDatabase)
}

func (e *Exporter) writeQueue(sb *strings.Builder, q *views.Element, indent string) {
	id := sanitizeID(q.ID)
	label := escapeQuotes(formatLabel(q.Title, q.ID, q.Description, q.Technology))
	fmt.Fprintf(sb, "%s%s(\"%s\")\n", indent, id, label)
	fmt.Fprintf(sb, "%sclass %s %s\n", indent, id, ClassQueue)
}

func (e *Exporter) writeComponent(sb *strings.Builder, comp *views.Element, indent string) {
	id := sanitizeID(comp.ID)
	label := escapeQuotes(formatLabel(comp.Title, comp.ID, comp.Description, comp.Technology))
	fmt.Fprintf(sb, "%s%s[\"%s\"]\n", indent, id, label)
	fmt.Fprintf(sb, "%sclass %s %s\n", indent, id, ClassComponent)
}

func (e *Exporter) writeRelation(sb *strings.Builder, rel *views.Relation) {
	from := sanitizeID(rel.From)
	to := sanitizeID(rel.To)

	label := rel.Label
	if label != "" {
		fmt.Fprintf(sb, "    %s -->|\"%s\"| %s\n", from, escapeQuotes(label), to)
	} else {
		fmt.Fprintf(sb, "    %s --> %s\n", from, to)
	}
}

// Helpers

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

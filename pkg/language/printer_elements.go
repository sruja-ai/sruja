// pkg/language/printer_elements.go
// Printer methods for architecture elements (System, Container, Component, etc.)
package language

import (
	"fmt"
	"strings"
)

// printRelation prints a relation node.
func (p *Printer) printRelation(sb *strings.Builder, rel *Relation) {
	indent := p.indent()
	sb.WriteString(indent)
	sb.WriteString(strings.Join(rel.From.Parts, "."))
	sb.WriteString(" -> ")
	sb.WriteString(strings.Join(rel.To.Parts, "."))
	if len(rel.Tags) > 0 {
		sb.WriteString(" [")
		for i, tag := range rel.Tags {
			if i > 0 {
				sb.WriteString(", ")
			}
			sb.WriteString(tag)
		}
		sb.WriteString("]")
	}
	if rel.Verb != nil {
		sb.WriteString(" ")
		if isIdent(*rel.Verb) {
			sb.WriteString(*rel.Verb)
		} else {
			_, _ = fmt.Fprintf(sb, "%q", *rel.Verb)
		}
	}
	if rel.Label != nil && *rel.Label != "" {
		sb.WriteString(" ")
		_, _ = fmt.Fprintf(sb, "%q", *rel.Label)
	}
	sb.WriteString("\n")
}

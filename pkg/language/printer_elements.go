// pkg/language/printer_elements.go
// Printer methods for architecture elements (System, Container, Component, etc.)
package language

import (
	"fmt"
	"strings"
)

// printSystem prints a system node.
func (p *Printer) printSystem(sb *strings.Builder, sys *System) {
	indent := p.indent()
	fmt.Fprintf(sb, "%ssystem %s %q", indent, sys.ID, sys.Label)

	if sys.Description != nil && *sys.Description != "" {
		fmt.Fprintf(sb, " %q", *sys.Description)
	}

	if len(sys.Items) > 0 || len(sys.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++

		for _, item := range sys.Items {
			if item.Container != nil {
				p.printContainer(sb, item.Container)
			}
			if item.DataStore != nil {
				p.printDataStore(sb, item.DataStore)
			}
			if item.Queue != nil {
				p.printQueue(sb, item.Queue)
			}
			if item.Person != nil {
				p.printPerson(sb, item.Person)
			}
			if item.Relation != nil {
				p.printRelation(sb, item.Relation)
			}
			if item.Metadata != nil {
				p.printMetadataBlock(sb, item.Metadata)
			}
		}

		if len(sys.Metadata) > 0 {
			p.printMetadata(sb, sys.Metadata)
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printContainer prints a container node.
//
//nolint:gocyclo // Printing logic is complex
func (p *Printer) printContainer(sb *strings.Builder, cont *Container) {
	indent := p.indent()
	fmt.Fprintf(sb, "%scontainer %s %q", indent, cont.ID, cont.Label)

	if cont.Description != nil && *cont.Description != "" {
		fmt.Fprintf(sb, " %q", *cont.Description)
	}

	if len(cont.Items) > 0 || len(cont.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++

		//nolint:gocyclo,gocritic // Logic is complex and verbose
		for i := range cont.Items {
			item := &cont.Items[i]
			if item.Technology != nil {
				indent = p.indent()
				fmt.Fprintf(sb, "%stechnology %q\n", indent, *item.Technology)
			}
			if len(item.Tags) > 0 {
				indent = p.indent()
				fmt.Fprintf(sb, "%stags [", indent)
				for i, tag := range item.Tags {
					if i > 0 {
						sb.WriteString(", ")
					}
					fmt.Fprintf(sb, "%q", tag)
				}
				sb.WriteString("]\n")
			}
			if item.Version != nil {
				indent = p.indent()
				fmt.Fprintf(sb, "%sversion %q\n", indent, *item.Version)
			}
			if item.Component != nil {
				p.printComponent(sb, item.Component)
			}
			if item.DataStore != nil {
				p.printDataStore(sb, item.DataStore)
			}
			if item.Queue != nil {
				p.printQueue(sb, item.Queue)
			}
			if item.Relation != nil {
				p.printRelation(sb, item.Relation)
			}
			if item.Metadata != nil {
				p.printMetadataBlock(sb, item.Metadata)
			}
		}

		if len(cont.Metadata) > 0 {
			p.printMetadata(sb, cont.Metadata)
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printComponent prints a component node.
func (p *Printer) printComponent(sb *strings.Builder, comp *Component) {
	indent := p.indent()
	fmt.Fprintf(sb, "%scomponent %s %q", indent, comp.ID, comp.Label)

	if comp.Description != nil && *comp.Description != "" {
		fmt.Fprintf(sb, " %q", *comp.Description)
	}

	if comp.Technology != nil || len(comp.Items) > 0 || len(comp.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++

		if comp.Technology != nil {
			indent = p.indent()
			fmt.Fprintf(sb, "%stechnology %q\n", indent, *comp.Technology)
		}

		for _, item := range comp.Items {
			if item.Relation != nil {
				p.printRelation(sb, item.Relation)
			}
			if item.Metadata != nil {
				p.printMetadataBlock(sb, item.Metadata)
			}
		}

		if len(comp.Metadata) > 0 {
			p.printMetadata(sb, comp.Metadata)
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printDataStore prints a datastore node.
func (p *Printer) printDataStore(sb *strings.Builder, ds *DataStore) {
	indent := p.indent()
	fmt.Fprintf(sb, "%sdatastore %s %q", indent, ds.ID, ds.Label)
	if ds.Description != nil && *ds.Description != "" {
		fmt.Fprintf(sb, " %q", *ds.Description)
	}
	if len(ds.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++
		p.printMetadata(sb, ds.Metadata)
		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printQueue prints a queue node.
func (p *Printer) printQueue(sb *strings.Builder, q *Queue) {
	indent := p.indent()
	fmt.Fprintf(sb, "%squeue %s %q", indent, q.ID, q.Label)
	if q.Description != nil && *q.Description != "" {
		fmt.Fprintf(sb, " %q", *q.Description)
	}
	if len(q.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++
		p.printMetadata(sb, q.Metadata)
		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printPerson prints a person node.
func (p *Printer) printPerson(sb *strings.Builder, person *Person) {
	indent := p.indent()
	fmt.Fprintf(sb, "%sperson %s %q", indent, person.ID, person.Label)
	if len(person.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++
		p.printMetadata(sb, person.Metadata)
		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		sb.WriteString("\n")
	}
}

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
			sb.WriteString(fmt.Sprintf("%q", *rel.Verb))
		}
	}
	if rel.Label != nil && *rel.Label != "" {
		sb.WriteString(" ")
		sb.WriteString(fmt.Sprintf("%q", *rel.Label))
	}
	sb.WriteString("\n")
}

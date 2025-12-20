// pkg/language/printer_blocks.go
// Printer methods for blocks (Policy, Flow, Requirement, ADR, etc.)
package language

import (
	"fmt"
	"strings"
)

// printFlow prints a flow node.
func (p *Printer) printFlow(sb *strings.Builder, flow *Flow) {
	indent := p.indent()
	sb.WriteString(indent)
	sb.WriteString("flow ")
	sb.WriteString(flow.ID)
	sb.WriteString(" ")
	if flow.Title != nil {
		sb.WriteString(fmt.Sprintf("%q {\n", *flow.Title))
	} else {
		sb.WriteString("{\n") // Optional title
	}
	p.IndentLevel++

	if flow.Description != nil {
		stepIndent := p.indent()
		sb.WriteString(stepIndent)
		sb.WriteString("description ")
		sb.WriteString(fmt.Sprintf("%q\n", *flow.Description))
	}

	for _, item := range flow.Items {
		if item.Step != nil {
			stepIndent := p.indent()
			sb.WriteString(stepIndent)
			sb.WriteString(item.Step.From.String())
			sb.WriteString(" -> ")
			sb.WriteString(item.Step.To.String())
			if item.Step.Description != nil {
				sb.WriteString(" ")
				sb.WriteString(fmt.Sprintf("%q", *item.Step.Description))
			}
			if len(item.Step.Tags) > 0 {
				sb.WriteString(" [")
				for i, tag := range item.Step.Tags {
					if i > 0 {
						sb.WriteString(", ")
					}
					sb.WriteString(tag)
				}
				sb.WriteString("]")
			}
			if item.Step.Order != nil {
				sb.WriteString(" order ")
				sb.WriteString(fmt.Sprintf("%q", *item.Step.Order))
			}
			sb.WriteString("\n")
		}
	}

	p.IndentLevel--
	sb.WriteString(indent)
	sb.WriteString("}\n")
}

// printRequirement prints a requirement node.
func (p *Printer) printRequirement(sb *strings.Builder, req *Requirement) {
	indent := p.indent()
	fmt.Fprintf(sb, "%srequirement %s", indent, req.ID)

	if req.Type != nil {
		fmt.Fprintf(sb, " %s", *req.Type)
	}
	if req.Description != nil {
		fmt.Fprintf(sb, " %q", *req.Description)
	}

	if req.Body != nil {
		sb.WriteString(" {\n")
		p.IndentLevel++
		indent = p.indent()

		if req.Body.Type != nil {
			fmt.Fprintf(sb, "%stype %q\n", indent, *req.Body.Type)
		}
		if req.Body.Description != nil {
			fmt.Fprintf(sb, "%sdescription %q\n", indent, *req.Body.Description)
		}
		if req.Body.Metadata != nil {
			p.printMetadataBlock(sb, req.Body.Metadata)
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		sb.WriteString("\n")
	}
}

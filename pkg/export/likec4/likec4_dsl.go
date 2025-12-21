package likec4

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

// DSLExporter generates LikeC4 DSL (.c4) format.
type DSLExporter struct{}

// NewDSLExporter creates a new DSL exporter.
func NewDSLExporter() *DSLExporter {
	return &DSLExporter{}
}

// ExportDSL converts a LikeC4 Program to LikeC4 DSL format.
func (e *DSLExporter) ExportDSL(program *language.Program) string {
	if program == nil || program.Model == nil {
		return ""
	}

	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	// Write specification block
	e.writeSpecification(sb, program)
	if program.Specification != nil {
		sb.WriteString("\n")
	}

	// Write model block
	e.writeModel(sb, program)

	// Write views block if there are items or views defined
	if program.Views != nil && len(program.Views.Items) > 0 {
		sb.WriteString("\n")
		e.writeViews(sb, program)
	}

	return sb.String()
}

func (e *DSLExporter) writeSpecification(sb *strings.Builder, program *language.Program) {
	if program.Specification != nil {
		sb.WriteString("specification {\n")
		// For simplicity, we emit default elements if the block exists.
		// In a full implementation, we'd iterate program.Specification.Items.
		sb.WriteString("  element component\n")
		sb.WriteString("  element database\n")
		sb.WriteString("  element queue\n")
		sb.WriteString("}\n")
	}
}

func (e *DSLExporter) writeModel(sb *strings.Builder, program *language.Program) {
	if program.Model == nil {
		sb.WriteString("model {\n}\n")
		return
	}

	sb.WriteString("model {\n")
	for _, item := range program.Model.Items {
		e.writeModelItem(sb, item, "  ")
	}
	sb.WriteString("}\n")
}

func (e *DSLExporter) writeModelItem(sb *strings.Builder, item language.ModelItem, indent string) {
	switch {
	case item.Requirement != nil:
		p := language.NewPrinter()
		psb := &strings.Builder{}
		p.PrintRequirement(psb, item.Requirement)
		sb.WriteString(indent + strings.TrimSpace(psb.String()) + "\n")
	case item.ADR != nil:
		p := language.NewPrinter()
		psb := &strings.Builder{}
		p.PrintADR(psb, item.ADR)
		sb.WriteString(indent + strings.TrimSpace(psb.String()) + "\n")
	case item.Relation != nil:
		e.writeRelation(sb, item.Relation, indent)
	case item.ElementDef != nil:
		e.writeElement(sb, item.ElementDef, indent)
	}
}

func (e *DSLExporter) writeElement(sb *strings.Builder, elem *language.LikeC4ElementDef, indent string) {
	id := elem.GetID()
	kind := elem.GetKind()
	// Fmt expects assignment style: S = system "S"
	if id != "" {
		fmt.Fprintf(sb, "%s%s = %s", indent, id, kind)
	} else {
		fmt.Fprintf(sb, "%s%s", indent, kind)
	}

	title := elem.GetTitle()
	if title != nil {
		fmt.Fprintf(sb, " %q", *title)
	}

	body := elem.GetBody()
	if body != nil && len(body.Items) > 0 {
		sb.WriteString(" {\n")
		for _, item := range body.Items {
			e.writeBodyItem(sb, item, indent+"  ")
		}
		sb.WriteString(indent + "}\n")
	} else {
		sb.WriteString("\n")
	}
}

func (e *DSLExporter) writeBodyItem(sb *strings.Builder, item *language.LikeC4BodyItem, indent string) {
	if item.Description != nil {
		fmt.Fprintf(sb, "%sdescription %q\n", indent, *item.Description)
	}
	if item.Technology != nil {
		fmt.Fprintf(sb, "%stechnology %q\n", indent, *item.Technology)
	}
	if item.Element != nil {
		e.writeElement(sb, item.Element, indent)
	}
	if item.Relation != nil {
		e.writeRelation(sb, item.Relation, indent)
	}
}

func (e *DSLExporter) writeRelation(sb *strings.Builder, rel *language.Relation, indent string) {
	from := sanitizeRef(rel.From.String())
	to := sanitizeRef(rel.To.String())

	label := getString(rel.Label)
	if label == "" {
		label = getString(rel.Verb)
	}

	if label != "" {
		fmt.Fprintf(sb, "%s%s -> %s %q\n", indent, from, to, label)
	} else {
		fmt.Fprintf(sb, "%s%s -> %s\n", indent, from, to)
	}
}

func (e *DSLExporter) writeViews(sb *strings.Builder, program *language.Program) {
	sb.WriteString("views {\n")

	if program.Views != nil {
		for _, item := range program.Views.Items {
			if item.View == nil {
				continue
			}
			v := item.View
			viewName := "index"
			if v.Name != nil {
				viewName = *v.Name
			}
			fmt.Fprintf(sb, "  view %s {\n", sanitizeID(viewName))
			if v.Title != nil {
				fmt.Fprintf(sb, "    title %q\n", *v.Title)
			}
			if v.Of != nil {
				fmt.Fprintf(sb, "    of %s\n", v.Of.String())
			}
			// Write include/exclude rules if present
			if v.Body != nil {
				for _, item := range v.Body.Items {
					if item.Title != nil {
						fmt.Fprintf(sb, "    title %q\n", *item.Title)
					}
					if item.Include != nil {
						sb.WriteString("    include")
						for _, expr := range item.Include.Expressions {
							fmt.Fprintf(sb, " %s", expr.String())
						}
						sb.WriteString("\n")
					}
					if item.Exclude != nil {
						sb.WriteString("    exclude")
						for _, expr := range item.Exclude.Expressions {
							fmt.Fprintf(sb, " %s", expr.String())
						}
						sb.WriteString("\n")
					}
				}
			}
			sb.WriteString("  }\n")
		}
	}

	sb.WriteString("}\n")
}

// sanitizeID makes an ID valid for LikeC4 (alphanumeric + underscore)
func sanitizeID(id string) string {
	return strings.Map(func(r rune) rune {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || r == '_' {
			return r
		}
		return '_'
	}, id)
}

// sanitizeRef handles qualified references (e.g., System.Container)
func sanitizeRef(ref string) string {
	parts := strings.Split(ref, ".")
	for i, part := range parts {
		parts[i] = sanitizeID(part)
	}
	return strings.Join(parts, ".")
}

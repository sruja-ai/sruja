// pkg/language/printer.go
package language

import (
	"fmt"
	"strings"
)

// Printer prints an AST back to DSL format.
type Printer struct {
	IndentLevel int
}

// NewPrinter creates a new printer.
func NewPrinter() *Printer {
	return &Printer{IndentLevel: 0}
}

// indentCache holds pre-computed indentation strings
var indentCache = make([]string, 20)

func init() {
	for i := range indentCache {
		indentCache[i] = strings.Repeat("  ", i)
	}
}

func (p *Printer) indent() string {
	if p.IndentLevel < len(indentCache) {
		return indentCache[p.IndentLevel]
	}
	return strings.Repeat("  ", p.IndentLevel)
}

// Print prints a Program back to DSL format.
func (p *Printer) Print(program *Program) string {
	if program == nil {
		return ""
	}
	sb := &strings.Builder{}

	// Specification items
	if program.Specification != nil {
		p.PrintSpecification(sb, program.Specification)
	}

	// Model items
	if program.Model != nil {
		for _, item := range program.Model.Items {
			p.PrintModelItem(sb, item)
		}
	}

	// Views items
	if program.Views != nil {
		p.PrintViews(sb, program.Views)
	}

	return sb.String()
}

func (p *Printer) PrintSpecification(sb *strings.Builder, spec *Specification) {
	for _, item := range spec.Items {
		if item.Element != nil {
			p.PrintElementKindDef(sb, item.Element)
		}
		if item.Tag != nil {
			p.PrintTagDef(sb, item.Tag)
		}
	}
}

func (p *Printer) PrintElementKindDef(sb *strings.Builder, def *ElementKindDef) {
	fmt.Fprintf(sb, "%s = kind", def.Name)
	if def.Title != nil {
		fmt.Fprintf(sb, " %q", *def.Title)
	}
	sb.WriteString("\n")
}

func (p *Printer) PrintTagDef(sb *strings.Builder, def *TagDef) {
	fmt.Fprintf(sb, "%s = tag", def.Name)
	if def.Title != nil {
		fmt.Fprintf(sb, " %q", *def.Title)
	}
	sb.WriteString("\n")
}

func (p *Printer) PrintModelItem(sb *strings.Builder, item ModelItem) {
	if item.Import != nil {
		p.PrintImport(sb, item.Import)
	}
	if item.Relation != nil {
		p.PrintRelation(sb, item.Relation)
	}
	if item.ElementDef != nil {
		p.PrintElementDef(sb, item.ElementDef)
	}
}

func (p *Printer) PrintElementDef(sb *strings.Builder, elem *ElementDef) {
	indent := p.indent()

	if elem.Assignment == nil {
		return
	}

	fmt.Fprintf(sb, "%s%s = %s", indent, elem.Assignment.Name, elem.Assignment.Kind)

	if elem.Assignment.SubKind != nil {
		fmt.Fprintf(sb, " %s", *elem.Assignment.SubKind)
	}

	if elem.Assignment.Title != nil {
		fmt.Fprintf(sb, " %q", *elem.Assignment.Title)
	}

	body := elem.GetBody()
	if body != nil {
		sb.WriteString(" {\n")
		p.IndentLevel++
		for _, item := range body.Items {
			p.PrintBodyItem(sb, item)
		}
		p.IndentLevel--
		sb.WriteString(p.indent() + "}\n")
	} else {
		sb.WriteString("\n")
	}
}

func (p *Printer) PrintBodyItem(sb *strings.Builder, item *BodyItem) {
	indent := p.indent()
	if item.Description != nil {
		fmt.Fprintf(sb, "%sdescription %q\n", indent, *item.Description)
	}
	if item.Technology != nil {
		fmt.Fprintf(sb, "%stechnology %q\n", indent, *item.Technology)
	}
	if item.Element != nil {
		p.PrintElementDef(sb, item.Element)
	}
	if item.Relation != nil {
		p.PrintRelation(sb, item.Relation)
	}
	if item.Metadata != nil {
		p.PrintMetadataBlock(sb, item.Metadata)
	}
	if item.Scale != nil {
		p.PrintScale(sb, item.Scale)
	}
	if item.SLO != nil {
		p.PrintSLO(sb, item.SLO)
	}

}

func (p *Printer) PrintRelation(sb *strings.Builder, rel *Relation) {
	fmt.Fprintf(sb, "%s%s %s %s", p.indent(), rel.From, rel.Arrow, rel.To)
	if rel.Label != nil {
		fmt.Fprintf(sb, " %q", *rel.Label)
	}
	sb.WriteString("\n")
}

func (p *Printer) PrintViews(sb *strings.Builder, views *Views) {
	for _, item := range views.Items {
		if item.View != nil {
			p.PrintViewDef(sb, item.View)
		}
		if item.Styles != nil {
			p.PrintStyles(sb, item.Styles)
		}
	}
}

func (p *Printer) PrintViewDef(sb *strings.Builder, v *ViewDef) {
	indent := p.indent()
	fmt.Fprintf(sb, "%sview", indent)
	if v.Name != nil {
		fmt.Fprintf(sb, " %s", *v.Name)
	}
	if v.Of != nil {
		fmt.Fprintf(sb, " of %s", v.Of.String())
	}
	sb.WriteString(" {\n")
	p.IndentLevel++
	if v.Title != nil {
		fmt.Fprintf(sb, "%stitle %q\n", p.indent(), *v.Title)
	}
	if v.Body != nil {
		for _, item := range v.Body.Items {
			if item.Title != nil {
				fmt.Fprintf(sb, "%stitle %q\n", p.indent(), *item.Title)
			}
			if item.Include != nil && len(item.Include.Expressions) > 0 {
				exprs := make([]string, len(item.Include.Expressions))
				for i, expr := range item.Include.Expressions {
					exprs[i] = expr.String()
				}
				fmt.Fprintf(sb, "%sinclude %s\n", p.indent(), strings.Join(exprs, ", "))
			}
			if item.Exclude != nil && len(item.Exclude.Expressions) > 0 {
				exprs := make([]string, len(item.Exclude.Expressions))
				for i, expr := range item.Exclude.Expressions {
					exprs[i] = expr.String()
				}
				fmt.Fprintf(sb, "%sexclude %s\n", p.indent(), strings.Join(exprs, ", "))
			}
		}
	}
	p.IndentLevel--
	sb.WriteString(indent + "}\n")
}

func (p *Printer) PrintRequirement(sb *strings.Builder, req *Requirement) {
	fmt.Fprintf(sb, "%srequirement %s {\n", p.indent(), req.ID)
	// Body printing simplified for now
	sb.WriteString(p.indent() + "}\n")
}

func (p *Printer) PrintADR(sb *strings.Builder, adr *ADR) {
	fmt.Fprintf(sb, "%sadr %s {\n", p.indent(), adr.ID)
	// Body printing simplified for now
	sb.WriteString(p.indent() + "}\n")
}

func (p *Printer) PrintImport(sb *strings.Builder, imp *ImportStatement) {
	indent := p.indent()
	sb.WriteString(indent)
	sb.WriteString("import { ")
	for i, elem := range imp.Elements {
		if i > 0 {
			sb.WriteString(", ")
		}
		sb.WriteString(elem)
	}
	sb.WriteString(" } from ")
	_, _ = fmt.Fprintf(sb, "%q\n", imp.From)
}

func (p *Printer) PrintScenario(sb *strings.Builder, s *Scenario) {
	fmt.Fprintf(sb, "%sscenario %s {\n", p.indent(), s.ID)
	sb.WriteString(p.indent() + "}\n")
}

func (p *Printer) PrintScale(sb *strings.Builder, _ *ScaleBlock) {
	fmt.Fprintf(sb, "%sscale {\n", p.indent())
	sb.WriteString(p.indent() + "}\n")
}

func (p *Printer) PrintSLO(sb *strings.Builder, _ *SLOBlock) {
	fmt.Fprintf(sb, "%sslo {\n", p.indent())
	sb.WriteString(p.indent() + "}\n")
}

func (p *Printer) PrintStyles(sb *strings.Builder, s *StyleDecl) {
	fmt.Fprintf(sb, "%s%s {\n", p.indent(), s.Keyword)
	sb.WriteString(p.indent() + "}\n")
}

func (p *Printer) PrintMetadataBlock(sb *strings.Builder, _ *MetadataBlock) {
	fmt.Fprintf(sb, "%smetadata {\n", p.indent())
	sb.WriteString(p.indent() + "}\n")
}

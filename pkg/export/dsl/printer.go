// Package dsl provides functionality to convert SrujaModelDump to DSL format.
package dsl

import (
	"sort"
	"strings"

	"github.com/sruja-ai/sruja/pkg/export/json"
)

// ModelPrinter converts a SrujaModelDump to DSL string format.
type ModelPrinter struct {
	indentLevel int
	sb          strings.Builder
}

// NewModelPrinter creates a new ModelPrinter.
func NewModelPrinter() *ModelPrinter {
	return &ModelPrinter{}
}

// Print converts a SrujaModelDump to DSL format.
func (p *ModelPrinter) Print(model *json.SrujaModelDump) string {
	if model == nil {
		return ""
	}

	p.sb.Reset()
	p.indentLevel = 0

	// Use the flat printing logic directly
	topLevel, children := p.buildChildrenMap(model)

	for _, el := range topLevel {
		p.printElement(&el, children)
	}

	if len(model.Relations) > 0 {
		p.sb.WriteString("\n")
		for _, rel := range model.Relations {
			p.printRelation(&rel)
		}
	}

	// Print views
	if len(model.Views) > 0 {
		p.printViews(model)
	}

	// Print extensions (ADRs, etc)
	if model.Sruja != nil {
		p.printExtensions(model.Sruja)
	}

	return p.sb.String()
}

// Helper to build children map if we want to stick to the recursive printing strategy
func (p *ModelPrinter) buildChildrenMap(model *json.SrujaModelDump) (topLevel []json.ElementDump, children map[string][]json.ElementDump) {
	topLevel = []json.ElementDump{}
	children = make(map[string][]json.ElementDump)
	for _, el := range model.Elements {
		if el.Parent == "" {
			topLevel = append(topLevel, el)
		} else {
			children[el.Parent] = append(children[el.Parent], el)
		}
	}
	sort.Slice(topLevel, func(i, j int) bool {
		return topLevel[i].ID < topLevel[j].ID
	})
	return
}

func (p *ModelPrinter) PrintFlat(model *json.SrujaModelDump) string {
	if model == nil {
		return ""
	}
	p.sb.Reset()
	p.indentLevel = 0

	topLevel, children := p.buildChildrenMap(model)

	for _, el := range topLevel {
		p.printElement(&el, children)
	}

	if len(model.Relations) > 0 {
		p.sb.WriteString("\n")
		for _, rel := range model.Relations {
			p.printRelation(&rel)
		}
	}
	return p.sb.String()

}

func (p *ModelPrinter) indent() string {
	return strings.Repeat("  ", p.indentLevel)
}

func (p *ModelPrinter) writeLine(format string, args ...interface{}) {
	p.sb.WriteString(p.indent())
	if len(args) == 0 {
		p.sb.WriteString(format)
	} else {
		for i, arg := range args {
			format = strings.Replace(format, "%s", toString(arg), 1)
			_ = i
		}
		p.sb.WriteString(format)
	}
	p.sb.WriteString("\n")
}

func toString(v interface{}) string {
	if s, ok := v.(string); ok {
		return s
	}
	return ""
}

// escapeString escapes quotes and newlines for DSL output.
func escapeString(s string) string {
	s = strings.ReplaceAll(s, "\\", "\\\\")
	s = strings.ReplaceAll(s, "\"", "\\\"")
	s = strings.ReplaceAll(s, "\n", "\\n")
	return s
}

func (p *ModelPrinter) printElement(el *json.ElementDump, children map[string][]json.ElementDump) {
	// Extract short name from FQN
	name := extractName(el.ID)

	// Element header: name = kind "title" {
	p.sb.WriteString(p.indent())
	p.sb.WriteString(name)
	p.sb.WriteString(" = ")
	p.sb.WriteString(el.Kind)
	p.sb.WriteString(" \"")
	p.sb.WriteString(escapeString(el.Title))
	p.sb.WriteString("\" {\n")

	p.indentLevel++

	// Description
	if el.Description != "" {
		p.writeLine("description \"" + escapeString(el.Description) + "\"")
	}

	// Technology
	if el.Technology != "" {
		p.writeLine("technology \"" + escapeString(el.Technology) + "\"")
	}

	// Tags
	if len(el.Tags) > 0 {
		p.sb.WriteString(p.indent())
		p.sb.WriteString("tags [")
		for i, tag := range el.Tags {
			if i > 0 {
				p.sb.WriteString(", ")
			}
			p.sb.WriteString("\"")
			p.sb.WriteString(escapeString(tag))
			p.sb.WriteString("\"")
		}
		p.sb.WriteString("]\n")
	}

	// Metadata
	if len(el.Metadata) > 0 {
		p.writeLine("metadata {")
		p.indentLevel++
		for key, value := range el.Metadata {
			p.writeLine(key + " \"" + escapeString(value) + "\"")
		}
		p.indentLevel--
		p.writeLine("}")
	}

	// Links
	for _, link := range el.Links {
		if link.Title != "" {
			p.writeLine("link \"" + escapeString(link.URL) + "\" \"" + escapeString(link.Title) + "\"")
		} else {
			p.writeLine("link \"" + escapeString(link.URL) + "\"")
		}
	}

	// Nested children
	if kids, ok := children[el.ID]; ok {
		sort.Slice(kids, func(i, j int) bool {
			return kids[i].ID < kids[j].ID
		})
		for _, child := range kids {
			p.printElement(&child, children)
		}
	}

	p.indentLevel--
	p.writeLine("}")
}

func (p *ModelPrinter) printRelation(rel *json.RelationDump) {
	// Get source and target FQNs
	sourceFqn := rel.Source.Model
	targetFqn := rel.Target.Model

	if sourceFqn == "" || targetFqn == "" {
		return
	}

	// Determine arrow type
	arrow := "->"
	if rel.Kind == "bidirectional" {
		arrow = "<->"
	} else if rel.Kind == "back" {
		arrow = "<-"
	}

	p.sb.WriteString(p.indent())
	p.sb.WriteString(sourceFqn)
	p.sb.WriteString(" ")
	p.sb.WriteString(arrow)
	p.sb.WriteString(" ")
	p.sb.WriteString(targetFqn)

	// Title
	if rel.Title != "" {
		p.sb.WriteString(" \"")
		p.sb.WriteString(escapeString(rel.Title))
		p.sb.WriteString("\"")
	}

	// Technology
	if rel.Technology != "" {
		p.sb.WriteString(" technology \"")
		p.sb.WriteString(escapeString(rel.Technology))
		p.sb.WriteString("\"")
	}

	// Tags
	if len(rel.Tags) > 0 {
		p.sb.WriteString(" tags [")
		for i, tag := range rel.Tags {
			if i > 0 {
				p.sb.WriteString(", ")
			}
			p.sb.WriteString("\"")
			p.sb.WriteString(escapeString(tag))
			p.sb.WriteString("\"")
		}
		p.sb.WriteString("]")
	}

	p.sb.WriteString("\n")
}

func (p *ModelPrinter) printExtensions(ext *json.SrujaExtensions) {
	if ext == nil {
		return
	}

	hasExtensions := len(ext.Requirements) > 0 ||
		len(ext.Policies) > 0 ||
		len(ext.ADRs) > 0 ||
		len(ext.Scenarios) > 0 ||
		len(ext.Flows) > 0 ||
		len(ext.Constraints) > 0 ||
		len(ext.Conventions) > 0

	if !hasExtensions {
		return
	}

	p.sb.WriteString("\n")

	// Requirements
	for _, req := range ext.Requirements {
		p.printRequirement(&req)
	}

	// Policies
	for _, policy := range ext.Policies {
		p.printPolicy(&policy)
	}

	// ADRs
	for _, adr := range ext.ADRs {
		p.printADR(&adr)
	}

	// Scenarios
	for _, scenario := range ext.Scenarios {
		p.printScenario(&scenario)
	}

	// Flows
	for _, flow := range ext.Flows {
		p.printFlow(&flow)
	}

	// Constraints
	for _, constraint := range ext.Constraints {
		p.printConstraint(&constraint)
	}

	// Conventions
	for _, convention := range ext.Conventions {
		p.printConvention(&convention)
	}
}

func (p *ModelPrinter) printRequirement(req *json.RequirementDump) {
	typeStr := ""
	if req.Type != "" {
		typeStr = " " + req.Type
	}
	p.writeLine("requirement " + req.ID + typeStr + " \"" + escapeString(req.Title) + "\"")

	if req.Description != "" {
		p.indentLevel++
		p.writeLine("description \"" + escapeString(req.Description) + "\"")
		p.indentLevel--
	}
	if req.Priority != "" {
		p.indentLevel++
		p.writeLine("priority \"" + escapeString(req.Priority) + "\"")
		p.indentLevel--
	}
	if req.Status != "" {
		p.indentLevel++
		p.writeLine("status \"" + escapeString(req.Status) + "\"")
		p.indentLevel--
	}
	if len(req.Elements) > 0 {
		p.indentLevel++
		p.sb.WriteString(p.indent())
		p.sb.WriteString("elements [")
		p.sb.WriteString(strings.Join(req.Elements, ", "))
		p.sb.WriteString("]\n")
		p.indentLevel--
	}
}

func (p *ModelPrinter) printPolicy(policy *json.PolicyDump) {
	catStr := ""
	if policy.Category != "" {
		catStr = " " + policy.Category
	}
	p.writeLine("policy " + policy.ID + catStr + " \"" + escapeString(policy.Title) + "\"")

	if policy.Description != "" {
		p.indentLevel++
		p.writeLine("description \"" + escapeString(policy.Description) + "\"")
		p.indentLevel--
	}
	if policy.Enforcement != "" {
		p.indentLevel++
		p.writeLine("enforcement \"" + escapeString(policy.Enforcement) + "\"")
		p.indentLevel--
	}
	if len(policy.Elements) > 0 {
		p.indentLevel++
		p.sb.WriteString(p.indent())
		p.sb.WriteString("elements [")
		p.sb.WriteString(strings.Join(policy.Elements, ", "))
		p.sb.WriteString("]\n")
		p.indentLevel--
	}
}

func (p *ModelPrinter) printADR(adr *json.ADRDump) {
	p.writeLine("adr " + adr.ID + " \"" + escapeString(adr.Title) + "\" {")
	p.indentLevel++

	if adr.Status != "" {
		p.writeLine("status \"" + escapeString(adr.Status) + "\"")
	}
	if adr.Context != "" {
		p.writeLine("context \"" + escapeString(adr.Context) + "\"")
	}
	if adr.Decision != "" {
		p.writeLine("decision \"" + escapeString(adr.Decision) + "\"")
	}
	if adr.Consequences != "" {
		p.writeLine("consequences \"" + escapeString(adr.Consequences) + "\"")
	}
	if adr.Date != "" {
		p.writeLine("date \"" + escapeString(adr.Date) + "\"")
	}
	if adr.Author != "" {
		p.writeLine("author \"" + escapeString(adr.Author) + "\"")
	}

	p.indentLevel--
	p.writeLine("}")
}

func (p *ModelPrinter) printScenario(scenario *json.ScenarioDump) {
	p.writeLine("scenario " + scenario.ID + " \"" + escapeString(scenario.Title) + "\" {")
	p.indentLevel++

	if scenario.Description != "" {
		p.writeLine("description \"" + escapeString(scenario.Description) + "\"")
	}

	for _, step := range scenario.Steps {
		p.printStep(&step)
	}

	p.indentLevel--
	p.writeLine("}")
}

func (p *ModelPrinter) printFlow(flow *json.FlowDump) {
	p.writeLine("flow " + flow.ID + " \"" + escapeString(flow.Title) + "\" {")
	p.indentLevel++

	if flow.Description != "" {
		p.writeLine("description \"" + escapeString(flow.Description) + "\"")
	}

	for _, step := range flow.Steps {
		p.printStep(&step)
	}

	p.indentLevel--
	p.writeLine("}")
}

func (p *ModelPrinter) printStep(step *json.StepDump) {
	p.sb.WriteString(p.indent())
	p.sb.WriteString("step \"")
	p.sb.WriteString(escapeString(step.Description))
	p.sb.WriteString("\"")

	if step.From != "" && step.To != "" {
		p.sb.WriteString(" from ")
		p.sb.WriteString(step.From)
		p.sb.WriteString(" to ")
		p.sb.WriteString(step.To)
	}

	p.sb.WriteString("\n")
}

func (p *ModelPrinter) printConstraint(constraint *json.ConstraintDump) {
	typeStr := ""
	if constraint.Type != "" {
		typeStr = " " + constraint.Type
	}
	p.writeLine("constraint " + constraint.ID + typeStr + " \"" + escapeString(constraint.Description) + "\"")
}

func (p *ModelPrinter) printConvention(convention *json.ConventionDump) {
	p.writeLine("convention " + convention.ID + " \"" + escapeString(convention.Description) + "\"")
}

func (p *ModelPrinter) printViews(model *json.SrujaModelDump) {
	if len(model.Views) == 0 {
		return
	}

	p.sb.WriteString("\n")

	// Sort views for consistent output
	viewIDs := make([]string, 0, len(model.Views))
	for id := range model.Views {
		viewIDs = append(viewIDs, id)
	}
	sort.Strings(viewIDs)

	for _, id := range viewIDs {
		view := model.Views[id]
		p.printView(&view)
	}
}

func (p *ModelPrinter) printView(view *json.ViewDump) {
	p.sb.WriteString(p.indent())
	p.sb.WriteString("view ")
	p.sb.WriteString(view.ID)

	if view.ViewOf != "" {
		p.sb.WriteString(" of ")
		p.sb.WriteString(view.ViewOf)
	}

	p.sb.WriteString(" {\n")
	p.indentLevel++

	if view.Title != "" {
		p.writeLine("title \"" + escapeString(view.Title) + "\"")
	}

	if view.Description != "" {
		p.writeLine("description \"" + escapeString(view.Description) + "\"")
	}

	// View rules
	for _, rule := range view.Rules {
		if rule.Include != nil {
			if rule.Include.Wildcard {
				p.writeLine("include *")
			} else {
				for _, el := range rule.Include.Elements {
					p.writeLine("include " + el)
				}
			}
		}
		if rule.Exclude != nil {
			if rule.Exclude.Wildcard {
				p.writeLine("exclude *")
			} else {
				for _, el := range rule.Exclude.Elements {
					p.writeLine("exclude " + el)
				}
			}
		}
	}

	// Tags
	if len(view.Tags) > 0 {
		p.sb.WriteString(p.indent())
		p.sb.WriteString("tags [")
		for i, tag := range view.Tags {
			if i > 0 {
				p.sb.WriteString(", ")
			}
			p.sb.WriteString("\"")
			p.sb.WriteString(escapeString(tag))
			p.sb.WriteString("\"")
		}
		p.sb.WriteString("]\n")
	}

	p.indentLevel--
	p.writeLine("}")
}

// extractName gets the short name from a fully qualified ID.
func extractName(id string) string {
	parts := strings.Split(id, ".")
	return parts[len(parts)-1]
}

// Print is a convenience function to print a model to DSL.
func Print(model *json.SrujaModelDump) string {
	p := NewModelPrinter()
	return p.Print(model)
}

package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
	exportpkg "github.com/sruja-ai/sruja/pkg/export"
	"github.com/sruja-ai/sruja/pkg/export/mermaid"
	"github.com/sruja-ai/sruja/pkg/language"
)

// Exporter handles Markdown document generation.
type Exporter struct {
	Options Options
}

// NewExporter creates a new Markdown exporter.
func NewExporter(options Options) *Exporter {
	return &Exporter{Options: options}
}

// Export generates a Markdown document from a program.
func (e *Exporter) Export(prog *language.Program) string {
	if prog == nil || prog.Model == nil {
		return ""
	}

	// Apply scoping if specified
	if e.Options.Scope != nil && e.Options.Scope.Type != "full" {
		prog = exportpkg.FilterByScope(prog, e.Options.Scope.Type, e.Options.Scope.ID)
		if prog == nil || prog.Model == nil {
			return ""
		}
	}

	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	// Extract elements from Model
	systems := extractSystemsFromModel(prog)
	persons := extractPersonsFromModel(prog)
	requirements := extractRequirementsFromModel(prog)
	adrs := extractADRsFromModel(prog)

	// Apply token optimization if needed
	optimizer := exportpkg.NewTokenOptimizer(e.Options.TokenLimit)

	// Build initial content to estimate tokens
	initialContent := e.buildContent(systems, persons, requirements, adrs, prog)

	if optimizer.ShouldOptimize(exportpkg.EstimateTokens(initialContent)) {
		// Apply optimizations
		systems, persons, requirements, adrs = e.optimizeContent(
			optimizer, systems, persons, requirements, adrs, prog)
	}

	// Build content based on context type
	e.writeContent(sb, systems, persons, requirements, adrs, prog)

	result := sb.String()

	// Final token check and truncation if needed
	if optimizer.ShouldOptimize(exportpkg.EstimateTokens(result)) {
		result = e.truncateToLimit(result, e.Options.TokenLimit)
	}

	return result
}

// buildContent builds initial content for token estimation
func (e *Exporter) buildContent(systems []*language.System, persons []*language.Person,
	requirements []*language.Requirement, adrs []*language.ADR, prog *language.Program) string {
	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	e.writeContent(sb, systems, persons, requirements, adrs, prog)
	return sb.String()
}

// writeContent writes the main content based on context
func (e *Exporter) writeContent(sb *strings.Builder, systems []*language.System,
	persons []*language.Person, requirements []*language.Requirement,
	adrs []*language.ADR, prog *language.Program) {

	arch := &struct {
		Name         string
		Description  *string
		Systems      []*language.System
		Persons      []*language.Person
		Requirements []*language.Requirement
		ADRs         []*language.ADR
	}{
		Name:         "Architecture",
		Systems:      systems,
		Persons:      persons,
		Requirements: requirements,
		ADRs:         adrs,
	}

	// Header
	title := arch.Name
	fmt.Fprintf(sb, "# %s\n\n", title)
	if arch.Description != nil {
		fmt.Fprintf(sb, "%s\n\n", *arch.Description)
	}

	// TOC
	if e.Options.IncludeTOC {
		e.writeTOC(sb, arch)
	}

	// Overview
	if e.Options.IncludeOverview {
		e.writeOverview(sb, prog)
	}

	// Adjust content based on context type
	switch e.Options.Context {
	case ContextCodeGeneration:
		e.writeContentForCodeGeneration(sb, arch, prog)
	case ContextReview:
		e.writeContentForReview(sb, arch, prog)
	case ContextAnalysis:
		e.writeContentForAnalysis(sb, arch, prog)
	default:
		e.writeContentDefault(sb, arch, prog)
	}
}

// writeContentDefault writes default balanced content
func (e *Exporter) writeContentDefault(sb *strings.Builder, arch interface{}, _ *language.Program) {
	if e.Options.IncludeSystems {
		e.writeSystems(sb, arch)
	}
	if e.Options.IncludePersons {
		e.writePersons(sb, arch)
	}
	if e.Options.IncludeRequirements {
		e.writeRequirements(sb, arch)
	}
	if e.Options.IncludeADRs {
		e.writeADRs(sb, arch)
	}
}

// writeContentForCodeGeneration emphasizes technology and relationships
func (e *Exporter) writeContentForCodeGeneration(sb *strings.Builder, arch interface{}, _ *language.Program) {
	sb.WriteString("## Technology Stack\n\n")
	e.writeSystems(sb, arch) // Systems include technology info
	if e.Options.IncludeRequirements {
		e.writeRequirements(sb, arch)
	}
	if e.Options.IncludeADRs {
		e.writeADRs(sb, arch)
	}
}

// writeContentForReview emphasizes ADRs and requirements
func (e *Exporter) writeContentForReview(sb *strings.Builder, arch interface{}, _ *language.Program) {
	if e.Options.IncludeADRs {
		e.writeADRs(sb, arch)
	}
	if e.Options.IncludeRequirements {
		e.writeRequirements(sb, arch)
	}
	if e.Options.IncludeSystems {
		e.writeSystems(sb, arch)
	}
}

// writeContentForAnalysis emphasizes relationships and data flows
func (e *Exporter) writeContentForAnalysis(sb *strings.Builder, arch interface{}, prog *language.Program) {
	if e.Options.IncludeSystems {
		e.writeSystems(sb, arch)
	}
	// Add relationship section for analysis context
	e.writeRelationships(sb, prog)
	if e.Options.IncludeRequirements {
		e.writeRequirements(sb, arch)
	}
}

// optimizeContent applies token optimizations
func (e *Exporter) optimizeContent(optimizer *exportpkg.TokenOptimizer,
	systems []*language.System, persons []*language.Person,
	requirements []*language.Requirement, adrs []*language.ADR,
	_ *language.Program) ([]*language.System, []*language.Person, []*language.Requirement, []*language.ADR) {

	// Truncate descriptions
	for _, sys := range systems {
		if sys.Description != nil {
			truncated := optimizer.TruncateDescription(*sys.Description, 50)
			sys.Description = &truncated
		}
	}

	// Limit requirements and ADRs if needed
	if len(requirements) > 10 {
		requirements = requirements[:10]
	}
	if len(adrs) > 5 {
		adrs = adrs[:5]
	}

	return systems, persons, requirements, adrs
}

// truncateToLimit truncates content to fit token limit
func (e *Exporter) truncateToLimit(content string, limit int) string {
	if limit <= 0 {
		return content
	}
	maxChars := limit * 4
	if len(content) <= maxChars {
		return content
	}
	// Try to truncate at a reasonable boundary
	truncated := content[:maxChars-50]
	if lastNewline := strings.LastIndex(truncated, "\n\n"); lastNewline > maxChars/2 {
		truncated = truncated[:lastNewline]
	}
	return truncated + "\n\n... (content truncated to fit token limit)"
}

// Helper functions to extract elements from Model
func extractSystemsFromModel(prog *language.Program) []*language.System {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var systems []*language.System
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "system" {
			if sys := extractSystemFromElement(item.ElementDef); sys != nil {
				systems = append(systems, sys)
			}
		}
	}
	return systems
}

func extractSystemFromElement(elem *language.LikeC4ElementDef) *language.System {
	if elem == nil || elem.GetKind() != "system" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	sys := &language.System{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
	body := elem.GetBody()
	if body != nil {
		for _, bodyItem := range body.Items {
			if bodyItem.Description != nil {
				sys.Description = bodyItem.Description
			}
			if bodyItem.Element != nil {
				if cont := extractContainerFromElement(bodyItem.Element); cont != nil {
					sys.Containers = append(sys.Containers, cont)
				}
				if ds := extractDataStoreFromElement(bodyItem.Element); ds != nil {
					sys.DataStores = append(sys.DataStores, ds)
				}
				if q := extractQueueFromElement(bodyItem.Element); q != nil {
					sys.Queues = append(sys.Queues, q)
				}
			}
		}
	}
	return sys
}

func extractContainerFromElement(elem *language.LikeC4ElementDef) *language.Container {
	if elem == nil || elem.GetKind() != "container" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	cont := &language.Container{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
	body := elem.GetBody()
	if body != nil {
		for _, bodyItem := range body.Items {
			if bodyItem.Description != nil {
				cont.Description = bodyItem.Description
			}
			if bodyItem.Element != nil {
				if comp := extractComponentFromElement(bodyItem.Element); comp != nil {
					cont.Components = append(cont.Components, comp)
				}
			}
		}
	}
	return cont
}

func extractComponentFromElement(elem *language.LikeC4ElementDef) *language.Component {
	if elem == nil || elem.GetKind() != "component" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	return &language.Component{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
}

func extractDataStoreFromElement(elem *language.LikeC4ElementDef) *language.DataStore {
	if elem == nil {
		return nil
	}
	kind := elem.GetKind()
	if kind != "database" && kind != "datastore" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	return &language.DataStore{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
}

func extractQueueFromElement(elem *language.LikeC4ElementDef) *language.Queue {
	if elem == nil || elem.GetKind() != "queue" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	return &language.Queue{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
}

func extractPersonsFromModel(prog *language.Program) []*language.Person {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var persons []*language.Person
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "person" {
			id := item.ElementDef.GetID()
			if id != "" {
				persons = append(persons, &language.Person{
					ID:    id,
					Label: getString(item.ElementDef.GetTitle()),
				})
			}
		}
	}
	return persons
}

func extractRequirementsFromModel(prog *language.Program) []*language.Requirement {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var requirements []*language.Requirement
	for _, item := range prog.Model.Items {
		if item.Requirement != nil {
			requirements = append(requirements, item.Requirement)
		}
	}
	return requirements
}

func extractADRsFromModel(prog *language.Program) []*language.ADR {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var adrs []*language.ADR
	for _, item := range prog.Model.Items {
		if item.ADR != nil {
			adrs = append(adrs, item.ADR)
		}
	}
	return adrs
}

func getString(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

func (e *Exporter) writeTOC(sb *strings.Builder, arch interface{}) {
	sb.WriteString("## Table of Contents\n\n")
	sb.WriteString("- [Architecture Overview](#architecture-overview)\n")
	archStruct := arch.(*struct {
		Name         string
		Description  *string
		Systems      []*language.System
		Persons      []*language.Person
		Requirements []*language.Requirement
		ADRs         []*language.ADR
	})
	if len(archStruct.Systems) > 0 {
		sb.WriteString("- [Systems](#systems)\n")
	}
	if len(archStruct.Persons) > 0 {
		sb.WriteString("- [Persons](#persons)\n")
	}
	if len(archStruct.Requirements) > 0 {
		sb.WriteString("- [Requirements](#requirements)\n")
	}
	if len(archStruct.ADRs) > 0 {
		sb.WriteString("- [Architecture Decision Records (ADRs)](#architecture-decision-records-adrs)\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeOverview(sb *strings.Builder, prog *language.Program) {
	sb.WriteString("## Architecture Overview\n\n")

	mermaidExporter := mermaid.NewExporter(e.Options.MermaidConfig)
	diagram := mermaidExporter.Export(prog)

	if diagram != "" {
		sb.WriteString("```mermaid\n")
		sb.WriteString(diagram)
		sb.WriteString("\n```\n\n")
	}
}

func (e *Exporter) writeSystems(sb *strings.Builder, arch interface{}) {
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

	sb.WriteString("## Systems\n\n")
	for _, sys := range archStruct.Systems {
		fmt.Fprintf(sb, "### %s\n\n", sys.Label)
		if sys.Description != nil {
			fmt.Fprintf(sb, "%s\n\n", *sys.Description)
		}

		// Note: Detailed system diagrams (L2/L3) can be added here
		// as we improve the Mermaid exporter to handle specific system views.
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

	sb.WriteString("## Persons\n\n")
	for _, p := range archStruct.Persons {
		desc := ""
		if p.Description != nil {
			desc = *p.Description
		}
		fmt.Fprintf(sb, "- **%s**: %s\n", p.Label, desc)
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeRequirements(sb *strings.Builder, arch interface{}) {
	archStruct := arch.(*struct {
		Name         string
		Description  *string
		Systems      []*language.System
		Persons      []*language.Person
		Requirements []*language.Requirement
		ADRs         []*language.ADR
	})
	if len(archStruct.Requirements) == 0 {
		return
	}

	sb.WriteString("## Requirements\n\n")
	for _, req := range archStruct.Requirements {
		desc := ""
		if req.Description != nil {
			desc = *req.Description
		}
		fmt.Fprintf(sb, "- **%s**: %s\n", req.ID, desc)
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeADRs(sb *strings.Builder, arch interface{}) {
	archStruct := arch.(*struct {
		Name         string
		Description  *string
		Systems      []*language.System
		Persons      []*language.Person
		Requirements []*language.Requirement
		ADRs         []*language.ADR
	})
	if len(archStruct.ADRs) == 0 {
		return
	}

	sb.WriteString("## Architecture Decision Records (ADRs)\n\n")
	for _, adr := range archStruct.ADRs {
		title := adr.ID
		if adr.Title != nil {
			title = *adr.Title
		}
		fmt.Fprintf(sb, "### %s\n\n", title)
		if adr.Body != nil {
			if adr.Body.Status != nil {
				fmt.Fprintf(sb, "**Status**: %s\n\n", *adr.Body.Status)
			}
			if adr.Body.Context != nil {
				fmt.Fprintf(sb, "**Context**:\n%s\n\n", *adr.Body.Context)
			}
			if adr.Body.Decision != nil {
				fmt.Fprintf(sb, "**Decision**:\n%s\n\n", *adr.Body.Decision)
			}
		}
	}
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

	sb.WriteString("## Key Relationships\n\n")

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

// extractRelationsFromModel extracts all relations from the model
func extractRelationsFromModel(prog *language.Program) []*language.Relation {
	if prog == nil || prog.Model == nil {
		return nil
	}

	var relations []*language.Relation

	// Collect top-level relations
	for _, item := range prog.Model.Items {
		if item.Relation != nil {
			relations = append(relations, item.Relation)
		}
	}

	// Collect relations from element bodies
	var collectFromElement func(elem *language.LikeC4ElementDef)
	collectFromElement = func(elem *language.LikeC4ElementDef) {
		if elem == nil {
			return
		}
		body := elem.GetBody()
		if body == nil {
			return
		}

		for _, bodyItem := range body.Items {
			if bodyItem.Relation != nil {
				relations = append(relations, bodyItem.Relation)
			}
			if bodyItem.Element != nil {
				collectFromElement(bodyItem.Element)
			}
		}
	}

	for _, item := range prog.Model.Items {
		if item.ElementDef != nil {
			collectFromElement(item.ElementDef)
		}
	}

	return relations
}

// prioritizeRelationships sorts relationships by importance
// Priority: direct relationships > relationships with labels > simple references
func prioritizeRelationships(relations []*language.Relation) []*language.Relation {
	if len(relations) == 0 {
		return relations
	}

	// Create a copy to avoid modifying original
	prioritized := make([]*language.Relation, len(relations))
	copy(prioritized, relations)

	// Simple prioritization: relationships with labels/verbs are more important
	// This is a basic implementation - can be enhanced with more sophisticated scoring
	for i := 0; i < len(prioritized)-1; i++ {
		for j := i + 1; j < len(prioritized); j++ {
			scoreI := relationshipScore(prioritized[i])
			scoreJ := relationshipScore(prioritized[j])
			if scoreJ > scoreI {
				prioritized[i], prioritized[j] = prioritized[j], prioritized[i]
			}
		}
	}

	return prioritized
}

// relationshipScore calculates importance score for a relationship
func relationshipScore(rel *language.Relation) int {
	score := 0

	// Relationships with labels are more important
	if rel.Label != nil && *rel.Label != "" {
		score += 10
	}

	// Relationships with verbs are more important
	if rel.Verb != nil && *rel.Verb != "" {
		score += 5
	}
	if rel.VerbRaw != nil && rel.VerbRaw.Value != "" {
		score += 5
	}

	// Relationships with tags are more important
	if len(rel.Tags) > 0 {
		score += len(rel.Tags) * 2
	}

	return score
}

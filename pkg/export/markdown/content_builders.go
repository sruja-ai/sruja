package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
	exportpkg "github.com/sruja-ai/sruja/pkg/export"
	"github.com/sruja-ai/sruja/pkg/language"
)

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
	if title == "" {
		title = "System Architecture Documentation"
	}
	fmt.Fprintf(sb, "# %s\n\n", title)

	// Executive Summary
	sb.WriteString("## Executive Summary\n\n")
	if arch.Description != nil {
		fmt.Fprintf(sb, "%s\n\n", *arch.Description)
	} else {
		sb.WriteString("This document provides a comprehensive overview of the system architecture, including system components, user roles, functional requirements, and key architectural decisions.\n\n")
	}

	// Metadata section
	if e.Options.IncludeMetadata {
		e.writeMetadata(sb, prog)
	}

	// Summary Statistics
	e.writeSummaryStatistics(sb, systems, persons, requirements, adrs, prog)

	// TOC
	if e.Options.IncludeTOC {
		e.writeTOC(sb, arch, prog)
	}

	// Overview
	if e.Options.IncludeOverview {
		e.writeOverview(sb, prog)
	}

	// Technology Stack Summary (before systems for context)
	if e.Options.IncludeSystems {
		e.writeTechnologyStackSummary(sb, systems)
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

	// Scenarios and Flows section
	if e.Options.IncludeScenarios {
		e.writeScenariosAndFlows(sb, prog)
	}

	// Glossary section
	if e.Options.IncludeGlossary {
		e.writeGlossary(sb, prog)
	}

	// Add recommendations section if enabled
	if e.Options.IncludeRecommendations {
		e.writeRecommendations(sb, prog)
	}
}

// writeContentDefault writes default balanced content
func (e *Exporter) writeContentDefault(sb *strings.Builder, arch interface{}, prog *language.Program) {
	if e.Options.IncludeSystems {
		e.writeSystems(sb, arch, prog)
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
func (e *Exporter) writeContentForCodeGeneration(sb *strings.Builder, arch interface{}, prog *language.Program) {
	sb.WriteString("## Technology Stack\n\n")
	e.writeSystems(sb, arch, prog) // Systems include technology info
	if e.Options.IncludeRequirements {
		e.writeRequirements(sb, arch)
	}
	if e.Options.IncludeADRs {
		e.writeADRs(sb, arch)
	}
}

// writeContentForReview emphasizes ADRs and requirements
func (e *Exporter) writeContentForReview(sb *strings.Builder, arch interface{}, prog *language.Program) {
	if e.Options.IncludeADRs {
		e.writeADRs(sb, arch)
	}
	if e.Options.IncludeRequirements {
		e.writeRequirements(sb, arch)
	}
	if e.Options.IncludeSystems {
		e.writeSystems(sb, arch, prog)
	}
}

// writeContentForAnalysis emphasizes relationships and data flows
func (e *Exporter) writeContentForAnalysis(sb *strings.Builder, arch interface{}, prog *language.Program) {
	if e.Options.IncludeSystems {
		e.writeSystems(sb, arch, prog)
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

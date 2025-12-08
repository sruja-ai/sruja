// pkg/export/markdown/markdown.go
// Package markdown provides Markdown export with embedded Mermaid diagrams.
//
//nolint:gocritic // Use WriteString for consistency
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

type Exporter struct {
	Options ExportOptions
}

func NewExporter() *Exporter {
	return &Exporter{
		Options: DefaultExportOptions(),
	}
}

// NewExporterWithOptions creates an exporter with custom options
func NewExporterWithOptions(options ExportOptions) *Exporter {
	return &Exporter{
		Options: options,
	}
}

// Export converts an Architecture AST to Markdown format with embedded Mermaid diagrams.
// Uses template-based rendering for better maintainability and separation of concerns.
func (e *Exporter) Export(arch *language.Architecture) (string, error) {
	if arch == nil {
		return "", fmt.Errorf("architecture is nil")
	}
	// Extract Mermaid config from style/metadata, then merge with options
	mermaidConfig := extractMermaidConfig(arch)
	// Merge with options (options take precedence)
	if e.Options.MermaidConfig.Direction != "" {
		mermaidConfig.Direction = e.Options.MermaidConfig.Direction
	}
	if e.Options.MermaidConfig.Theme != "" {
		mermaidConfig.Theme = e.Options.MermaidConfig.Theme
	}
	if e.Options.MermaidConfig.Layout != "" {
		mermaidConfig.Layout = e.Options.MermaidConfig.Layout
	}
	if e.Options.MermaidConfig.Look != "" {
		mermaidConfig.Look = e.Options.MermaidConfig.Look
	}
	// Frontmatter setting from options takes precedence
	if e.Options.MermaidConfig.UseFrontmatter {
		mermaidConfig.UseFrontmatter = true
	}

	// Build document data with all pre-rendered sections
	docData := e.buildDocumentData(arch, mermaidConfig)

	// Load and execute main document template
	tmpl, err := loadTemplate("main")
	if err != nil {
		// Fallback to non-template approach if template loading fails
		return e.exportLegacy(arch, mermaidConfig)
	}

	return executeTemplate(tmpl, docData)
}

// exportLegacy is a fallback non-template implementation (for backwards compatibility)
//
//nolint:funlen,gocyclo // Legacy export is complex
func (e *Exporter) exportLegacy(arch *language.Architecture, mermaidConfig MermaidConfig) (string, error) {
	var sb strings.Builder

	// Write header
	sb.WriteString(fmt.Sprintf("# %s\n\n", arch.Name))
	if arch.Description != nil {
		sb.WriteString(fmt.Sprintf("%s\n\n", *arch.Description))
	}

	// Table of Contents
	e.writeTOC(&sb, arch)

	// Write executive summary
	e.writeExecutiveSummary(&sb, arch)

	// Write architecture overview diagram
	// Check if custom views are defined, otherwise use standard C4 views
	if arch.Views != nil && len(arch.Views.Views) > 0 {
		// Use custom views from DSL
		for _, view := range arch.Views.Views {
			if view.Type == "systemContext" {
				sb.WriteString(fmt.Sprintf("## %s\n\n", strings.Trim(view.Name, "\"")))
				diagram := generateSystemDiagram(arch, mermaidConfig)
				sb.WriteString(fmt.Sprintf("```mermaid\n%s\n```\n\n", diagram))
			}
		}
	} else {
		// Standard C4 views (automatic)
		if len(arch.Systems) > 0 || len(arch.Persons) > 0 {
			sb.WriteString("## Architecture Overview (C4 L1)\n\n")
			diagram := generateSystemDiagram(arch, mermaidConfig)
			sb.WriteString(fmt.Sprintf("```mermaid\n%s\n```\n\n", diagram))
		}
	}

	// Write systems section
	if len(arch.Systems) > 0 {
		sb.WriteString("## Systems\n\n")
		for _, sys := range arch.Systems {
			e.writeSystem(&sb, sys, arch, mermaidConfig)
		}
	}

	// Write deployment section with Architecture diagrams
	if len(arch.DeploymentNodes) > 0 {
		sb.WriteString("## Deployment Architecture\n\n")
		for _, deployment := range arch.DeploymentNodes {
			e.writeDeployment(&sb, deployment, mermaidConfig)
		}
	}

	// Write persons section
	if len(arch.Persons) > 0 {
		sb.WriteString("## Persons\n\n")
		for _, person := range arch.Persons {
			e.writePerson(&sb, person)
		}
	}

	// Write requirements section
	if len(arch.Requirements) > 0 {
		sb.WriteString("## Requirements\n\n")
		e.writeRequirementsGrouped(&sb, arch.Requirements)
	}

	// Write ADRs section
	if len(arch.ADRs) > 0 {
		sb.WriteString("## Architecture Decision Records (ADRs)\n\n")
		for _, adr := range arch.ADRs {
			e.writeADR(&sb, adr)
		}
	}

	// Write scenarios section
	if len(arch.Scenarios) > 0 {
		sb.WriteString("## Scenarios\n\n")
		for _, scenario := range arch.Scenarios {
			e.writeScenario(&sb, scenario, mermaidConfig)
		}
	}

	// Write policies section
	if len(arch.Policies) > 0 {
		sb.WriteString("## Policies\n\n")
		for _, policy := range arch.Policies {
			e.writePolicy(&sb, policy)
		}
	}

	// Write constraints section
	if len(arch.Constraints) > 0 {
		sb.WriteString("## Constraints\n\n")
		for _, c := range arch.Constraints {
			sb.WriteString(fmt.Sprintf("- **%s**: %s\n", c.Key, c.Value))
		}
		sb.WriteString("\n")
	}

	// Write conventions section
	if len(arch.Conventions) > 0 {
		sb.WriteString("## Conventions\n\n")
		for _, c := range arch.Conventions {
			sb.WriteString(fmt.Sprintf("- **%s**: %s\n", c.Key, c.Value))
		}
		sb.WriteString("\n")
	}

	// Write flows section
	if len(arch.Flows) > 0 {
		sb.WriteString("## Flows\n\n")
		for _, flow := range arch.Flows {
			e.writeFlow(&sb, flow)
		}
	}

	// Write contracts section
	if len(arch.Contracts) > 0 {
		sb.WriteString("## Integration Contracts\n\n")
		e.writeContracts(&sb, arch.Contracts)
	}

	// Write data consistency section
	e.writeDataConsistency(&sb, arch)

	// Write failure modes section
	e.writeFailureModes(&sb, arch)

	// Write relations section
	e.writeRelations(&sb, arch)

	// Write quality attributes derived from requirements and properties
	e.writeQualityAttributes(&sb, arch)

	// Write security section derived from requirements and policies
	e.writeSecurity(&sb, arch)

	// Write metadata section
	e.writeMetadata(&sb, arch)

	// Write glossary
	e.writeGlossary(&sb, arch)

	// Write Phase 2 sections
	e.writeCapacityPlanning(&sb, arch)
	e.writeMonitoringObservability(&sb, arch)
	e.writeSecurityThreatModel(&sb, arch)
	e.writeComplianceMatrix(&sb, arch)
	e.writeDependencyRiskAssessment(&sb, arch)

	// Write Phase 3 sections
	e.writeCostAnalysis(&sb, arch)
	e.writeAPIVersioning(&sb, arch)
	e.writeMultiRegionConsiderations(&sb, arch)
	e.writeDataLifecycleManagement(&sb, arch)

	// Add branding footer
	sb.WriteString("\n---\n\n")
	sb.WriteString("*Powered by [Sruja](https://sruja.ai) - Architecture as Code*\n")

	return sb.String(), nil
}

func (e *Exporter) writeSystem(sb *strings.Builder, sys *language.System, arch *language.Architecture, config MermaidConfig) {
	writeSystemHeader(sb, sys)
	writeSystemContainerDiagram(sb, sys, arch, config)
	writeSystemComponentDiagrams(sb, sys, arch, config)
	e.writeSystemElements(sb, sys)
	sb.WriteString("\n")
}

func (e *Exporter) writeContainer(sb *strings.Builder, cont *language.Container) {
	writeElement(sb, cont.ID, cont.Label, cont.Description)
}

func (e *Exporter) writeComponent(sb *strings.Builder, comp *language.Component) {
	writeElement(sb, comp.ID, comp.Label, comp.Description)
}

func (e *Exporter) writeDataStore(sb *strings.Builder, ds *language.DataStore) {
	writeElement(sb, ds.ID, ds.Label, ds.Description)
}

func (e *Exporter) writeQueue(sb *strings.Builder, q *language.Queue) {
	writeElement(sb, q.ID, q.Label, q.Description)
}

func (e *Exporter) writePerson(sb *strings.Builder, person *language.Person) {
	sb.WriteString(fmt.Sprintf("### %s\n\n", person.Label))
	if person.Description != nil {
		sb.WriteString(fmt.Sprintf("%s\n\n", *person.Description))
	}
}

func (e *Exporter) writeRequirement(sb *strings.Builder, req *language.Requirement) {
	sb.WriteString(fmt.Sprintf("- %s: %s\n", req.ID, strVal(req.Description)))
}

func (e *Exporter) writeADR(sb *strings.Builder, adr *language.ADR) {
	title := adr.ID
	if adr.Title != nil {
		title = fmt.Sprintf("%s: %s", adr.ID, *adr.Title)
	}
	sb.WriteString(fmt.Sprintf("### %s\n\n", title))
	if adr.Body != nil {
		if adr.Body.Status != nil {
			sb.WriteString(fmt.Sprintf("**Status**: %s\n\n", *adr.Body.Status))
		}
		if adr.Body.Context != nil {
			sb.WriteString(fmt.Sprintf("**Context**: %s\n\n", *adr.Body.Context))
		}
		if adr.Body.Decision != nil {
			sb.WriteString(fmt.Sprintf("**Decision**: %s\n\n", *adr.Body.Decision))
		}
		if adr.Body.Consequences != nil {
			sb.WriteString(fmt.Sprintf("**Consequences**: %s\n\n", *adr.Body.Consequences))
		}
	}
}

func (e *Exporter) writeDeployment(sb *strings.Builder, deployment *language.DeploymentNode, config MermaidConfig) {
	writeDeploymentHeader(sb, deployment)
	writeDeploymentDiagram(sb, deployment, config)
	writeContainerInstances(sb, deployment.ContainerInstances)

	// Recursively write child deployment nodes
	for _, child := range deployment.Children {
		e.writeDeployment(sb, child, config)
	}
}

func (e *Exporter) writeScenario(sb *strings.Builder, scenario *language.Scenario, config MermaidConfig) {
	sb.WriteString(fmt.Sprintf("### %s\n\n", scenario.Title))
	if scenario.Description != nil {
		sb.WriteString(fmt.Sprintf("%s\n\n", *scenario.Description))
	}

	// Generate sequence diagram for scenario
	if len(scenario.Steps) > 0 {
		diagram := generateScenarioDiagram(scenario, config)
		sb.WriteString(fmt.Sprintf("```mermaid\n%s\n```\n\n", diagram))
	}

	// Write steps as list
	if len(scenario.Steps) > 0 {
		sb.WriteString("#### Steps\n\n")
		for i, step := range scenario.Steps {
			sb.WriteString(fmt.Sprintf("%d. %s → %s", i+1, step.From.String(), step.To.String()))
			if step.Description != nil {
				sb.WriteString(fmt.Sprintf(" (%s)", *step.Description))
			}
			if len(step.Tags) > 0 {
				sb.WriteString(fmt.Sprintf(" [%s]", strings.Join(step.Tags, ", ")))
			}
			sb.WriteString("\n")
		}
		sb.WriteString("\n")
	}
}

func (e *Exporter) writePolicy(sb *strings.Builder, policy *language.Policy) {
	sb.WriteString(fmt.Sprintf("### %s\n\n", policy.ID))
	sb.WriteString(fmt.Sprintf("%s\n\n", policy.Description))

	if policy.Category != nil {
		sb.WriteString(fmt.Sprintf("**Category**: %s\n\n", *policy.Category))
	}
	if policy.Enforcement != nil {
		sb.WriteString(fmt.Sprintf("**Enforcement**: %s\n\n", *policy.Enforcement))
	}

	// Write metadata if present
	if len(policy.Metadata) > 0 {
		fmt.Fprintf(sb, "**Metadata**:\n")
		for _, meta := range policy.Metadata {
			val := ""
			if meta.Value != nil {
				val = *meta.Value
			}
			sb.WriteString(fmt.Sprintf("- **%s**: %s\n", meta.Key, val))
		}
		sb.WriteString("\n")
	}
}

func (e *Exporter) writeFlow(sb *strings.Builder, flow *language.Flow) {
	sb.WriteString(fmt.Sprintf("### %s: %s\n\n", flow.ID, flow.Title))

	if flow.Description != nil {
		sb.WriteString(fmt.Sprintf("%s\n\n", *flow.Description))
	}

	// Write flow steps (Flow is alias to Scenario - uses ScenarioStep with From/To)
	if len(flow.Steps) > 0 {
		sb.WriteString("#### Data Flows\n\n")
		for i, step := range flow.Steps {
			sb.WriteString(fmt.Sprintf("%d. %s -> %s", i+1, step.From.String(), step.To.String()))
			if step.Description != nil {
				sb.WriteString(fmt.Sprintf(" - %s", *step.Description))
			}
			sb.WriteString("\n")
		}
		sb.WriteString("\n")
	}
}

func (e *Exporter) writeContract(sb *strings.Builder, contract *language.Contract) {
	writeContractHeader(sb, contract)

	if contract.Body == nil {
		return
	}

	writeContractDetails(sb, contract.Body)
	writeSchema(sb, "Request Schema", contract.Body.Request)
	writeSchema(sb, "Response Schema", contract.Body.Response)
	writeContractErrors(sb, contract.Body.Errors)
	writeContractGuarantees(sb, contract.Body.Guarantees)

	// Write event-specific fields (if any)
	// Note: Event contracts may have emits_schema or writes_schema fields
}

func (e *Exporter) writeDataConsistency(sb *strings.Builder, arch *language.Architecture) {
	// Extract consistency information from containers, relations, and metadata
	consistencyNotes := extractConsistencyMetadata(arch)
	consistencyNotes = append(consistencyNotes, extractConsistencyFromRelations(arch)...)

	// Only render section if there's content
	if len(consistencyNotes) == 0 {
		return
	}

	sb.WriteString("## Data Consistency Models\n\n")
	sb.WriteString("### Consistency Guarantees\n\n")
	for _, note := range consistencyNotes {
		sb.WriteString(note + "\n")
	}
	sb.WriteString("\n")
}

// writeExecutiveSummary is implemented in executive_summary_helpers.go

// writeFailureModes is implemented in failure_modes_helpers.go

func (e *Exporter) writeRequirementsGrouped(sb *strings.Builder, reqs []*language.Requirement) {
	typeGroup := map[string][]*language.Requirement{}
	for _, r := range reqs {
		typeGroup[strVal(r.Type)] = append(typeGroup[strVal(r.Type)], r)
	}
	for _, t := range []string{"functional", "performance", "security", "constraint"} {
		if entries, ok := typeGroup[t]; ok {
			//nolint:staticcheck // Simple title casing is sufficient here, guarding against dependency addition
			sb.WriteString(fmt.Sprintf("### %s\n\n", strings.Title(t)))
			for _, r := range entries {
				e.writeRequirement(sb, r)
			}
		}
	}
}

func (e *Exporter) writeContracts(sb *strings.Builder, contracts []*language.Contract) {
	apis, events, data := groupContractsByKind(contracts)

	if len(apis) > 0 {
		sb.WriteString("### API Contracts\n\n")
		for _, c := range apis {
			e.writeContract(sb, c)
		}
	}
	if len(events) > 0 {
		sb.WriteString("### Event Contracts\n\n")
		for _, c := range events {
			e.writeContract(sb, c)
		}
	}
	if len(data) > 0 {
		sb.WriteString("### Data Contracts\n\n")
		for _, c := range data {
			e.writeContract(sb, c)
		}
	}
}

func (e *Exporter) writeRelations(sb *strings.Builder, arch *language.Architecture) {
	if len(arch.Relations) == 0 {
		return
	}
	sb.WriteString("## Relations\n\n")
	for _, rel := range arch.Relations {
		label := ""
		if rel.Verb != nil {
			label = *rel.Verb
		} else if rel.Label != nil {
			label = *rel.Label
		}
		if label != "" {
			sb.WriteString(fmt.Sprintf("- %s → %s: %s", rel.From.String(), rel.To.String(), label))
		} else {
			sb.WriteString(fmt.Sprintf("- %s → %s", rel.From.String(), rel.To.String()))
		}
		if len(rel.Tags) > 0 {
			sb.WriteString(fmt.Sprintf(" [%s]", strings.Join(rel.Tags, ", ")))
		}
		sb.WriteString("\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeQualityAttributes(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Quality Attributes\n\n")
	items := extractQualityRequirements(arch)
	items = append(items, extractQualityProperties(arch)...)

	if len(items) == 0 {
		sb.WriteString("Not specified\n")
	} else {
		for _, item := range items {
			sb.WriteString(item)
		}
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeSecurity(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Security\n\n")
	var wroteAny bool
	for _, r := range arch.Requirements {
		if strVal(r.Type) == "security" {
			sb.WriteString(fmt.Sprintf("- %s\n", strVal(r.Description)))
			wroteAny = true
		}
	}
	for _, p := range arch.Policies {
		if p.Category != nil && strings.Contains(strings.ToLower(*p.Category), "security") {
			sb.WriteString(fmt.Sprintf("- %s\n", p.Description))
			wroteAny = true
		}
	}
	if !wroteAny {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeMetadata(sb *strings.Builder, arch *language.Architecture) {
	if arch.Properties == nil && len(arch.Metadata) == 0 {
		return
	}
	sb.WriteString("## Document Metadata\n\n")
	if arch.Properties != nil {
		sb.WriteString("**Properties**:\n\n")
		for k, v := range arch.Properties {
			sb.WriteString(fmt.Sprintf("- **%s**: %s\n", k, v))
		}
		sb.WriteString("\n")
	}
	if len(arch.Metadata) > 0 {
		sb.WriteString("**Metadata**:\n\n")
		for _, m := range arch.Metadata {
			if m.Value != nil {
				sb.WriteString(fmt.Sprintf("- **%s**: %s\n", m.Key, *m.Value))
			}
		}
		sb.WriteString("\n")
	}
}

func (e *Exporter) writeGlossary(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Glossary\n\n")
	var wroteAny bool
	for _, sys := range arch.Systems {
		sb.WriteString(fmt.Sprintf("- **%s**: %s\n", sys.ID, sys.Label))
		wroteAny = true
	}
	for _, c := range arch.Containers {
		sb.WriteString(fmt.Sprintf("- **%s**: %s\n", c.ID, c.Label))
		wroteAny = true
	}
	for _, p := range arch.Persons {
		sb.WriteString(fmt.Sprintf("- **%s**: %s\n", p.ID, p.Label))
		wroteAny = true
	}
	if !wroteAny {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeTOC(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Table of Contents\n\n")
	sb.WriteString("- [Executive Summary](#executive-summary)\n")
	sb.WriteString("- [Architecture Overview (C4 L1)](#architecture-overview-c4-l1)\n")
	sb.WriteString("- [Systems (C4 L2/L3)](#systems)\n")

	writeTOCItem(sb, len(arch.DeploymentNodes) > 0, "Deployment Architecture", "deployment-architecture")
	writeTOCItem(sb, len(arch.Persons) > 0, "Persons", "persons")
	writeTOCItem(sb, len(arch.Requirements) > 0, "Requirements", "requirements")
	writeTOCItem(sb, len(arch.Policies) > 0, "Policies", "policies")
	writeTOCItem(sb, len(arch.Constraints) > 0, "Constraints", "constraints")
	writeTOCItem(sb, len(arch.Conventions) > 0, "Conventions", "conventions")
	writeTOCItem(sb, len(arch.Contracts) > 0, "Integration Contracts", "integration-contracts")
	writeTOCItem(sb, len(arch.Scenarios) > 0, "Scenarios", "scenarios")
	writeTOCItem(sb, len(arch.Flows) > 0, "Flows", "flows")

	writeTOCItems(sb)

	writeTOCItem(sb, len(arch.ADRs) > 0, "Architecture Decision Records (ADRs)", "architecture-decision-records-adrs")
	sb.WriteString("\n")
}

func strVal(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

// writeCapacityPlanning and other FAANG helpers are implemented in faang_helpers.go
// All FAANG section helpers are implemented in faang_helpers.go:
// - writeCapacityPlanning
// - writeMonitoringObservability
// - writeSecurityThreatModel
// - writeComplianceMatrix
// - writeDependencyRiskAssessment
// - writeCostAnalysis
// - writeAPIVersioning
// - writeMultiRegionConsiderations
// - writeDataLifecycleManagement

// pkg/export/markdown/template_data.go
// Package markdown provides data structures for template-based rendering.
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// DocumentData holds all pre-rendered sections for the main document template
type DocumentData struct {
	Header                    HeaderSection
	TOC                       string
	ExecutiveSummary          string
	Overview                  string
	Systems                   string
	Deployments               string
	Persons                   string
	Requirements              string
	ADRs                      string
	Scenarios                 string
	Policies                  string
	Constraints               string
	Conventions               string
	Flows                     string
	Contracts                 string
	DataConsistency           string
	FailureModes              string
	DomainModel               string
	Relations                 string
	QualityAttributes         string
	Security                  string
	CapacityPlanning          string
	MonitoringObservability   string
	SecurityThreatModel       string
	ComplianceMatrix          string
	DependencyRiskAssessment  string
	CostAnalysis              string
	APIVersioning             string
	MultiRegionConsiderations string
	DataLifecycleManagement   string
	Metadata                  string
	Glossary                  string
}

// HeaderSection holds header data
type HeaderSection struct {
	Name        string
	Description string
}

// buildDocumentData constructs DocumentData from Architecture by rendering each section
func (e *Exporter) buildDocumentData(arch *language.Architecture, config MermaidConfig) DocumentData {
	if arch == nil {
		return DocumentData{}
	}
	data := DocumentData{
		Header: HeaderSection{
			Name:        arch.Name,
			Description: e.getDescription(arch.Description),
		},
		TOC:              e.conditionalRender(e.Options.IncludeTOC, func() string { return e.renderTOC(arch) }),
		ExecutiveSummary: e.conditionalRender(true, func() string { return e.renderExecutiveSummary(arch) }),
		Overview:         e.conditionalRender(e.Options.IncludeOverview, func() string { return e.renderOverview(arch, config) }),
		Systems:          e.conditionalRender(e.Options.IncludeSystems, func() string { return e.renderSystems(arch, config) }),
		Deployments:      e.conditionalRender(e.Options.IncludeDeployments, func() string { return e.renderDeployments(arch, config) }),
		Persons:          e.conditionalRender(e.Options.IncludePersons, func() string { return e.renderPersons(arch) }),
		Requirements:     e.conditionalRender(e.Options.IncludeRequirements, func() string { return e.renderRequirements(arch) }),
		ADRs:             e.conditionalRender(e.Options.IncludeADRs, func() string { return e.renderADRs(arch) }),
		Scenarios:        e.conditionalRender(e.Options.IncludeScenarios, func() string { return e.renderScenarios(arch, config) }),
		Policies:         e.conditionalRender(e.Options.IncludePolicies, func() string { return e.renderPolicies(arch) }),
		Constraints:      e.conditionalRender(e.Options.IncludeConstraints, func() string { return e.renderConstraints(arch) }),
		Conventions:      e.conditionalRender(e.Options.IncludeConventions, func() string { return e.renderConventions(arch) }),
		Flows:            e.conditionalRender(e.Options.IncludeFlows, func() string { return e.renderFlows(arch) }),
		Contracts:        e.conditionalRender(e.Options.IncludeContracts, func() string { return e.renderContracts(arch) }),
		DataConsistency:  e.conditionalRender(e.Options.IncludeDataConsistency, func() string { return e.renderDataConsistency(arch) }),
		FailureModes:     e.conditionalRender(e.Options.IncludeFailureModes, func() string { return e.renderFailureModes(arch) }),

		Relations:                 e.conditionalRender(e.Options.IncludeRelations, func() string { return e.renderRelations(arch) }),
		QualityAttributes:         e.conditionalRender(e.Options.IncludeQualityAttributes, func() string { return e.renderQualityAttributes(arch) }),
		Security:                  e.conditionalRender(e.Options.IncludeSecurity, func() string { return e.renderSecurity(arch) }),
		CapacityPlanning:          e.conditionalRender(true, func() string { return e.renderCapacityPlanning(arch) }),
		MonitoringObservability:   e.conditionalRender(true, func() string { return e.renderMonitoringObservability(arch) }),
		SecurityThreatModel:       e.conditionalRender(true, func() string { return e.renderSecurityThreatModel(arch) }),
		ComplianceMatrix:          e.conditionalRender(true, func() string { return e.renderComplianceMatrix(arch) }),
		DependencyRiskAssessment:  e.conditionalRender(true, func() string { return e.renderDependencyRiskAssessment(arch) }),
		CostAnalysis:              e.conditionalRender(true, func() string { return e.renderCostAnalysis(arch) }),
		APIVersioning:             e.conditionalRender(true, func() string { return e.renderAPIVersioning(arch) }),
		MultiRegionConsiderations: e.conditionalRender(true, func() string { return e.renderMultiRegionConsiderations(arch) }),
		DataLifecycleManagement:   e.conditionalRender(true, func() string { return e.renderDataLifecycleManagement(arch) }),
		Metadata:                  e.conditionalRender(e.Options.IncludeMetadata, func() string { return e.renderMetadata(arch) }),
		Glossary:                  e.conditionalRender(e.Options.IncludeGlossary, func() string { return e.renderGlossary(arch) }),
	}
	return data
}

// conditionalRender renders a section only if enabled in options
func (e *Exporter) conditionalRender(include bool, renderFn func() string) string {
	if !include {
		return ""
	}
	return renderFn()
}

// getDescription safely gets description string
func (e *Exporter) getDescription(desc *string) string {
	if desc == nil {
		return ""
	}
	return *desc
}

// renderTOC renders table of contents
func (e *Exporter) renderTOC(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeTOC(&sb, arch)
	return sb.String()
}

// renderOverview renders architecture overview section
func (e *Exporter) renderOverview(arch *language.Architecture, config MermaidConfig) string {
	if len(arch.Systems) == 0 && len(arch.Persons) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Architecture Overview (C4 L1)\n\n")
	diagram := generateSystemDiagram(arch, config)
	sb.WriteString(fmt.Sprintf("```mermaid\n%s\n```\n\n", diagram))
	return sb.String()
}

// renderSystems renders systems section
func (e *Exporter) renderSystems(arch *language.Architecture, config MermaidConfig) string {
	if len(arch.Systems) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Systems\n\n")
	for _, sys := range arch.Systems {
		e.writeSystem(&sb, sys, arch, config)
	}
	return sb.String()
}

// renderDeployments renders deployments section
func (e *Exporter) renderDeployments(arch *language.Architecture, config MermaidConfig) string {
	if len(arch.DeploymentNodes) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Deployment Architecture\n\n")
	for _, deployment := range arch.DeploymentNodes {
		e.writeDeployment(&sb, deployment, config)
	}
	return sb.String()
}

// renderPersons renders persons section
func (e *Exporter) renderPersons(arch *language.Architecture) string {
	if len(arch.Persons) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Persons\n\n")
	for _, person := range arch.Persons {
		e.writePerson(&sb, person)
	}
	return sb.String()
}

// renderRequirements renders requirements section
func (e *Exporter) renderRequirements(arch *language.Architecture) string {
	if len(arch.Requirements) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Requirements\n\n")
	e.writeRequirementsGrouped(&sb, arch.Requirements)
	return sb.String()
}

// renderADRs renders ADRs section
func (e *Exporter) renderADRs(arch *language.Architecture) string {
	if len(arch.ADRs) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Architecture Decision Records (ADRs)\n\n")
	for _, adr := range arch.ADRs {
		e.writeADR(&sb, adr)
	}
	return sb.String()
}

// renderScenarios renders scenarios section
func (e *Exporter) renderScenarios(arch *language.Architecture, config MermaidConfig) string {
	if len(arch.Scenarios) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Scenarios\n\n")
	for _, scenario := range arch.Scenarios {
		e.writeScenario(&sb, scenario, config)
	}
	return sb.String()
}

// renderPolicies renders policies section
func (e *Exporter) renderPolicies(arch *language.Architecture) string {
	if len(arch.Policies) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Policies\n\n")
	for _, policy := range arch.Policies {
		e.writePolicy(&sb, policy)
	}
	return sb.String()
}

// renderConstraints renders constraints section
func (e *Exporter) renderConstraints(arch *language.Architecture) string {
	if len(arch.Constraints) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Constraints\n\n")
	for _, c := range arch.Constraints {
		sb.WriteString(fmt.Sprintf("- **%s**: %s\n", c.Key, c.Value))
	}
	sb.WriteString("\n")
	return sb.String()
}

// renderConventions renders conventions section
func (e *Exporter) renderConventions(arch *language.Architecture) string {
	if len(arch.Conventions) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Conventions\n\n")
	for _, c := range arch.Conventions {
		sb.WriteString(fmt.Sprintf("- **%s**: %s\n", c.Key, c.Value))
	}
	sb.WriteString("\n")
	return sb.String()
}

// renderFlows renders flows section
func (e *Exporter) renderFlows(arch *language.Architecture) string {
	if len(arch.Flows) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Flows\n\n")
	for _, flow := range arch.Flows {
		e.writeFlow(&sb, flow)
	}
	return sb.String()
}

// renderContracts renders contracts section
func (e *Exporter) renderContracts(arch *language.Architecture) string {
	if len(arch.Contracts) == 0 {
		return ""
	}
	var sb strings.Builder
	sb.WriteString("## Integration Contracts\n\n")
	e.writeContracts(&sb, arch.Contracts)
	return sb.String()
}

// renderDataConsistency renders data consistency section
func (e *Exporter) renderDataConsistency(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeDataConsistency(&sb, arch)
	return sb.String()
}

// renderFailureModes renders failure modes section
func (e *Exporter) renderFailureModes(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeFailureModes(&sb, arch)
	return sb.String()
}

// renderRelations renders relations section
func (e *Exporter) renderRelations(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeRelations(&sb, arch)
	return sb.String()
}

// renderQualityAttributes renders quality attributes section
func (e *Exporter) renderQualityAttributes(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeQualityAttributes(&sb, arch)
	return sb.String()
}

// renderSecurity renders security section
func (e *Exporter) renderSecurity(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeSecurity(&sb, arch)
	return sb.String()
}

// renderMetadata renders metadata section
func (e *Exporter) renderMetadata(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeMetadata(&sb, arch)
	return sb.String()
}

// renderGlossary renders glossary section
func (e *Exporter) renderGlossary(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeGlossary(&sb, arch)
	return sb.String()
}

// renderExecutiveSummary renders executive summary section
func (e *Exporter) renderExecutiveSummary(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeExecutiveSummary(&sb, arch)
	return sb.String()
}

// renderCapacityPlanning renders capacity planning section
func (e *Exporter) renderCapacityPlanning(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeCapacityPlanning(&sb, arch)
	return sb.String()
}

// renderMonitoringObservability renders monitoring & observability section
func (e *Exporter) renderMonitoringObservability(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeMonitoringObservability(&sb, arch)
	return sb.String()
}

// renderSecurityThreatModel renders security threat model section
func (e *Exporter) renderSecurityThreatModel(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeSecurityThreatModel(&sb, arch)
	return sb.String()
}

// renderComplianceMatrix renders compliance matrix section
func (e *Exporter) renderComplianceMatrix(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeComplianceMatrix(&sb, arch)
	return sb.String()
}

// renderDependencyRiskAssessment renders dependency risk assessment section
func (e *Exporter) renderDependencyRiskAssessment(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeDependencyRiskAssessment(&sb, arch)
	return sb.String()
}

// renderCostAnalysis renders cost analysis section
func (e *Exporter) renderCostAnalysis(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeCostAnalysis(&sb, arch)
	return sb.String()
}

// renderAPIVersioning renders API versioning section
func (e *Exporter) renderAPIVersioning(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeAPIVersioning(&sb, arch)
	return sb.String()
}

// renderMultiRegionConsiderations renders multi-region considerations section
func (e *Exporter) renderMultiRegionConsiderations(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeMultiRegionConsiderations(&sb, arch)
	return sb.String()
}

// renderDataLifecycleManagement renders data lifecycle management section
func (e *Exporter) renderDataLifecycleManagement(arch *language.Architecture) string {
	var sb strings.Builder
	e.writeDataLifecycleManagement(&sb, arch)
	return sb.String()
}

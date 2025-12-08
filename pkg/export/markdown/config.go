// pkg/export/markdown/config.go
// Package markdown provides configuration options for markdown export.
package markdown

// ExportOptions holds configuration options for markdown export
type ExportOptions struct {
	// Section control - which sections to include
	IncludeTOC               bool
	IncludeOverview          bool
	IncludeSystems           bool
	IncludeDeployments       bool
	IncludePersons           bool
	IncludeRequirements      bool
	IncludeADRs              bool
	IncludeScenarios         bool
	IncludePolicies          bool
	IncludeConstraints       bool
	IncludeConventions       bool
	IncludeFlows             bool
	IncludeContracts         bool
	IncludeDataConsistency   bool
	IncludeFailureModes      bool
	IncludeDomainModel       bool
	IncludeRelations         bool
	IncludeQualityAttributes bool
	IncludeSecurity          bool
	IncludeMetadata          bool
	IncludeGlossary          bool

	// Mermaid configuration
	MermaidConfig MermaidConfig

	// Output formatting
	HeadingLevel int // Starting heading level (1 = #, 2 = ##, etc.)
}

// DefaultExportOptions returns default export options with all sections enabled
func DefaultExportOptions() ExportOptions {
	return ExportOptions{
		IncludeTOC:               true,
		IncludeOverview:          true,
		IncludeSystems:           true,
		IncludeDeployments:       true,
		IncludePersons:           true,
		IncludeRequirements:      true,
		IncludeADRs:              true,
		IncludeScenarios:         true,
		IncludePolicies:          true,
		IncludeConstraints:       true,
		IncludeConventions:       true,
		IncludeFlows:             true,
		IncludeContracts:         true,
		IncludeDataConsistency:   true,
		IncludeFailureModes:      true,
		IncludeDomainModel:       true,
		IncludeRelations:         true,
		IncludeQualityAttributes: true,
		IncludeSecurity:          true,
		IncludeMetadata:          true,
		IncludeGlossary:          true,
		HeadingLevel:             1,
	}
}

// MinimalExportOptions returns minimal export options (only essential sections)
func MinimalExportOptions() ExportOptions {
	return ExportOptions{
		IncludeTOC:               true,
		IncludeOverview:          true,
		IncludeSystems:           true,
		IncludeDeployments:       true,
		IncludePersons:           true,
		IncludeRequirements:      false,
		IncludeADRs:              false,
		IncludeScenarios:         false,
		IncludePolicies:          false,
		IncludeConstraints:       false,
		IncludeConventions:       false,
		IncludeFlows:             false,
		IncludeContracts:         false,
		IncludeDataConsistency:   false,
		IncludeFailureModes:      false,
		IncludeDomainModel:       false,
		IncludeRelations:         true,
		IncludeQualityAttributes: false,
		IncludeSecurity:          false,
		IncludeMetadata:          false,
		IncludeGlossary:          false,
		HeadingLevel:             1,
	}
}




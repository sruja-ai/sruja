package markdown

import (
	"testing"
)

func TestMinimalExportOptions(t *testing.T) {
	opts := MinimalExportOptions()

	// Essential sections that should be enabled
	if !opts.IncludeTOC {
		t.Error("MinimalExportOptions should include TOC")
	}
	if !opts.IncludeOverview {
		t.Error("MinimalExportOptions should include Overview")
	}
	if !opts.IncludeSystems {
		t.Error("MinimalExportOptions should include Systems")
	}
	if !opts.IncludeDeployments {
		t.Error("MinimalExportOptions should include Deployments")
	}
	if !opts.IncludePersons {
		t.Error("MinimalExportOptions should include Persons")
	}
	if !opts.IncludeRelations {
		t.Error("MinimalExportOptions should include Relations")
	}

	// Optional sections that should be disabled
	if opts.IncludeRequirements {
		t.Error("MinimalExportOptions should not include Requirements")
	}
	if opts.IncludeADRs {
		t.Error("MinimalExportOptions should not include ADRs")
	}
	if opts.IncludeScenarios {
		t.Error("MinimalExportOptions should not include Scenarios")
	}
	if opts.IncludePolicies {
		t.Error("MinimalExportOptions should not include Policies")
	}
	if opts.IncludeConstraints {
		t.Error("MinimalExportOptions should not include Constraints")
	}
	if opts.IncludeConventions {
		t.Error("MinimalExportOptions should not include Conventions")
	}
	if opts.IncludeFlows {
		t.Error("MinimalExportOptions should not include Flows")
	}
	if opts.IncludeContracts {
		t.Error("MinimalExportOptions should not include Contracts")
	}
	if opts.IncludeDataConsistency {
		t.Error("MinimalExportOptions should not include DataConsistency")
	}
	if opts.IncludeFailureModes {
		t.Error("MinimalExportOptions should not include FailureModes")
	}
	if opts.IncludeDomainModel {
		t.Error("MinimalExportOptions should not include DomainModel")
	}
	if opts.IncludeQualityAttributes {
		t.Error("MinimalExportOptions should not include QualityAttributes")
	}
	if opts.IncludeSecurity {
		t.Error("MinimalExportOptions should not include Security")
	}
	if opts.IncludeMetadata {
		t.Error("MinimalExportOptions should not include Metadata")
	}
	if opts.IncludeGlossary {
		t.Error("MinimalExportOptions should not include Glossary")
	}

	// Check heading level default
	if opts.HeadingLevel != 1 {
		t.Errorf("Expected HeadingLevel=1, got %d", opts.HeadingLevel)
	}
}

func TestDefaultExportOptions(t *testing.T) {
	opts := DefaultExportOptions()

	// All sections should be enabled in default
	if !opts.IncludeTOC {
		t.Error("DefaultExportOptions should include TOC")
	}
	if !opts.IncludeOverview {
		t.Error("DefaultExportOptions should include Overview")
	}
	if !opts.IncludeSystems {
		t.Error("DefaultExportOptions should include Systems")
	}
	if !opts.IncludeRequirements {
		t.Error("DefaultExportOptions should include Requirements")
	}
	if !opts.IncludeADRs {
		t.Error("DefaultExportOptions should include ADRs")
	}
	if !opts.IncludeScenarios {
		t.Error("DefaultExportOptions should include Scenarios")
	}
	if !opts.IncludeGlossary {
		t.Error("DefaultExportOptions should include Glossary")
	}

	// Check heading level default
	if opts.HeadingLevel != 1 {
		t.Errorf("Expected HeadingLevel=1, got %d", opts.HeadingLevel)
	}
}

func TestNewExporterWithOptions(t *testing.T) {
	customOpts := ExportOptions{
		IncludeTOC:      false,
		IncludeOverview: true,
		IncludeSystems:  true,
		HeadingLevel:    2,
		MermaidConfig: MermaidConfig{
			Direction: "LR",
			Theme:     "dark",
		},
	}

	exporter := NewExporterWithOptions(customOpts)

	if exporter == nil {
		t.Fatal("NewExporterWithOptions returned nil")
	}

	if exporter.Options.IncludeTOC {
		t.Error("Expected IncludeTOC to be false")
	}
	if !exporter.Options.IncludeOverview {
		t.Error("Expected IncludeOverview to be true")
	}
	if exporter.Options.HeadingLevel != 2 {
		t.Errorf("Expected HeadingLevel=2, got %d", exporter.Options.HeadingLevel)
	}
	if exporter.Options.MermaidConfig.Direction != "LR" {
		t.Errorf("Expected Direction=LR, got %s", exporter.Options.MermaidConfig.Direction)
	}
	if exporter.Options.MermaidConfig.Theme != "dark" {
		t.Errorf("Expected Theme=dark, got %s", exporter.Options.MermaidConfig.Theme)
	}
}

func TestNewExporter_DefaultOptions(t *testing.T) {
	exporter := NewExporter()

	if exporter == nil {
		t.Fatal("NewExporter returned nil")
	}

	// Should use default options
	if !exporter.Options.IncludeTOC {
		t.Error("NewExporter should use DefaultExportOptions which includes TOC")
	}
	if !exporter.Options.IncludeOverview {
		t.Error("NewExporter should use DefaultExportOptions which includes Overview")
	}
	if exporter.Options.HeadingLevel != 1 {
		t.Errorf("Expected HeadingLevel=1 from defaults, got %d", exporter.Options.HeadingLevel)
	}
}

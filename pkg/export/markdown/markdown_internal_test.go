package markdown

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestRenderConstraints(t *testing.T) {
	arch := &language.Architecture{
		Constraints: []*language.ConstraintEntry{
			{Key: "C1", Value: "V1"},
		},
	}
	e := NewExporter()
	output := e.renderConstraints(arch)
	if !strings.Contains(output, "## Constraints") {
		t.Error("Expected Constraints header")
	}
	if !strings.Contains(output, "**C1**: V1") {
		t.Error("Expected constraint content")
	}
}

func TestRenderConventions(t *testing.T) {
	arch := &language.Architecture{
		Conventions: []*language.ConventionEntry{
			{Key: "C1", Value: "V1"},
		},
	}
	e := NewExporter()
	output := e.renderConventions(arch)
	if !strings.Contains(output, "## Conventions") {
		t.Error("Expected Conventions header")
	}
	if !strings.Contains(output, "**C1**: V1") {
		t.Error("Expected convention content")
	}
}

func TestRenderPolicies(t *testing.T) {
	arch := &language.Architecture{
		Policies: []*language.Policy{
			{ID: "P1", Description: "Desc"},
		},
	}
	e := NewExporter()
	output := e.renderPolicies(arch)
	if !strings.Contains(output, "## Policies") {
		t.Error("Expected Policies header")
	}
	if !strings.Contains(output, "P1") {
		t.Error("Expected policy ID")
	}
}

func TestRenderScenarios(t *testing.T) {
	arch := &language.Architecture{
		Scenarios: []*language.Scenario{
			{Title: "S1"},
		},
	}
	e := NewExporter()
	output := e.renderScenarios(arch, MermaidConfig{})
	if !strings.Contains(output, "## Scenarios") {
		t.Error("Expected Scenarios header")
	}
	if !strings.Contains(output, "S1") {
		t.Error("Expected scenario title")
	}
}
func TestRenderADRs(t *testing.T) {
	title := "Use Go"
	status := "Accepted"
	arch := &language.Architecture{
		ADRs: []*language.ADR{
			{ID: "ADR1", Title: &title, Body: &language.ADRBody{Status: &status}},
		},
	}
	e := NewExporter()
	output := e.renderADRs(arch)
	if !strings.Contains(output, "## Architecture Decision Records (ADRs)") {
		t.Error("Expected ADRs header")
	}
	// ID is not rendered if Title is present
	if !strings.Contains(output, "Use Go") {
		t.Error("Expected ADR title")
	}
}

func TestRenderFlows(t *testing.T) {
	desc := "Flow description"
	arch := &language.Architecture{
		Flows: []*language.Flow{
			{ID: "F1", Description: &desc},
		},
	}
	e := NewExporter()
	output := e.renderFlows(arch)
	if !strings.Contains(output, "## Flows") {
		t.Error("Expected Flows header")
	}
	if !strings.Contains(output, "F1") {
		t.Error("Expected flow ID")
	}
}

func TestRenderContracts(t *testing.T) {
	arch := &language.Architecture{
		Contracts: []*language.Contract{
			{ID: "C1", Kind: "api"},
		},
	}
	e := NewExporter()
	output := e.renderContracts(arch)
	if !strings.Contains(output, "## Integration Contracts") {
		t.Errorf("Expected Contracts header, got: %q", output)
	}
	if !strings.Contains(output, "C1") {
		t.Error("Expected contract ID")
	}
}

func TestSystemRepID(t *testing.T) {
	id := "Sys1"
	expected := "Sys1_ctx"
	if got := systemRepID(id); got != expected {
		t.Errorf("systemRepID(%s) = %s, want %s", id, got, expected)
	}
}

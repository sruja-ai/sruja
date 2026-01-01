package dsl

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/json"
)

func TestPrint_Links(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"Sys1": {
				ID:    "Sys1",
				Kind:  "system",
				Title: "System 1",
				Links: []json.LinkDump{
					{URL: "https://example.com/docs", Title: "Docs"},
					{URL: "https://example.com/repo"},
				},
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, `link "https://example.com/docs" "Docs"`) {
		t.Error("Expected titled link")
	}
	if !strings.Contains(result, `link "https://example.com/repo"`) {
		t.Error("Expected untitled link")
	}
}

func TestPrint_ConstraintsAndConventions(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{},
		Sruja: &json.SrujaExtensions{
			Constraints: []json.ConstraintDump{
				{ID: "CONS1", Type: "technical", Description: "Must run on Linux"},
			},
			Conventions: []json.ConventionDump{
				{ID: "CONV1", Description: "Use snake_case"},
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, `constraint CONS1 technical "Must run on Linux"`) {
		t.Error("Expected constraint")
	}
	if !strings.Contains(result, `convention CONV1 "Use snake_case"`) {
		t.Error("Expected convention")
	}
}

func TestPrint_RelationBackArrow(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"A": {ID: "A"},
			"B": {ID: "B"},
		},
		Relations: []json.RelationDump{
			{
				Source: json.FqnRefDump{Model: "A"},
				Target: json.FqnRefDump{Model: "B"},
				Kind:   "back",
			},
		},
	}

	result := Print(model)

	if !strings.Contains(result, "A <- B") {
		t.Error("Expected back arrow relation")
	}
}

func TestPrint_RelationInvalid(t *testing.T) {
	model := &json.SrujaModelDump{
		Elements: map[string]json.ElementDump{
			"A": {ID: "A"},
		},
		Relations: []json.RelationDump{
			{Source: json.FqnRefDump{Model: "A"}, Target: json.FqnRefDump{Model: ""}}, // Missing target
			{Source: json.FqnRefDump{Model: ""}, Target: json.FqnRefDump{Model: "A"}}, // Missing source
		},
	}

	result := Print(model)
	// Should not print invalid relations
	if strings.Contains(result, "->") || strings.Contains(result, "<-") {
		t.Error("Should not print relation with missing source/target")
	}
}

func TestPrint_ExtensionsEmpty(t *testing.T) {
	model := &json.SrujaModelDump{
		Sruja: &json.SrujaExtensions{}, // Not nil, but empty
	}
	result := Print(model)
	// Should be empty string (or close to it)
	if strings.TrimSpace(result) != "" {
		t.Error("Expected empty output for empty extensions")
	}
}

func TestPrintFlat_NilModel(t *testing.T) {
	p := NewModelPrinter()
	res := p.PrintFlat(nil)
	if res != "" {
		t.Error("Expected empty string for nil model")
	}
}

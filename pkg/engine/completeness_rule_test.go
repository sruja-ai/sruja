package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestCompletenessRule_MissingDescriptions(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container API "Order API" {
			technology "Go"
            component Comp "Order Component" {
				technology "Go"
			}
        }
    }
}
`
	// Note: Description is missing for all elements above
	parser, _ := language.NewParser()
	program, _, _ := parser.Parse("test.sruja", dsl)

	rule := &engine.CompletenessRule{}
	diags := rule.Validate(program)

	expectedMsgs := []string{
		"System 'Sys' has no description. Adding one helps context.",
		"Container 'API' has no description.",
		"Component 'Comp' has no description.",
	}

	if len(diags) != len(expectedMsgs) {
		t.Errorf("Expected %d suggestions, got %d", len(expectedMsgs), len(diags))
		for _, d := range diags {
			t.Logf("Got: %s", d.Message)
		}
	}
}

func TestCompletenessRule_EmptySystem(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" "Description"
}
`
	parser, _ := language.NewParser()
	program, _, _ := parser.Parse("test.sruja", dsl)

	rule := &engine.CompletenessRule{}
	diags := rule.Validate(program)

	found := false
	for _, d := range diags {
		if d.Code == "SUGGESTION_EMPTY_SYSTEM" {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected SUGGESTION_EMPTY_SYSTEM for empty system")
	}
}

func TestCompletenessRule_MissingTechnology(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" "Description" {
        container API "Order API" "Description" {
            component Comp "Order Component" "Description"
        }
    }
}
`
	parser, _ := language.NewParser()
	program, _, _ := parser.Parse("test.sruja", dsl)

	rule := &engine.CompletenessRule{}
	diags := rule.Validate(program)

	found := false
	for _, d := range diags {
		if d.Code == "SUGGESTION_MISSING_TECH" {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected SUGGESTION_MISSING_TECH for container without technology")
	}
}

func TestCompletenessRule_EmptyContainer(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" "Description" {
        container API "Order API" "Description" {
			technology "Go"
		}
    }
}
`
	parser, _ := language.NewParser()
	program, _, _ := parser.Parse("test.sruja", dsl)

	rule := &engine.CompletenessRule{}
	diags := rule.Validate(program)

	found := false
	for _, d := range diags {
		if d.Code == "SUGGESTION_EMPTY_CONTAINER" {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected SUGGESTION_EMPTY_CONTAINER for container without components")
	}
}

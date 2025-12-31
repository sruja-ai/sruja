// pkg/dx/explainer_format_test.go
package dx

import (
	"strings"
	"testing"
)

func TestFormat(t *testing.T) {
	explanation := &ElementExplanation{
		ID:          "API",
		Description: "Test description",
		Metadata: map[string]string{
			"owner": "team",
		},
		Relations: RelationsInfo{
			Incoming: []*RelationInfo{
				{From: "User", To: "API", Label: "Uses"},
			},
			Outgoing: []*RelationInfo{
				{From: "API", To: "DB", Label: "Queries"},
			},
		},
		Dependencies: []string{"DB"},
		ADRs: []string{
			"ADR001",
		},
		Scenarios: []*ScenarioInfo{
			{ID: "scenario1", Label: "Test Scenario"},
		},
	}

	formatted := explanation.Format()
	if !strings.Contains(formatted, "Test description") {
		t.Error("Format should contain description")
	}
	if !strings.Contains(formatted, "Metadata") {
		t.Error("Format should contain metadata section")
	}
	if !strings.Contains(formatted, "Relations") {
		t.Error("Format should contain relations section")
	}
	if !strings.Contains(formatted, "Dependencies") {
		t.Error("Format should contain dependencies section")
	}
	if !strings.Contains(formatted, "Related ADRs") {
		t.Error("Format should contain ADRs section")
	}
	if !strings.Contains(formatted, "Related Scenarios") {
		t.Error("Format should contain scenarios section")
	}
}

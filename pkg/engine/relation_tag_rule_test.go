// pkg/engine/relation_tag_rule_test.go
package engine

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestRelationTagRule_Name(t *testing.T) {
	rule := &RelationTagRule{}
	if rule.Name() != "Relation Tags" {
		t.Errorf("Expected name 'Relation Tags', got '%s'", rule.Name())
	}
}

func TestRelationTagRule_Validate(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	tests := []struct {
		name            string
		input           string
		wantWarning     bool
		warningContains string
	}{
		{
			name:        "valid relation tags",
			input:       `model { API = system "API" DB = system "DB" API -> DB [reads, writes] }`,
			wantWarning: false,
		},
		{
			name:            "invalid relation tag",
			input:           `model { API = system "API" DB = system "DB" API -> DB [invalidtag] }`,
			wantWarning:     true,
			warningContains: "Invalid relation tag",
		},
		{
			name:            "multiple invalid tags",
			input:           `model { API = system "API" DB = system "DB" API -> DB [bad1, bad2] }`,
			wantWarning:     true,
			warningContains: "Invalid relation tag",
		},
		{
			name:        "all allowed tags",
			input:       `model { API = system "API" DB = system "DB" API -> DB [reads, writes, sends, uses, calls, processes, queries, updates, deletes, creates, triggers, notifies] }`,
			wantWarning: false,
		},
		{
			name:        "case insensitive tags",
			input:       `model { API = system "API" DB = system "DB" API -> DB [READS, Writes, SeNdS] }`,
			wantWarning: false,
		},
		{
			name:            "invalid tag in system relation",
			input:           `model { API = system "API" { API -> API [badtag] } }`,
			wantWarning:     true,
			warningContains: "Invalid relation tag",
		},
		{
			name:            "invalid tag in container relation",
			input:           `model { API = system "API" { Web = container "Web" { Web -> Web [badtag] } } }`,
			wantWarning:     true,
			warningContains: "Invalid relation tag",
		},
		{
			name:            "invalid tag in component relation",
			input:           `model { API = system "API" { Web = container "Web" { Auth = component "Auth" { Auth -> Auth [badtag] } } } }`,
			wantWarning:     true,
			warningContains: "Invalid relation tag",
		},
		{
			name:        "no tags - valid",
			input:       `model { API = system "API" DB = system "DB" API -> DB }`,
			wantWarning: false,
		},
		{
			name:        "nil program",
			input:       "",
			wantWarning: false,
		},
	}

	rule := &RelationTagRule{}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var program *language.Program
			if tt.input != "" {
				var parseErr error
				program, _, parseErr = parser.Parse("test.sruja", tt.input)
				if parseErr != nil {
					t.Fatalf("Parse error: %v", parseErr)
				}
			}

			diags := rule.Validate(program)
			hasWarning := false
			warningMessages := []string{}

			for _, diag := range diags {
				if diag.Severity == diagnostics.SeverityWarning {
					hasWarning = true
					warningMessages = append(warningMessages, diag.Message)
				}
			}

			if tt.wantWarning && !hasWarning {
				t.Errorf("Expected validation warning but got none. Diagnostics: %v", diags)
			}
			if !tt.wantWarning && hasWarning {
				t.Errorf("Unexpected validation warning: %v", diags)
			}
			if tt.warningContains != "" && hasWarning {
				found := false
				for _, msg := range warningMessages {
					if strings.Contains(strings.ToLower(msg), strings.ToLower(tt.warningContains)) {
						found = true
						break
					}
				}
				if !found {
					t.Errorf("Expected warning message to contain '%s', but got: %v", tt.warningContains, warningMessages)
				}
			}
		})
	}
}

func TestRelationTagRule_IntegrationWithValidator(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	input := `model {
		API = system "API"
		DB = system "DB"
		API -> DB [invalidtag]
	}`

	program, _, err := parser.Parse("test.sruja", input)
	if err != nil {
		t.Fatalf("Parse error: %v", err)
	}

	validator := NewValidator()
	validator.RegisterRule(&RelationTagRule{})
	diags := validator.Validate(program)

	hasWarning := false
	for _, diag := range diags {
		if diag.Severity == diagnostics.SeverityWarning && strings.Contains(diag.Message, "Invalid relation tag") {
			hasWarning = true
			break
		}
	}

	if !hasWarning {
		t.Errorf("Expected validation warning for invalid tag, but got none. Diagnostics: %v", diags)
	}
}

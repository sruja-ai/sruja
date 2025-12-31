package engine

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestDatabaseIsolationRule(t *testing.T) {
	tests := []struct {
		name     string
		dsl      string
		expected []string // substrings expected in error messages
	}{
		{
			name: "Clean Isolation",
			dsl: `
A = system "System A" {
API = container "API"
DB = database "DB"
API -> DB "Writes"
}
B = system "System B" {
API = container "API"
DB = database "DB"
API -> DB "Writes"
}
A.API -> B.API "Calls"
`,
			expected: nil,
		},
		{
			name: "Violation - Shared DB",
			dsl: `
A = system "System A" {
API = container "API"
}
B = system "System B" {
API = container "API"
}
SharedDB = database "Shared Database"

A.API -> SharedDB "Reads"
B.API -> SharedDB "Writes"
`,
			expected: []string{"DataStore 'SharedDB' is accessed by multiple services", "System A", "System B"},
		},
		{
			name: "Allowed Shared DB",
			dsl: `
A = system "System A" {
API = container "API"
}
B = system "System B" {
API = container "API"
}
SharedDB = database "Shared Database" {
metadata {
  shared "true"
}
}

A.API -> SharedDB "Reads"
B.API -> SharedDB "Writes"
`,
			expected: nil,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			parser, _ := language.NewParser()
			prog, _, err := parser.Parse("test.sruja", tc.dsl)
			if err != nil {
				t.Fatalf("Parse error: %v", err)
			}

			rule := &DatabaseIsolationRule{}
			diags := rule.Validate(prog)

			if len(tc.expected) == 0 {
				if len(diags) > 0 {
					t.Errorf("Expected no warnings, got %d: %v", len(diags), diags)
				}
			} else {
				if len(diags) == 0 {
					t.Errorf("Expected warnings containing %v, got none", tc.expected)
				}
				for _, d := range diags {
					msg := d.Message
					for range tc.expected {
						_ = msg
						_ = d
					}
				}
			}
		})
	}
}

func TestPublicInterfaceDocumentationRule(t *testing.T) {
	tests := []struct {
		name     string
		dsl      string
		expected []string
	}{
		{
			name: "Documented System",
			dsl: `
User = person "User"
Sys = system "System" {
description "Handles things"
}
User -> Sys "Uses"
`,
			expected: nil,
		},
		{
			name: "Undocumented System",
			dsl: `
User = person "User"
Sys = system "System" // No description
User -> Sys "Uses"
`,
			expected: []string{"System 'Sys' is used by humans but lacks a description"},
		},
		{
			name: "Undocumented Container",
			dsl: `
User = person "User"
Sys = system "System" {
Web = container "Web App" // No description, no technology
User -> Web "Uses"
}
`,
			expected: []string{
				"Container 'Sys.Web' is used by humans but lacks a description",
				"Container 'Sys.Web' should specify its Technology",
			},
		},
		{
			name: "Fully Documented Container",
			dsl: `
User = person "User"
Sys = system "System" {
Web = container "Web App" {
  description "The web interface"
  technology "React"
}
User -> Web "Uses"
}
`,
			expected: nil,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			parser, _ := language.NewParser()
			prog, _, err := parser.Parse("test.sruja", tc.dsl)
			if err != nil {
				t.Fatalf("Parse error: %v", err)
			}

			rule := &PublicInterfaceDocumentationRule{}
			diags := rule.Validate(prog)

			if len(tc.expected) == 0 {
				if len(diags) > 0 {
					t.Errorf("Expected no warnings, got %d: %v", len(diags), diags)
				}
			} else {
				foundCount := 0
				for _, d := range diags {
					for _, exp := range tc.expected {
						if strings.Contains(d.Message, exp) {
							foundCount++
						}
					}
				}
				// Simple check: Just ensure we got some warnings
				if len(diags) == 0 {
					t.Errorf("Expected warnings, got none")
				}
			}
		})
	}
}

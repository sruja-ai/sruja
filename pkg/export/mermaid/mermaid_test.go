package mermaid

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExporter_Export(t *testing.T) {
	// Setup a sample program
	title := "My System"
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{
					ElementDef: &language.LikeC4ElementDef{
						Definition: &language.LikeC4Definition{
							Kind:  "person",
							Name:  mkStr("User"),
							Title: mkStr("End User"),
						},
					},
				},
				{
					ElementDef: &language.LikeC4ElementDef{
						Definition: &language.LikeC4Definition{
							Kind:  "system",
							Name:  mkStr("SystemA"),
							Title: &title,
							Body: &language.LikeC4ElementDefBody{
								Items: []*language.LikeC4BodyItem{
									{
										Element: &language.LikeC4ElementDef{
											Definition: &language.LikeC4Definition{
												Kind:  "container",
												Name:  mkStr("WebApp"),
												Title: mkStr("Web Application"),
												Body: &language.LikeC4ElementDefBody{
													Items: []*language.LikeC4BodyItem{
														{
															Element: &language.LikeC4ElementDef{
																Definition: &language.LikeC4Definition{
																	Kind:  "component",
																	Name:  mkStr("UI"),
																	Title: mkStr("User Interface"),
																},
															},
														},
													},
												},
											},
										},
									},
									{
										Element: &language.LikeC4ElementDef{
											Definition: &language.LikeC4Definition{
												Kind:  "database",
												Name:  mkStr("DB"),
												Title: mkStr("Main DB"),
											},
										},
									},
									{
										Element: &language.LikeC4ElementDef{
											Definition: &language.LikeC4Definition{
												Kind:  "queue",
												Name:  mkStr("Queue1"),
												Title: mkStr("Message Queue"),
											},
										},
									},
									{
										Element: &language.LikeC4ElementDef{
											Definition: &language.LikeC4Definition{
												Kind:  "external",
												Name:  mkStr("ExternalSystem"),
												Title: mkStr("External System"),
											},
										},
									},
								},
							},
						},
					},
				},
				{
					ElementDef: &language.LikeC4ElementDef{
						Definition: &language.LikeC4Definition{
							Kind:  "container",
							Name:  mkStr("StandaloneCont"),
							Title: mkStr("Standalone Container"),
						},
					},
				},
				{
					ElementDef: &language.LikeC4ElementDef{
						Definition: &language.LikeC4Definition{
							Kind:  "system",
							Name:  mkStr("EmptySys"),
							Title: mkStr("Empty System"),
						},
					},
				},
				{
					Relation: &language.Relation{
						From: language.QualifiedIdent{Parts: []string{"User"}},
						To:   language.QualifiedIdent{Parts: []string{"SystemA"}},
						Verb: mkStr("navigates"),
					},
				},
				{
					Relation: &language.Relation{
						From: language.QualifiedIdent{Parts: []string{"User"}},
						To:   language.QualifiedIdent{Parts: []string{"StandaloneCont"}},
					},
				},
			},
		},
	}

	exporter := NewExporter(DefaultConfig())
	result := exporter.Export(prog)

	// Basic assertions
	if !strings.Contains(result, "graph LR") {
		t.Errorf("expected graph definition, got %s", result)
	}
	if !strings.Contains(result, "EmptySys") {
		t.Errorf("expected EmptySys to be present")
	}
	if !strings.Contains(result, "navigates") {
		t.Errorf("expected navigates verb to be present, but got:\n%s", result)
	}
}

func TestExporter_Export_Empty(t *testing.T) {
	exporter := NewExporter(DefaultConfig())
	result := exporter.Export(nil)
	if result != "" {
		t.Errorf("expected empty string for nil program, got %s", result)
	}

	result = exporter.Export(&language.Program{})
	if result != "" {
		t.Errorf("expected empty string for empty program, got %s", result)
	}
}

func TestExporter_Config(t *testing.T) {
	config := DefaultConfig()
	config.Direction = "TD"
	config.Theme = "dark"
	config.UseFrontmatter = true
	exporter := NewExporter(config)
	prog := &language.Program{
		Model: &language.ModelBlock{},
	}
	result := exporter.Export(prog)
	if !strings.Contains(result, "---") {
		t.Errorf("expected frontmatter, got %s", result)
	}
	if !strings.Contains(result, "graph TD") {
		t.Errorf("expected graph TD, got %s", result)
	}
}

func TestFormatLabel(t *testing.T) {
	tests := []struct {
		name        string
		label       string
		id          string
		description string
		tech        string
		expected    string
	}{
		{"all", "Label", "ID", "Desc", "Tech", "Label\n(Tech)\nDesc"},
		{"no desc", "Label", "ID", "", "Tech", "Label\n(Tech)"},
		{"no tech", "Label", "ID", "Desc", "", "Label\nDesc"},
		{"only label", "Label", "ID", "", "", "Label"},
		{"no label", "", "ID", "", "", "ID"},
		{"long desc", "Label", "ID", "This is a very long description that should be truncated by the mermaid exporter to ensure it doesn't break the diagram layout", "", "Label\nThis is a very long description that should be ..."},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := formatLabel(tt.label, tt.id, tt.description, tt.tech)
			if got != tt.expected {
				t.Errorf("formatLabel(%s) = %q, want %q", tt.name, got, tt.expected)
			}
		})
	}
}

func TestSanitizeID(t *testing.T) {
	if sanitizeID("abc-123") != "abc_123" {
		t.Errorf("expected abc_123, got %s", sanitizeID("abc-123"))
	}
}

func mkStr(s string) *string {
	return &s
}

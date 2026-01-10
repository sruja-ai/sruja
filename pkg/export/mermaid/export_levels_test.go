package mermaid

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExporter_ExportL2(t *testing.T) {
	title := "My System"
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind:  "system",
							Name:  "SystemA",
							Title: &title,
							Body: &language.ElementDefBody{
								Items: []*language.BodyItem{
									{
										Element: &language.ElementDef{
											Assignment: &language.ElementAssignment{
												Kind:  "container",
												Name:  "WebApp",
												Title: mkStr("Web Application"),
											},
										},
									},
								},
							},
						},
					},
				},
			},
		},
	}

	config := DefaultConfig()
	config.ViewLevel = 2
	config.TargetID = "SystemA"
	exporter := NewExporter(config)
	result := exporter.Export(prog)

	if result == "" {
		t.Fatal("expected non-empty result for L2 export")
	}
	if !strings.Contains(result, "WebApp") {
		t.Errorf("expected WebApp to be present in L2 export, got %s", result)
	}

	// Test missing target
	config.TargetID = "NonExistent"
	exporter = NewExporter(config)
	result = exporter.Export(prog)
	if result != "" {
		t.Errorf("expected empty string for non-existent L2 target, got %s", result)
	}
}

func TestExporter_ExportL3(t *testing.T) {
	title := "My System"
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind:  "system",
							Name:  "SystemA",
							Title: &title,
							Body: &language.ElementDefBody{
								Items: []*language.BodyItem{
									{
										Element: &language.ElementDef{
											Assignment: &language.ElementAssignment{
												Kind:  "container",
												Name:  "WebApp",
												Title: mkStr("Web Application"),
												Body: &language.ElementDefBody{
													Items: []*language.BodyItem{
														{
															Element: &language.ElementDef{
																Assignment: &language.ElementAssignment{
																	Kind:  "component",
																	Name:  "UI",
																	Title: mkStr("User Interface"),
																},
															},
														},
													},
												},
											},
										},
									},
								},
							},
						},
					},
				},
			},
		},
	}

	config := DefaultConfig()
	config.ViewLevel = 3
	config.TargetID = "SystemA.WebApp"
	exporter := NewExporter(config)
	result := exporter.Export(prog)

	if result == "" {
		t.Fatal("expected non-empty result for L3 export")
	}
	if !strings.Contains(result, "UI") {
		t.Errorf("expected UI to be present in L3 export, got %s", result)
	}

	// Test standalone container L3
	progStandalone := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind: "container",
							Name: "Standalone",
						},
					},
				},
			},
		},
	}
	config.TargetID = "Standalone"
	exporter = NewExporter(config)
	result = exporter.Export(progStandalone)
	if result == "" {
		t.Fatal("expected non-empty result for standalone container L3 export")
	}

	// Test missing target
	config.TargetID = "NonExistent"
	exporter = NewExporter(config)
	result = exporter.Export(prog)
	if result != "" {
		t.Errorf("expected empty string for non-existent L3 target, got %s", result)
	}
}

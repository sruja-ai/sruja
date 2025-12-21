package json

import (
	"encoding/json"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestLikeC4Exporter_Comprehensive(t *testing.T) {
	// Create a comprehensive program with all features
	program := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{
					ElementDef: &language.LikeC4ElementDef{
						Definition: &language.LikeC4Definition{
							Kind:  "system",
							Name:  sPtr("sys1"),
							Title: sPtr("System 1"),
							Body: &language.LikeC4ElementDefBody{
								Items: []*language.LikeC4BodyItem{
									{
										Description: sPtr("Description of Sys1"),
										Technology:  sPtr("Go, React"),
										Metadata: &language.MetadataBlock{
											Entries: []*language.MetaEntry{
												{Key: "owner", Value: sPtr("team-a")},
											},
										},
									},
									{
										Relation: &language.Relation{
											From:  language.QualifiedIdent{Parts: []string{"sys1"}},
											To:    language.QualifiedIdent{Parts: []string{"sys2"}},
											Label: sPtr("uses"),
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
							Kind:  "system",
							Name:  sPtr("sys2"),
							Title: sPtr("System 2"),
							Body: &language.LikeC4ElementDefBody{
								Items: []*language.LikeC4BodyItem{
									{
										Element: &language.LikeC4ElementDef{
											Definition: &language.LikeC4Definition{
												Kind: "container",
												Name: sPtr("api"),
												Body: &language.LikeC4ElementDefBody{
													Items: []*language.LikeC4BodyItem{
														{
															Relation: &language.Relation{
																From:  language.QualifiedIdent{Parts: []string{"sys2", "api"}},
																To:    language.QualifiedIdent{Parts: []string{"sys1"}},
																Label: sPtr("replies"),
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
				{
					Requirement: &language.Requirement{
						ID:          "req1",
						Description: sPtr("Req 1"),
						Type:        sPtr("functional"),
					},
				},
				{
					ADR: &language.ADR{
						ID:    "adr1",
						Title: sPtr("ADR 1"),
						Body: &language.ADRBody{
							Status:       sPtr("accepted"),
							Context:      sPtr("Context..."),
							Decision:     sPtr("Decision..."),
							Consequences: sPtr("Consequences..."),
						},
					},
				},
				{
					Policy: &language.Policy{
						ID:          "pol1",
						Description: "Policy 1",
						Category:    sPtr("security"),
						Enforcement: sPtr("strict"),
					},
				},
				{
					Scenario: &language.Scenario{
						ID:    "sc1",
						Title: sPtr("Scenario 1"),
						Steps: []*language.ScenarioStep{
							{Description: sPtr("Step 1")},
						},
					},
				},
				{
					Flow: &language.Flow{
						ID:    "flow1",
						Title: sPtr("Flow 1"),
					},
				},
				{
					DeploymentNode: &language.DeploymentNode{
						ID:    "node1",
						Label: "Node 1",
					},
				},
				{
					ConstraintsBlock: &language.ConstraintsBlock{
						Entries: []*language.ConstraintEntry{{Key: "c1", Value: "Constraint 1"}},
					},
				},
				{
					ConventionsBlock: &language.ConventionsBlock{
						Entries: []*language.ConventionEntry{{Key: "conv1", Value: "Convention 1"}},
					},
				},
				{
					ContractsBlock: &language.ContractsBlock{
						Contracts: []*language.Contract{{ID: "contract1", Kind: "openapi"}},
					},
				},
				{
					Import: &language.ImportStatement{
						From:     "lib",
						Elements: []string{"Elem1"},
					},
				},
			},
		},
		Views: &language.LikeC4ViewsBlock{
			Items: []*language.LikeC4ViewsItem{
				{
					View: &language.LikeC4ViewDef{
						Name:  sPtr("view1"),
						Title: sPtr("View 1"),
						Body: &language.LikeC4ViewBody{
							Items: []*language.LikeC4ViewItem{
								{
									Include: &language.IncludePredicate{
										Expressions: []language.ViewExpr{
											{Wildcard: true},
											{Selector: sPtr("sys1")},
										},
									},
								},
								{
									Exclude: &language.ExcludePredicate{
										Expressions: []language.ViewExpr{
											{Recursive: true},
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

	exporter := NewLikeC4Exporter()
	jsonOutput, err := exporter.Export(program)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	var dump SrujaModelDump
	if err := json.Unmarshal([]byte(jsonOutput), &dump); err != nil {
		t.Fatalf("Failed to unmarshal exported JSON: %v. Output: %s", err, jsonOutput)
	}

	// Verify extensions
	if dump.Sruja == nil {
		t.Fatal("Expected Sruja extensions")
	}
	if len(dump.Sruja.Requirements) != 1 || dump.Sruja.Requirements[0].ID != "req1" {
		t.Error("Requirement check failed")
	}
	if len(dump.Sruja.ADRs) != 1 || dump.Sruja.ADRs[0].ID != "adr1" {
		t.Error("ADR check failed")
	}
	if len(dump.Sruja.Policies) != 1 || dump.Sruja.Policies[0].ID != "pol1" {
		t.Error("Policy check failed")
	}
	if len(dump.Sruja.Scenarios) != 1 || dump.Sruja.Scenarios[0].ID != "sc1" {
		t.Error("Scenario check failed")
	}
	if len(dump.Sruja.Flows) != 1 || dump.Sruja.Flows[0].ID != "flow1" {
		t.Error("Flow check failed")
	}
	if len(dump.Sruja.Deployments) != 1 || dump.Sruja.Deployments[0].ID != "node1" {
		t.Error("Deployment check failed")
	}
	if len(dump.Sruja.Constraints) != 1 {
		t.Error("Constraints check failed")
	}
	if len(dump.Sruja.Conventions) != 1 {
		t.Error("Conventions check failed")
	}
	if len(dump.Sruja.Contracts) != 1 {
		t.Error("Contracts check failed")
	}
	if len(dump.Sruja.Imports) != 1 {
		t.Error("Imports check failed")
	}

	// Verify View
	if v, ok := dump.Views["view1"]; !ok {
		t.Error("View view1 not found")
	} else {
		if v.Title != "View 1" {
			t.Errorf("Expected view title 'View 1', got %q", v.Title)
		}
		if len(v.Rules) != 2 {
			t.Errorf("Expected 2 view rules, got %d", len(v.Rules))
		}
	}

	// Verify Nested Elements
	if _, ok := dump.Elements["sys2.api"]; !ok {
		t.Error("Nested element sys2.api not found")
	}

	// Verify Relations
	if len(dump.Relations) == 0 {
		t.Error("Relations check failed")
	}
}

func sPtr(s string) *string {
	return &s
}

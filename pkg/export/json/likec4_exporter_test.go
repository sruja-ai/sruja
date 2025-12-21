package json

import (
	"encoding/json"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestLikeC4Exporter_Export(t *testing.T) {
	title := "My System"
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{
					ElementDef: &language.LikeC4ElementDef{
						Definition: &language.LikeC4Definition{
							Kind:  "person",
							Name:  mkStrJSON("User"),
							Title: mkStrJSON("End User"),
						},
					},
				},
				{
					ElementDef: &language.LikeC4ElementDef{
						Definition: &language.LikeC4Definition{
							Kind:  "system",
							Name:  mkStrJSON("SystemA"),
							Title: &title,
							Body: &language.LikeC4ElementDefBody{
								Items: []*language.LikeC4BodyItem{
									{
										Description: mkStrJSON("System description"),
										Technology:  mkStrJSON("Cloud"),
									},
									{
										Element: &language.LikeC4ElementDef{
											Definition: &language.LikeC4Definition{
												Kind:  "container",
												Name:  mkStrJSON("WebApp"),
												Title: mkStrJSON("Web Application"),
											},
										},
									},
								},
							},
						},
					},
				},
				{
					Relation: &language.Relation{
						From:  language.QualifiedIdent{Parts: []string{"User"}},
						To:    language.QualifiedIdent{Parts: []string{"SystemA"}},
						Label: mkStrJSON("Uses"),
					},
				},
			},
		},
	}

	exporter := NewLikeC4Exporter()
	jsonStr, err := exporter.Export(prog)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	var dump SrujaModelDump
	if err := json.Unmarshal([]byte(jsonStr), &dump); err != nil {
		t.Fatalf("failed to unmarshal JSON: %v", err)
	}

	if len(dump.Elements) < 2 {
		t.Errorf("expected at least 2 elements, got %d", len(dump.Elements))
	}
	if dump.Elements["User"].Kind != "person" {
		t.Errorf("expected person, got %s", dump.Elements["User"].Kind)
	}
	if dump.Elements["SystemA.WebApp"].Parent != "SystemA" {
		t.Errorf("expected parent SystemA, got %s", dump.Elements["SystemA.WebApp"].Parent)
	}
	if len(dump.Relations) != 1 {
		t.Errorf("expected 1 relation, got %d", len(dump.Relations))
	}
}

func TestLikeC4Exporter_ExportCompact(t *testing.T) {
	prog := &language.Program{
		Model: &language.ModelBlock{},
	}
	exporter := NewLikeC4Exporter()
	data, err := exporter.ExportCompact(prog)
	if err != nil {
		t.Fatalf("ExportCompact failed: %v", err)
	}
	if len(data) == 0 {
		t.Error("expected non-empty data")
	}
}

func TestLikeC4Exporter_ToModelDump_Nil(t *testing.T) {
	exporter := NewLikeC4Exporter()
	dump := exporter.ToModelDump(nil)
	if dump == nil {
		t.Fatal("expected non-nil dump")
	}
}

func TestMetaToMap(t *testing.T) {
	val := "value"
	meta := []*language.MetaEntry{
		{Key: "key1", Value: &val},
		{Key: "key2", Array: []string{"a", "b"}},
	}
	m := metaToMap(meta)
	if m["key1"] != "value" {
		t.Errorf("expected value, got %s", m["key1"])
	}
	if m["key2"] != "a,b" {
		t.Errorf("expected a,b, got %s", m["key2"])
	}
}

func TestLikeC4Exporter_FullProgram(t *testing.T) {
	viewName := "myview"
	viewTitle := "My View"
	prog := &language.Program{
		Model: &language.ModelBlock{
			Items: []language.ModelItem{
				{
					Requirement: &language.Requirement{
						ID:          "REQ1",
						Description: mkStrJSON("Req1 desc"),
					},
				},
				{
					ADR: &language.ADR{
						ID:    "ADR1",
						Title: mkStrJSON("ADR1 title"),
						Body: &language.ADRBody{
							Status: mkStrJSON("accepted"),
						},
					},
				},
				{
					Policy: &language.Policy{
						ID:          "POL1",
						Description: "Policy desc",
					},
				},
			},
		},
		Views: &language.LikeC4ViewsBlock{
			Items: []*language.LikeC4ViewsItem{
				{
					View: &language.LikeC4ViewDef{
						Name:  &viewName,
						Title: &viewTitle,
					},
				},
			},
		},
	}

	exporter := NewLikeC4Exporter()
	dump := exporter.ToModelDump(prog)

	if len(dump.Sruja.Requirements) != 1 {
		t.Error("expected 1 requirement")
	}
	if len(dump.Sruja.ADRs) != 1 || dump.Sruja.ADRs[0].Status != "accepted" {
		t.Error("expected 1 ADR with accepted status")
	}
	if len(dump.Sruja.Policies) != 1 {
		t.Error("expected 1 policy")
	}
	if _, ok := dump.Views["myview"]; !ok {
		t.Error("expected myview to be present")
	}
}

func TestExtractTechnology(t *testing.T) {
	tech := "Go"
	c := &language.Container{
		Items: []language.ContainerItem{
			{Technology: &tech},
		},
	}
	got := extractTechnology(c)
	if got == nil || *got != "Go" {
		t.Errorf("expected Go, got %v", got)
	}

	c = &language.Container{}
	if extractTechnology(c) != nil {
		t.Error("expected nil")
	}
}

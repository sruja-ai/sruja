package mermaid

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestHelpers(t *testing.T) {
	t.Run("getString nil", func(t *testing.T) {
		if getString(nil) != "" {
			t.Error("expected empty string for nil")
		}
	})

	t.Run("getString value", func(t *testing.T) {
		s := "test"
		if getString(&s) != "test" {
			t.Error("expected 'test'")
		}
	})

	t.Run("escapeQuotes", func(t *testing.T) {
		if escapeQuotes(`He said "hi"`) != `He said #quot;hi#quot;` {
			t.Error("expected quotes escaped")
		}
	})

	t.Run("sanitizeID special", func(t *testing.T) {
		if sanitizeID("hello.world/foo-bar") != "hello_world_foo_bar" {
			t.Errorf("expected underscore replacement, got %s", sanitizeID("hello.world/foo-bar"))
		}
	})
}

func TestExtractSystemsFromModel_Nil(t *testing.T) {
	if extractSystemsFromModel(nil) != nil {
		t.Error("expected nil for nil program")
	}

	prog := &language.Program{}
	if extractSystemsFromModel(prog) != nil {
		t.Error("expected nil for program with nil model")
	}
}

func TestExtractPersonsFromModel(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind:  "person",
							Name:  "TestUser",
							Title: mkStr("Test User"),
						},
					},
				},
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind:  "Person", // uppercase variant
							Name:  "Admin",
							Title: mkStr("Administrator"),
						},
					},
				},
			},
		},
	}

	persons := extractPersonsFromModel(prog)
	if len(persons) != 2 {
		t.Errorf("expected 2 persons, got %d", len(persons))
	}
}

func TestExtractPersonsFromModel_Nil(t *testing.T) {
	if extractPersonsFromModel(nil) != nil {
		t.Error("expected nil")
	}
}

func TestExtractTopLevelContainers(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind:  "container",
							Name:  "ContainerA",
							Title: mkStr("Container A"),
						},
					},
				},
			},
		},
	}

	containers := extractTopLevelContainers(prog)
	if len(containers) != 1 {
		t.Errorf("expected 1 container, got %d", len(containers))
	}
}

func TestExtractTopLevelContainers_Nil(t *testing.T) {
	if extractTopLevelContainers(nil) != nil {
		t.Error("expected nil")
	}
}

func TestExtractRelationsFromModel(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					Relation: &language.Relation{
						From: language.QualifiedIdent{Parts: []string{"A"}},
						To:   language.QualifiedIdent{Parts: []string{"B"}},
					},
				},
				{
					Relation: &language.Relation{
						From: language.QualifiedIdent{Parts: []string{"B"}},
						To:   language.QualifiedIdent{Parts: []string{"C"}},
					},
				},
			},
		},
	}

	relations := extractRelationsFromModel(prog)
	if len(relations) != 2 {
		t.Errorf("expected 2 relations, got %d", len(relations))
	}
}

func TestExtractRelationsFromModel_Nil(t *testing.T) {
	if extractRelationsFromModel(nil) != nil {
		t.Error("expected nil")
	}
}

func TestExtractSystemFromElement_NilOrWrongKind(t *testing.T) {
	if extractSystemFromElement(nil) != nil {
		t.Error("expected nil for nil element")
	}

	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind: "container",
			Name: "Cont",
		},
	}
	if extractSystemFromElement(elem) != nil {
		t.Error("expected nil for non-system element")
	}
}

func TestExtractContainerFromElement_NilOrWrongKind(t *testing.T) {
	if extractContainerFromElement(nil) != nil {
		t.Error("expected nil for nil element")
	}

	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind: "system",
			Name: "Sys",
		},
	}
	if extractContainerFromElement(elem) != nil {
		t.Error("expected nil for non-container element")
	}
}

func TestExtractComponentFromElement(t *testing.T) {
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind:  "component",
			Name:  "Comp",
			Title: mkStr("My Component"),
		},
	}
	comp := extractComponentFromElement(elem)
	if comp == nil || comp.ID != "Comp" {
		t.Error("expected component with ID 'Comp'")
	}
}

func TestExtractComponentFromElement_NilOrWrongKind(t *testing.T) {
	if extractComponentFromElement(nil) != nil {
		t.Error("expected nil")
	}

	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind: "database",
			Name: "DB",
		},
	}
	if extractComponentFromElement(elem) != nil {
		t.Error("expected nil for non-component")
	}
}

func TestExtractDataStoreFromElement(t *testing.T) {
	// Test database kind
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind:  "database",
			Name:  "DB1",
			Title: mkStr("Primary DB"),
		},
	}
	ds := extractDataStoreFromElement(elem)
	if ds == nil || ds.ID != "DB1" {
		t.Error("expected datastore with ID 'DB1'")
	}

	// Test datastore kind
	elem2 := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind:  "datastore",
			Name:  "DS1",
			Title: mkStr("Data Store"),
		},
	}
	ds2 := extractDataStoreFromElement(elem2)
	if ds2 == nil || ds2.ID != "DS1" {
		t.Error("expected datastore with ID 'DS1'")
	}
}

func TestExtractDataStoreFromElement_Nil(t *testing.T) {
	if extractDataStoreFromElement(nil) != nil {
		t.Error("expected nil")
	}
}

func TestExtractQueueFromElement(t *testing.T) {
	elem := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind:  "queue",
			Name:  "Q1",
			Title: mkStr("Message Queue"),
		},
	}
	q := extractQueueFromElement(elem)
	if q == nil || q.ID != "Q1" {
		t.Error("expected queue with ID 'Q1'")
	}

	// Test MessageQueue kind
	elem2 := &language.ElementDef{
		Assignment: &language.ElementAssignment{
			Kind:  "MessageQueue",
			Name:  "MQ1",
			Title: mkStr("Event Queue"),
		},
	}
	q2 := extractQueueFromElement(elem2)
	if q2 == nil || q2.ID != "MQ1" {
		t.Error("expected queue with ID 'MQ1'")
	}
}

func TestExtractQueueFromElement_Nil(t *testing.T) {
	if extractQueueFromElement(nil) != nil {
		t.Error("expected nil")
	}
}

func TestIndexProgram(t *testing.T) {
	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{
					ElementDef: &language.ElementDef{
						Assignment: &language.ElementAssignment{
							Kind: "system",
							Name: "Sys1",
							Body: &language.ElementDefBody{
								Items: []*language.BodyItem{
									{
										Element: &language.ElementDef{
											Assignment: &language.ElementAssignment{
												Kind: "container",
												Name: "Cont1",
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

	idx := indexProgram(prog)
	if idx == nil {
		t.Fatal("expected non-nil index")
	}
	if !idx.ids["Sys1"] {
		t.Error("expected Sys1 to be indexed")
	}
	if !idx.ids["Sys1.Cont1"] {
		t.Error("expected Sys1.Cont1 to be indexed")
	}
}

func TestIndexProgram_Nil(t *testing.T) {
	idx := indexProgram(nil)
	if idx == nil || idx.ids == nil {
		t.Fatal("expected non-nil index even for nil program")
	}
	if len(idx.ids) != 0 {
		t.Error("expected empty ids for nil program")
	}
}

func TestWriteRelation_NoLabel(t *testing.T) {
	exporter := NewExporter(DefaultConfig())
	var sb strings.Builder
	rel := &language.Relation{
		From: language.QualifiedIdent{Parts: []string{"A"}},
		To:   language.QualifiedIdent{Parts: []string{"B"}},
		// No Label or Verb
	}
	exporter.writeRelation(&sb, rel, nil)
	result := sb.String()
	if !strings.Contains(result, "A --> B") {
		t.Errorf("expected simple arrow, got: %s", result)
	}
}

func TestWriteRelation_WithLabel(t *testing.T) {
	exporter := NewExporter(DefaultConfig())
	var sb strings.Builder
	label := "uses"
	rel := &language.Relation{
		From:  language.QualifiedIdent{Parts: []string{"A"}},
		To:    language.QualifiedIdent{Parts: []string{"B"}},
		Label: &label,
	}
	exporter.writeRelation(&sb, rel, nil)
	result := sb.String()
	if !strings.Contains(result, `|"uses"|`) {
		t.Errorf("expected labeled arrow, got: %s", result)
	}
}

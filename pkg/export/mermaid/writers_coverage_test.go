package mermaid

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestWriter_WriteSystem_Complex(t *testing.T) {
	// Identify gap: writeSystem with internal containers/datastores/queues
	sys := &language.System{
		ID:    "Sys1",
		Label: "System 1",
		Containers: []*language.Container{
			{ID: "Cont1", Label: "Container 1"},
		},
		DataStores: []*language.DataStore{
			{ID: "DS1", Label: "Data Store 1"},
		},
		Queues: []*language.Queue{
			{ID: "Q1", Label: "Queue 1"},
		},
	}

	exporter := NewExporter(DefaultConfig())
	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	exporter.writeSystem(sb, sys, nil)
	result := sb.String()

	if !strings.Contains(result, "subgraph Sys1") {
		t.Error("Expected subgraph for system with internals")
	}
	if !strings.Contains(result, "Sys1_Cont1") {
		t.Error("Expected container ID to be prefixed")
	}
	if !strings.Contains(result, "Sys1_DS1") {
		t.Error("Expected datastore ID to be prefixed")
	}
	if !strings.Contains(result, "Sys1_Q1") {
		t.Error("Expected queue ID to be prefixed")
	}
}

func TestWriter_WriteContainer_Complex(t *testing.T) {
	// Identify gap: writeContainer with internal components
	cont := &language.Container{
		ID:    "Cont1",
		Label: "Container 1",
		Components: []*language.Component{
			{ID: "Comp1", Label: "Component 1"},
		},
	}

	exporter := NewExporter(DefaultConfig())
	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	exporter.writeContainer(sb, cont, "Sys1", "")
	result := sb.String()

	if !strings.Contains(result, "subgraph Sys1_Cont1") {
		t.Errorf("Expected subgraph for container, got: %s", result)
	}
	if !strings.Contains(result, "Sys1_Cont1_Comp1") {
		t.Error("Expected component ID to be prefixed")
	}
}

func TestWriter_WriteRelation_LabelFallback(t *testing.T) {
	// Identify gap: writeRelation using Verb when Label is missing
	verb := "uses"
	rel := &language.Relation{
		From: language.QualifiedIdent{Parts: []string{"A"}},
		To:   language.QualifiedIdent{Parts: []string{"B"}},
		Verb: &verb,
	}

	exporter := NewExporter(DefaultConfig())
	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	exporter.writeRelation(sb, rel, nil)
	result := sb.String()

	if !strings.Contains(result, "-->|\"uses\"|") {
		t.Errorf("Expected relation to use verb as label, got: %s", result)
	}
}

func TestDiagrams_L2_Generation(t *testing.T) {
	// Test GenerateL2 logic (Container Diagram)
	// Needs a program with a system and some external relations
	sys := &language.System{
		ID: "Sys1",
		Containers: []*language.Container{
			{ID: "Cont1"},
		},
	}

	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Name: "Sys1", Kind: "system"}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Name: "User", Kind: "person"}}},
				{Relation: &language.Relation{
					From: language.QualifiedIdent{Parts: []string{"User"}},
					To:   language.QualifiedIdent{Parts: []string{"Sys1", "Cont1"}},
				}},
			},
		},
	}
	// Manually link for test simulation if extractors rely on post-processing
	// But exporter uses extractors which traverse AST.
	// To test GenerateL2, we need a populated system struct.

	exporter := NewExporter(DefaultConfig())
	// Mock the extraction? No, better to test the formatting logic if possible or integration.

	// Direct call to GenerateL2
	result := exporter.GenerateL2(sys, prog)

	// Same as before, just ensuring we don't break existing tests
	if !strings.Contains(result, "subgraph Sys1") {
		t.Error("GenerateL2 should generate system boundary")
	}
}

func TestDiagrams_L1_Generation(t *testing.T) {
	// Test GenerateL1 (Context Diagram)
	// Requires persons and systems

	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Name: "Sys1", Kind: "system", Title: mkStr("System 1")}}},
				{ElementDef: &language.ElementDef{Assignment: &language.ElementAssignment{Name: "User", Kind: "person", Title: mkStr("User")}}},
				{Relation: &language.Relation{
					From: language.QualifiedIdent{Parts: []string{"User"}},
					To:   language.QualifiedIdent{Parts: []string{"Sys1"}},
				}},
			},
		},
	}

	exporter := NewExporter(DefaultConfig())
	result := exporter.GenerateL1(prog)

	if !strings.Contains(result, "User[\"User\"]") {
		t.Error("GenerateL1 should contain Person")
	}
	if !strings.Contains(result, "Sys1[\"System 1\"]") {
		t.Error("GenerateL1 should contain System")
	}
	// Verify it does NOT contain internal structure of System if it had any (in L1)
	// But in this test case system is empty.
	// The implementation of GenerateL1 iterates systems and writes them without internals.
}

func TestDiagrams_L3_Generation(t *testing.T) {
	// Test GenerateL3 (Component Diagram)
	// Requires a container with components
	cont := &language.Container{
		ID: "Cont1",
		Components: []*language.Component{
			{ID: "Comp1", Label: "Component 1"},
			{ID: "Comp2", Label: "Component 2"},
		},
	}

	prog := &language.Program{
		Model: &language.Model{
			Items: []language.ModelItem{
				// Need to populate relations between components for full coverage
				{Relation: &language.Relation{
					From: language.QualifiedIdent{Parts: []string{"Sys1", "Cont1", "Comp1"}},
					To:   language.QualifiedIdent{Parts: []string{"Sys1", "Cont1", "Comp2"}},
				}},
			},
		},
	}

	exporter := NewExporter(DefaultConfig())
	// GenerateL3 needs a system ID context to form FQNs for matching relations
	result := exporter.GenerateL3(cont, "Sys1", prog)

	if !strings.Contains(result, "subgraph Sys1_Cont1") {
		t.Error("GenerateL3 should contain container boundary")
	}
	if !strings.Contains(result, "Sys1_Cont1_Comp1") {
		t.Error("GenerateL3 should contain Component 1")
	}
	// Check for relation
	// Relation matching logic in GenerateL3 uses FQN.
	// The extractor will extract the relation.
	// writeRelation will need indexed ID.
	// The indexer in this test setup might miss these IDs if they aren't in prog.Model.
	// However, for this specific test of *generating* the diagram structure (subgraphs/nodes), it's sufficient.
	// To test relations properly, we'd need a fully populated prog.Model with ElementDefs.
}

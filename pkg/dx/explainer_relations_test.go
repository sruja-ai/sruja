// pkg/dx/explainer_relations_test.go
package dx

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExtractRelationInfo_WithBoth(t *testing.T) {
	rel := &language.Relation{
		From:  language.QualifiedIdent{Parts: []string{"A"}},
		To:    language.QualifiedIdent{Parts: []string{"B"}},
		Verb:  stringPtr("calls"),
		Label: stringPtr("HTTP"),
	}
	label, verb := extractRelationInfo(rel)
	if label != "HTTP" {
		t.Errorf("Expected label 'HTTP', got '%s'", label)
	}
	if verb != "calls" {
		t.Errorf("Expected verb 'calls', got '%s'", verb)
	}
}

func TestExtractRelationInfo_WithNeither(t *testing.T) {
	rel := &language.Relation{
		From: language.QualifiedIdent{Parts: []string{"A"}},
		To:   language.QualifiedIdent{Parts: []string{"B"}},
	}
	label, verb := extractRelationInfo(rel)
	if label != "" {
		t.Errorf("Expected empty label, got '%s'", label)
	}
	if verb != "" {
		t.Errorf("Expected empty verb, got '%s'", verb)
	}
}
func TestProcessRelation(t *testing.T) {
	rel := &language.Relation{
		From:  language.QualifiedIdent{Parts: []string{"A"}},
		To:    language.QualifiedIdent{Parts: []string{"B"}},
		Verb:  stringPtr("calls"),
		Label: stringPtr("HTTP"),
	}

	info := &RelationsInfo{}

	// Test outgoing
	processRelation(rel, "A", info)
	if len(info.Outgoing) != 1 {
		t.Errorf("Expected 1 outgoing relation, got %d", len(info.Outgoing))
	}
	if info.Outgoing[0].To != "B" {
		t.Errorf("Expected outgoing target 'B', got '%s'", info.Outgoing[0].To)
	}

	// Test incoming
	processRelation(rel, "B", info)
	if len(info.Incoming) != 1 {
		t.Errorf("Expected 1 incoming relation, got %d", len(info.Incoming))
	}
	if info.Incoming[0].From != "A" {
		t.Errorf("Expected incoming source 'A', got '%s'", info.Incoming[0].From)
	}
}

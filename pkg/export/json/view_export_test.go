package json

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestBuildQualifiedIDForView(t *testing.T) {
	tests := []struct {
		name     string
		parts    []string
		expected string
	}{
		{"empty", []string{}, ""},
		{"single", []string{"A"}, "A"},
		{"multiple", []string{"A", "B", "C"}, "A.B.C"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := buildQualifiedIDForView(tt.parts...)
			if got != tt.expected {
				t.Errorf("buildQualifiedIDForView() = %v, want %v", got, tt.expected)
			}
		})
	}
}

func TestBuildEdgeID(t *testing.T) {
	got := buildEdgeID("A", "B", 1)
	expected := "edge-A-B-1"
	if got != expected {
		t.Errorf("expected %s, got %s", expected, got)
	}

	got = buildEdgeID("X", "Y", 0)
	expected = "edge-X-Y-0"
	if got != expected {
		t.Errorf("expected %s, got %s", expected, got)
	}
}

func TestLabelOrID(t *testing.T) {
	if labelOrID("Label", "ID") != "Label" {
		t.Error("expected Label")
	}
	if labelOrID("", "ID") != "ID" {
		t.Error("expected ID")
	}
}

func TestPtrStrPrint(t *testing.T) {
	s := "test"
	if ptrStr(&s) != "test" {
		t.Error("expected test")
	}
	if ptrStr(nil) != "" {
		t.Error("expected empty string")
	}
}

func TestGetRelationLabel(t *testing.T) {
	label := "label"
	verb := "verb"

	r := &language.Relation{Label: &label, Verb: &verb}
	if getRelationLabel(r) != "label" {
		t.Error("expected label")
	}

	r = &language.Relation{Verb: &verb}
	if getRelationLabel(r) != "verb" {
		t.Error("expected verb")
	}

	r = &language.Relation{}
	if getRelationLabel(r) != "" {
		t.Error("expected empty string")
	}
}

func TestGetContainerTechnology(t *testing.T) {
	tech := "Go"
	c := &language.Container{
		Items: []language.ContainerItem{
			{Technology: &tech},
		},
	}
	if getContainerTechnology(c) != "Go" {
		t.Error("expected Go")
	}

	c = &language.Container{
		Items: []language.ContainerItem{
			{Description: mkStrJSON("desc")},
		},
	}
	if getContainerTechnology(c) != "" {
		t.Error("expected empty string")
	}
}

func TestGetLastPart(t *testing.T) {
	if getLastPart("A.B.C") != "C" {
		t.Error("expected C")
	}
	if getLastPart("A") != "A" {
		t.Error("expected A")
	}
}

func TestGetL1ID(t *testing.T) {
	if getL1ID("A.B.C") != "A" {
		t.Error("expected A")
	}
	if getL1ID("X") != "X" {
		t.Error("expected X")
	}
}

func mkStrJSON(s string) *string {
	return &s
}

package json

import (
    "testing"
    "github.com/sruja-ai/sruja/pkg/language"
)

func TestGenerateViews_LabelFallbacksAndEdges(t *testing.T) {
    // Persons and systems
    p := &language.Person{ID: "U", Label: "User"}
    s := &language.System{ID: "S", Label: "Sys"}
    c := &language.Container{ID: "C", Label: "Cont"}
    s.Containers = []*language.Container{c}
    arch := &language.Architecture{Systems: []*language.System{s}, Persons: []*language.Person{p}}

    // Top-level relation with only verb
    verb := "uses"
    arch.Relations = []*language.Relation{{From: language.QualifiedIdent{Parts: []string{"U"}}, To: language.QualifiedIdent{Parts: []string{"S"}}, Verb: &verb}}

    views := GenerateViews(arch)
    if len(views.L1.Edges) == 0 { t.Fatalf("expected L1 edges") }
    // Label fallback should use verb when label is empty
    found := false
    for _, e := range views.L1.Edges { if e.Label == "uses" { found = true; break } }
    if !found { t.Fatalf("expected edge label 'uses' in L1 edges") }

    // L2 for system S should exist
    if _, ok := views.L2["S"]; !ok { t.Fatalf("expected L2 view for S") }
    // L3 for S.C should exist
    if _, ok := views.L3["S.C"]; !ok { t.Fatalf("expected L3 view for S.C") }
}


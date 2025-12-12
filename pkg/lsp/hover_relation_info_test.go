package lsp

import (
    "testing"
    "github.com/sruja-ai/sruja/pkg/language"
)

func TestFindRelationInfo_ArchitectureLevel(t *testing.T) {
    verb := "uses"
    label := "HTTP"
    arch := &language.Architecture{
        Relations: []*language.Relation{
            {From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}, Verb: &verb, Label: &label},
        },
    }
    v, l := findRelationInfo(arch, "A", "B")
    if v != verb || l != label {
        t.Fatalf("expected verb=%q label=%q, got verb=%q label=%q", verb, label, v, l)
    }
}

func TestFindRelationInfo_SystemAndContainerLevel(t *testing.T) {
    verb := "reads"
    s := &language.System{ID: "S"}
    c := &language.Container{ID: "C"}
    q := &language.Queue{ID: "Q"}
    c.Relations = []*language.Relation{{From: language.QualifiedIdent{Parts: []string{"S", "C"}}, To: language.QualifiedIdent{Parts: []string{"S", "Q"}}, Verb: &verb}}
    s.Containers = []*language.Container{c}
    s.Queues = []*language.Queue{q}
    arch := &language.Architecture{Systems: []*language.System{s}}
    v, l := findRelationInfo(arch, "S.C", "S.Q")
    if v != verb || l != "" {
        t.Fatalf("expected verb=%q label='', got verb=%q label=%q", verb, v, l)
    }
}

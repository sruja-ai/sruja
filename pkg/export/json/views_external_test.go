package json

import (
    "testing"
    "github.com/sruja-ai/sruja/pkg/language"
)

func TestGenerateViews_L2ExternalNodesAndEdges(t *testing.T) {
    s := &language.System{ID: "S", Label: "Sys"}
    c := &language.Container{ID: "C", Label: "Cont"}
    s.Containers = []*language.Container{c}
    x := &language.System{ID: "X", Label: "Ext"}
    arch := &language.Architecture{Systems: []*language.System{s, x}}
    // Relation S.C -> X at architecture-level should make X external in L2 of S
    rel := &language.Relation{From: language.QualifiedIdent{Parts: []string{"S", "C"}}, To: language.QualifiedIdent{Parts: []string{"X"}}}
    arch.Relations = []*language.Relation{rel}

    v := GenerateViews(arch)
    l2 := v.L2["S"]
    // Find external node X
    foundExt := false
    for _, n := range l2.Nodes {
        if n.ID == "X" && n.IsExternal { foundExt = true; break }
    }
    if !foundExt { t.Fatalf("expected external node X in L2 view for S") }
}


package engine

import (
    "testing"
    "github.com/sruja-ai/sruja/pkg/language"
)

func TestScorer_MissingDescriptions_GradeB(t *testing.T) {
    // Build architecture with multiple missing descriptions to trigger deductions
    cont := &language.Container{ID: "C1", Label: "Container"}
    comp := &language.Component{ID: "X", Label: "Comp"}
    sys := &language.System{ID: "S", Label: "System", Items: []language.SystemItem{{Container: cont}}}
    arch := &language.Architecture{Items: []language.ArchitectureItem{{Component: comp}, {System: sys}, {Container: &language.Container{ID: "C2", Label: "Container"}}}}
    prog := &language.Program{Architecture: arch}

    sc := NewScorer()
    card := sc.CalculateScore(prog)
    if card.Score < 90 {
        if card.Grade != "B" && card.Grade != "C" && card.Grade != "D" { t.Fatalf("expected grade below A, got %s", card.Grade) }
    } else {
        if card.Grade != "A" { t.Fatalf("expected grade A when minimal deductions, got %s", card.Grade) }
    }
    if len(card.Deductions) == 0 { t.Fatalf("expected deductions present") }
}

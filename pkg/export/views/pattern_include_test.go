package views

import (
    "testing"
    "github.com/sruja-ai/sruja/pkg/language"
)

func TestApplyViewExpressions_PatternWildcard(t *testing.T) {
    arch := &language.Architecture{Systems: []*language.System{{ID: "S", Label: "Sys", Containers: []*language.Container{{ID: "C", Label: "Cont"}}}}}
    pattern := "->Element->"
    v := &language.View{Type: "container", Scope: language.QualifiedIdent{Parts: []string{"S"}}, Expressions: []*language.ViewExpression{{Type: "include", Pattern: &pattern}}}
    inc, err := ApplyViewExpressions(arch, v)
    if err != nil { t.Fatalf("unexpected error: %v", err) }
    if !inc["S.C"] { t.Fatalf("expected S.C included by pattern wildcard") }
}


package views

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestApplyViewExpressions_PatternWildcard(t *testing.T) {
	dsl := `model {
        S = system "Sys" {
            C = container "Cont"
        }
    }`
	prog := parseDSL(t, dsl)
	pattern := "->Element->"
	v := &language.View{Type: "container", Scope: language.QualifiedIdent{Parts: []string{"S"}}, Expressions: []*language.ViewExpression{{Type: "include", Pattern: &pattern}}}
	inc, err := ApplyViewExpressions(prog, v)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !inc["S.C"] {
		t.Fatalf("expected S.C included by pattern wildcard")
	}
}

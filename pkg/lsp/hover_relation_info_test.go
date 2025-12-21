package lsp

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func parseDSL(t *testing.T, dsl string) *language.Program {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	prog, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}
	return prog
}

func TestFindRelationInfo_ArchitectureLevel(t *testing.T) {
	dsl := `model {
        A = system "A"
        B = system "B"
        A -> B "uses" "HTTP"
    }`
	prog := parseDSL(t, dsl)

	v, l := findRelationInfoInModel(prog.Model, "A", "B")
	if v != "uses" || l != "HTTP" {
		t.Fatalf("expected verb=%q label=%q, got verb=%q label=%q", "uses", "HTTP", v, l)
	}
}

func TestFindRelationInfo_SystemAndContainerLevel(t *testing.T) {
	dsl := `model {
        S = system "System" {
            C = container "Container"
            Q = queue "Queue"
            C -> Q "reads"
        }
    }`
	prog := parseDSL(t, dsl)

	v, l := findRelationInfoInModel(prog.Model, "S.C", "S.Q")
	if v != "reads" || l != "" {
		t.Fatalf("expected verb=%q label='', got verb=%q label=%q", "reads", v, l)
	}
}

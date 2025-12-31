package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestScorer_MissingDescriptions_GradeB(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	// Build architecture with multiple missing descriptions to trigger deductions
	dsl := `	S = system "System" {
		C1 = container "Container"
	}
	C2 = container "Container"
	X = component "Comp"`

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	sc := NewScorer()
	card := sc.CalculateScore(program)
	if card.Score < 90 {
		if card.Grade != "B" && card.Grade != "C" && card.Grade != "D" {
			t.Fatalf("expected grade below A, got %s", card.Grade)
		}
	} else {
		if card.Grade != "A" {
			t.Fatalf("expected grade A when minimal deductions, got %s", card.Grade)
		}
	}
	if len(card.Deductions) == 0 {
		t.Fatalf("expected deductions present")
	}
}

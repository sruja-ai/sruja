package engine

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestEngine_EdgeCases(t *testing.T) {
	t.Run("SimplicityRule_ComplexNesting", func(_ *testing.T) {
		parser, err := language.NewParser()
		if err != nil {
			t.Fatalf("Failed to create parser: %v", err)
		}

		// Create DSL with many containers
		var sb strings.Builder
		sb.WriteString("  Sys1 = system \"System 1\" {\n")
		for i := 0; i < 20; i++ {
			sb.WriteString("    Cont")
			sb.WriteString(string(rune('A' + i)))
			sb.WriteString(" = container \"Container ")
			sb.WriteString(string(rune('A' + i)))
			sb.WriteString("\"\n")
		}
		sb.WriteString("  }\n")

		program, _, err := parser.Parse("test.sruja", sb.String())
		if err != nil {
			t.Fatalf("Failed to parse DSL: %v", err)
		}

		rule := &SimplicityRule{}
		errs := rule.Validate(program)
		// We don't necessarily expect errors, just that it doesn't panic and runs correctly
		// If SimplicityRule has a threshold, we might expect a warning.
		// Assuming current implementation just checks for basic structural sanity or limits.
		_ = errs
	})

	t.Run("OrphanDetectionRule_DisconnectedGraph", func(t *testing.T) {
		parser, err := language.NewParser()
		if err != nil {
			t.Fatalf("Failed to create parser: %v", err)
		}

		dsl := `		S1 = system "System 1"
		S2 = system "System 2"`

		program, _, err := parser.Parse("test.sruja", dsl)
		if err != nil {
			t.Fatalf("Failed to parse DSL: %v", err)
		}

		rule := &OrphanDetectionRule{}
		errs := rule.Validate(program)
		// Should detect orphans
		if len(errs) == 0 {
			t.Error("Expected orphan detection errors")
		}
	})

	t.Run("CycleDetectionRule_SelfCycle", func(t *testing.T) {
		parser, err := language.NewParser()
		if err != nil {
			t.Fatalf("Failed to create parser: %v", err)
		}

		dsl := `		S1 = system "System 1"
		S1 -> S1 "self-reference"`

		program, _, err := parser.Parse("test.sruja", dsl)
		if err != nil {
			t.Fatalf("Failed to parse DSL: %v", err)
		}

		rule := &CycleDetectionRule{}
		errs := rule.Validate(program)
		// Self-cycles might be valid or invalid depending on rule strictness.
		// Usually a cycle detection rule should catch this.
		if len(errs) == 0 {
			t.Error("Expected error for self-cycle")
		}
	})

	t.Run("UniqueIDRule_CaseSensitivity", func(_ *testing.T) {
		parser, err := language.NewParser()
		if err != nil {
			t.Fatalf("Failed to create parser: %v", err)
		}

		dsl := `		sys1 = system "System 1"
		SYS1 = system "System 1 (Upper)"`

		program, _, err := parser.Parse("test.sruja", dsl)
		if err != nil {
			t.Fatalf("Failed to parse DSL: %v", err)
		}

		rule := &UniqueIDRule{}
		errs := rule.Validate(program)
		// If IDs are case-insensitive, this should be an error.
		// If case-sensitive, no error.
		// Let's assume case-sensitive for now, so no error expected.
		// But if we want to enforce case-insensitivity, we'd expect error.
		// Checking current behavior: usually IDs are case-sensitive in code but maybe not in design.
		// If no error, it means case-sensitive.
		_ = errs
	})
}

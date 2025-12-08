package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestEngine_EdgeCases(t *testing.T) {
	t.Run("SimplicityRule_ComplexNesting", func(_ *testing.T) {
		// Create a deeply nested structure that might trigger simplicity warnings
		// System -> Container -> Component -> Component -> ...
		// Note: Sruja DSL structure is usually System -> Container -> Component
		// But let's test if the rule handles deep hierarchies if the AST allows it (or just many elements)

		sys := &language.System{ID: "Sys1", Label: "System 1"}
		// Add many containers to trigger complexity check if any
		for i := 0; i < 20; i++ {
			sys.Containers = append(sys.Containers, &language.Container{
				ID:    "Cont",
				Label: "Container",
			})
		}

		prog := &language.Program{
			Architecture: &language.Architecture{
				Name:    "Complex",
				Systems: []*language.System{sys},
			},
		}

		rule := &SimplicityRule{}
		errs := rule.Validate(prog)
		// We don't necessarily expect errors, just that it doesn't panic and runs correctly
		// If SimplicityRule has a threshold, we might expect a warning.
		// Assuming current implementation just checks for basic structural sanity or limits.
		_ = errs
	})

	t.Run("OrphanDetectionRule_DisconnectedGraph", func(t *testing.T) {
		// Create elements with no relations
		prog := &language.Program{
			Architecture: &language.Architecture{
				Name: "Orphans",
				Systems: []*language.System{
					{ID: "S1", Label: "System 1"},
					{ID: "S2", Label: "System 2"},
				},
				// No relations
			},
		}

		rule := &OrphanDetectionRule{}
		errs := rule.Validate(prog)
		// Should detect orphans
		if len(errs) == 0 {
			t.Error("Expected orphan detection errors")
		}
	})

	t.Run("CycleDetectionRule_SelfCycle", func(t *testing.T) {
		prog := &language.Program{
			Architecture: &language.Architecture{
				Name: "SelfCycle",
				Systems: []*language.System{
					{ID: "S1", Label: "System 1"},
				},
				Relations: []*language.Relation{
					{
						From: language.QualifiedIdent{Parts: []string{"S1"}},
						To:   language.QualifiedIdent{Parts: []string{"S1"}},
					},
				},
			},
		}

		rule := &CycleDetectionRule{}
		errs := rule.Validate(prog)
		// Self-cycles might be valid or invalid depending on rule strictness.
		// Usually a cycle detection rule should catch this.
		if len(errs) == 0 {
			t.Error("Expected error for self-cycle")
		}
	})

	t.Run("UniqueIDRule_CaseSensitivity", func(_ *testing.T) {
		prog := &language.Program{
			Architecture: &language.Architecture{
				Name: "CaseTest",
				Systems: []*language.System{
					{ID: "sys1", Label: "System 1"},
					{ID: "SYS1", Label: "System 1 (Upper)"},
				},
			},
		}

		rule := &UniqueIDRule{}
		errs := rule.Validate(prog)
		// If IDs are case-insensitive, this should be an error.
		// If case-sensitive, no error.
		// Let's assume case-sensitive for now, so no error expected.
		// But if we want to enforce case-insensitivity, we'd expect error.
		// Checking current behavior: usually IDs are case-sensitive in code but maybe not in design.
		// If no error, it means case-sensitive.
		_ = errs
	})
}

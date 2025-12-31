package language

import (
	"testing"
)

func TestWorkspace_ResolveAndMergeImports_Wildcard(t *testing.T) {
	ws := NewWorkspace()

	// Simulate stdlib program with kinds
	stdlibProg := &Program{
		Specification: &Specification{
			Items: []SpecificationItem{
				{Element: &ElementKindDef{Name: "person", Title: strPtr("Person")}},
				{Element: &ElementKindDef{Name: "system", Title: strPtr("System")}},
				{Tag: &TagDef{Name: "external", Title: strPtr("External")}},
			},
		},
	}
	ws.AddProgram("sruja.ai/stdlib/core.sruja", stdlibProg, nil)

	// Program that imports from stdlib
	mainProg := &Program{
		Model: &Model{
			Items: []ModelItem{
				{
					Import: &ImportStatement{
						Elements: []string{"*"},
						From:     "sruja.ai/stdlib",
					},
				},
			},
		},
	}
	ws.AddProgram("main.sruja", mainProg, nil)

	// Run import merging
	ws.ResolveAndMergeImports()

	// Check that kinds were merged
	if mainProg.Specification == nil {
		t.Fatal("Expected Specification to be created")
	}
	if len(mainProg.Specification.Items) != 3 {
		t.Errorf("Expected 3 specification items, got %d", len(mainProg.Specification.Items))
	}

	// Verify specific items exist
	names := make(map[string]bool)
	for _, item := range mainProg.Specification.Items {
		names[ws.getSpecificationItemName(item)] = true
	}
	if !names["person"] {
		t.Error("Expected 'person' kind to be imported")
	}
	if !names["system"] {
		t.Error("Expected 'system' kind to be imported")
	}
	if !names["external"] {
		t.Error("Expected 'external' tag to be imported")
	}
}

func TestWorkspace_ResolveAndMergeImports_Named(t *testing.T) {
	ws := NewWorkspace()

	// Simulate stdlib program with kinds
	stdlibProg := &Program{
		Specification: &Specification{
			Items: []SpecificationItem{
				{Element: &ElementKindDef{Name: "person", Title: strPtr("Person")}},
				{Element: &ElementKindDef{Name: "system", Title: strPtr("System")}},
				{Element: &ElementKindDef{Name: "container", Title: strPtr("Container")}},
			},
		},
	}
	ws.AddProgram("sruja.ai/stdlib/core.sruja", stdlibProg, nil)

	// Program that imports specific items from stdlib
	mainProg := &Program{
		Model: &Model{
			Items: []ModelItem{
				{
					Import: &ImportStatement{
						Elements: []string{"person", "system"},
						From:     "sruja.ai/stdlib",
					},
				},
			},
		},
	}
	ws.AddProgram("main.sruja", mainProg, nil)

	// Run import merging
	ws.ResolveAndMergeImports()

	// Check that only requested kinds were merged
	if mainProg.Specification == nil {
		t.Fatal("Expected Specification to be created")
	}
	if len(mainProg.Specification.Items) != 2 {
		t.Errorf("Expected 2 specification items, got %d", len(mainProg.Specification.Items))
	}

	// Verify only requested items exist
	names := make(map[string]bool)
	for _, item := range mainProg.Specification.Items {
		names[ws.getSpecificationItemName(item)] = true
	}
	if !names["person"] {
		t.Error("Expected 'person' kind to be imported")
	}
	if !names["system"] {
		t.Error("Expected 'system' kind to be imported")
	}
	if names["container"] {
		t.Error("'container' should NOT be imported (not requested)")
	}
}

func TestWorkspace_ResolveAndMergeImports_CircularSafe(t *testing.T) {
	ws := NewWorkspace()

	// Program A imports from B
	progA := &Program{
		Specification: &Specification{
			Items: []SpecificationItem{
				{Element: &ElementKindDef{Name: "typeA"}},
			},
		},
		Model: &Model{
			Items: []ModelItem{
				{
					Import: &ImportStatement{
						Elements: []string{"*"},
						From:     "./b.sruja",
					},
				},
			},
		},
	}

	// Program B imports from A (circular)
	progB := &Program{
		Specification: &Specification{
			Items: []SpecificationItem{
				{Element: &ElementKindDef{Name: "typeB"}},
			},
		},
		Model: &Model{
			Items: []ModelItem{
				{
					Import: &ImportStatement{
						Elements: []string{"*"},
						From:     "./a.sruja",
					},
				},
			},
		},
	}

	ws.AddProgram("/project/a.sruja", progA, nil)
	ws.AddProgram("/project/b.sruja", progB, nil)

	// This should not panic or infinite loop
	ws.ResolveAndMergeImports()

	// Test passes if we get here without hanging
	t.Log("Circular import detection works")
}

func TestWorkspace_ResolveAndMergeImports_NoImports(t *testing.T) {
	ws := NewWorkspace()

	// Program with no imports
	prog := &Program{
		Specification: &Specification{
			Items: []SpecificationItem{
				{Element: &ElementKindDef{Name: "myKind"}},
			},
		},
		Model: &Model{
			Items: []ModelItem{
				{ElementDef: &ElementDef{Assignment: &ElementAssignment{Name: "test", Kind: "myKind"}}},
			},
		},
	}
	ws.AddProgram("test.sruja", prog, nil)

	// Run import merging (should be a no-op)
	ws.ResolveAndMergeImports()

	// Original spec items should be unchanged
	if len(prog.Specification.Items) != 1 {
		t.Errorf("Expected 1 specification item, got %d", len(prog.Specification.Items))
	}
}

func strPtr(s string) *string {
	return &s
}

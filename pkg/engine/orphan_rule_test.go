// pkg/engine/orphan_rule_test.go
package engine_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

const expectedOrphanMessage = "Orphan element 'Orphan' is defined but never used in any relation."

func TestOrphanDetectionRule_EmptyArchitecture(t *testing.T) {
	program := &language.Program{
		Architecture: nil,
	}

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for nil architecture, got %d", len(errs))
	}
}

func TestOrphanDetectionRule_EmptySystems(t *testing.T) {
	program := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{},
		},
	}

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) != 0 {
		t.Errorf("Expected 0 errors for empty systems, got %d", len(errs))
	}
}

func TestOrphanDetectionRule_OrphanSystem(t *testing.T) {
	dsl := `
architecture "Test" {
    system Orphan "Orphan System" {}
}
`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	if len(errs) == 0 {
		t.Error("Expected orphan error for unused system")
	}
	found := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element") && strings.Contains(e.Message, "never used") {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected orphan error message for 'Orphan' system")
	}
}

func TestOrphanDetectionRule_OrphanContainer(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Orphan "Orphan Container" {}
    }
}
`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	foundOrphan := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element") && strings.Contains(e.Message, "never used") {
			foundOrphan = true
			break
		}
	}
	if !foundOrphan {
		t.Error("Expected orphan error for unused container")
	}
}

func TestOrphanDetectionRule_OrphanComponent(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Cont "Container" {
            component Orphan "Orphan Component" {}
        }
    }
}
`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	foundOrphan := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element") && strings.Contains(e.Message, "never used") {
			foundOrphan = true
			break
		}
	}
	if !foundOrphan {
		t.Error("Expected orphan error for unused component")
	}
}

func TestOrphanDetectionRule_OrphanDataStore(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        datastore Orphan "Orphan DB" {}
    }
}
`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	foundOrphan := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element") && strings.Contains(e.Message, "never used") {
			foundOrphan = true
			break
		}
	}
	if !foundOrphan {
		t.Error("Expected orphan error for unused datastore")
	}
}

func TestOrphanDetectionRule_OrphanQueue(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        queue Orphan "Orphan Queue" {}
    }
}
`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	foundOrphan := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element 'Orphan'") && strings.Contains(e.Message, "never used") {
			foundOrphan = true
			break
		}
	}
	if !foundOrphan {
		t.Error("Expected orphan error for unused queue")
	}
}

func TestOrphanDetectionRule_OrphanPerson(t *testing.T) {
	dsl := `
architecture "Test" {
    person Orphan "Orphan Person" {}
}
`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	foundOrphan := false
	for _, e := range errs {
		// Updated message format includes additional context
		if strings.Contains(e.Message, "Orphan element") && strings.Contains(e.Message, "never used") {
			foundOrphan = true
			break
		}
	}
	if !foundOrphan {
		t.Error("Expected orphan error for unused person")
	}
}

func TestOrphanDetectionRule_SystemLevelRelations(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Cont "Container" {}
        Sys -> Cont "Uses"
    }
}
`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	for _, e := range errs {
		if e.Message == "Orphan element 'Sys' is defined but never used in any relation." {
			t.Error("System should not be orphan when it has relations")
		}
	}
}

func TestOrphanDetectionRule_ContainerLevelRelations(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Cont "Container" {
            component Comp "Component" {}
            Cont -> Comp "Uses"
        }
    }
}
`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	for _, e := range errs {
		if e.Message == "Orphan element 'Cont' is defined but never used in any relation." {
			t.Error("Container should not be orphan when it has relations")
		}
	}
}

func TestOrphanDetectionRule_ComponentLevelRelations(t *testing.T) {
	dsl := `
architecture "Test" {
    system Sys "System" {
        container Cont "Container" {
            component Comp1 "Component 1" {}
            component Comp2 "Component 2" {}
            Comp1 -> Comp2 "Uses"
        }
    }
}
`
	program := parse(t, dsl)

	rule := &engine.OrphanDetectionRule{}
	errs := rule.Validate(program)
	for _, e := range errs {
		if e.Message == "Orphan element 'Comp1' is defined but never used in any relation." {
			t.Error("Component should not be orphan when it has relations")
		}
	}
}

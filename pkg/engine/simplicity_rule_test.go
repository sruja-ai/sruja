package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestSimplicityRule_DomainForDeployment(t *testing.T) {
	// DDD features (DomainBlock) removed - deferred to Phase 2
	t.Skip("DDD features removed - deferred to Phase 2")

	// rule := &SimplicityRule{}
	// prog := &language.Program{
	// 	Architecture: &language.Architecture{
	// 		Domains: []*language.DomainBlock{
	// 			{
	// 				ID: "ECommerce",
	// 				Components: []*language.Component{
	// 					{ID: "WebApp"},
	// 					{ID: "Database"},
	// 				},
	// 			},
	// 		},
	// 	},
	// }
	// errors := rule.Validate(prog)
	// ...
}

func TestSimplicityRule_SystemForDomain(t *testing.T) {
	// DDD features (ContextBlock, Aggregate) removed - deferred to Phase 2
	t.Skip("DDD features removed - deferred to Phase 2")

	// rule := &SimplicityRule{}
	// prog := &language.Program{
	// 	Architecture: &language.Architecture{
	// 		Systems: []*language.System{
	// 			{
	// 				ID: "ShopAPI",
	// 				Contexts: []*language.ContextBlock{
	// 					{
	// 						ID: "Orders",
	// 						Aggregates: []*language.Aggregate{
	// 							{ID: "Order"},
	// 						},
	// 					},
	// 				},
	// 			},
	// 		},
	// 	},
	// }
	// errors := rule.Validate(prog)
	// ...
}

func TestSimplicityRule_ValidUsage(t *testing.T) {
	rule := &SimplicityRule{}

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	// Valid: System with containers (deployment modeling)
	dsl1 := `
		ShopAPI = system "Shop API" {
			WebApp = container "Web App"
			Database = container "Database"
		}
	`

	program1, _, err := parser.Parse("test.sruja", dsl1)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	errors1 := rule.Validate(program1)
	if len(errors1) > 0 {
		t.Errorf("Expected no errors for valid system usage, got: %v", errors1)
	}

	// DDD features removed - DomainBlock, ContextBlock, Aggregate removed
	// Valid: Domain with contexts and aggregates (domain modeling)
	// prog2 := &language.Program{
	// 	Architecture: &language.Architecture{
	// 		Domains: []*language.DomainBlock{
	// 			{
	// 				ID: "ECommerce",
	// 				Contexts: []*language.ContextBlock{
	// 					{
	// 						ID: "Orders",
	// 						Aggregates: []*language.Aggregate{
	// 							{ID: "Order"},
	// 						},
	// 					},
	// 				},
	// 			},
	// 		},
	// 	},
	// }
	// errors2 := rule.Validate(prog2)
	// ...

	// Valid: Both together (complementary) - DDD features removed
	// prog3 := &language.Program{
	// 	Architecture: &language.Architecture{
	// 		Systems: []*language.System{
	// 			{
	// 				ID: "ShopAPI",
	// 				Containers: []*language.Container{
	// 					{ID: "WebApp"},
	// 				},
	// 			},
	// 		},
	// 		Domains: []*language.DomainBlock{
	// 			{
	// 				ID: "ECommerce",
	// 				Contexts: []*language.ContextBlock{
	// 					{
	// 						ID: "Orders",
	// 						Aggregates: []*language.Aggregate{
	// 							{ID: "Order"},
	// 						},
	// 					},
	// 				},
	// 			},
	// 		},
	// 	},
	// }
	// errors3 := rule.Validate(prog3)
	// ...
}

// Additional tests for edge cases to improve coverage
func TestSimplicityRule_NilArchitecture(t *testing.T) {
	rule := &SimplicityRule{}

	prog := &language.Program{
		Model: nil,
	}

	errors := rule.Validate(prog)
	if len(errors) != 0 {
		t.Errorf("Expected no errors for nil model, got: %v", errors)
	}
}

func TestSimplicityRule_EmptySystems(t *testing.T) {
	rule := &SimplicityRule{}

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	dsl := ``

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	errors := rule.Validate(program)
	if len(errors) != 0 {
		t.Errorf("Expected no errors for empty model, got: %v", errors)
	}
}

func TestSimplicityRule_SystemWithMultipleContainerTypes(t *testing.T) {
	rule := &SimplicityRule{}

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	dsl := `
		ComplexSys = system "Complex System" {
			Web = container "Web"
			DB = datastore "Database"
		}
	`

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	errors := rule.Validate(program)
	if len(errors) != 0 {
		t.Errorf("Expected no errors, got: %v", errors)
	}
}

func TestSimplicityRule_Name(t *testing.T) {
	rule := &SimplicityRule{}
	name := rule.Name()

	if name != "SimplicityGuidance" {
		t.Errorf("Expected name 'SimplicityGuidance', got %q", name)
	}
}

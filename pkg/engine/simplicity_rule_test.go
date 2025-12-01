package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestSimplicityRule_DomainForDeployment(t *testing.T) {
	rule := &SimplicityRule{}

	// Domain used for simple deployment modeling (should suggest system)
	prog := &language.Program{
		Architecture: &language.Architecture{
			Domains: []*language.DomainBlock{
				{
					ID: "ECommerce",
					Components: []*language.Component{
						{ID: "WebApp"},
						{ID: "Database"},
					},
					// No contexts, aggregates, entities - just components
				},
			},
		},
	}

	errors := rule.Validate(prog)
	if len(errors) == 0 {
		t.Error("Expected warning about using domain for deployment modeling")
	}

	found := false
	for _, err := range errors {
		if containsString(err.Message, "Consider using 'system'") || containsString(err.Message, "consider using 'system'") {
			found = true
			break
		}
	}
	if !found {
		t.Errorf("Expected suggestion to use 'system', got: %v", errors)
	}
}

func TestSimplicityRule_SystemForDomain(t *testing.T) {
	rule := &SimplicityRule{}

	// System used for pure domain modeling (should suggest domain)
	prog := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{
				{
					ID: "ShopAPI",
					Contexts: []*language.ContextBlock{
						{
							ID: "Orders",
							Aggregates: []*language.Aggregate{
								{ID: "Order"},
							},
						},
					},
					// No containers, components, or datastores - just domain concepts
				},
			},
		},
	}

	errors := rule.Validate(prog)
	if len(errors) == 0 {
		t.Error("Expected warning about using system for domain modeling")
	}

	found := false
	for _, err := range errors {
		if containsString(err.Message, "Consider using 'domain'") || containsString(err.Message, "consider using 'domain'") {
			found = true
			break
		}
	}
	if !found {
		t.Errorf("Expected suggestion to use 'domain', got: %v", errors)
	}
}

func TestSimplicityRule_ValidUsage(t *testing.T) {
	rule := &SimplicityRule{}

	// Valid: System with containers (deployment modeling)
	prog1 := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{
				{
					ID: "ShopAPI",
					Containers: []*language.Container{
						{ID: "WebApp"},
						{ID: "Database"},
					},
				},
			},
		},
	}

	errors1 := rule.Validate(prog1)
	if len(errors1) > 0 {
		t.Errorf("Expected no errors for valid system usage, got: %v", errors1)
	}

	// Valid: Domain with contexts and aggregates (domain modeling)
	prog2 := &language.Program{
		Architecture: &language.Architecture{
			Domains: []*language.DomainBlock{
				{
					ID: "ECommerce",
					Contexts: []*language.ContextBlock{
						{
							ID: "Orders",
							Aggregates: []*language.Aggregate{
								{ID: "Order"},
							},
						},
					},
				},
			},
		},
	}

	errors2 := rule.Validate(prog2)
	if len(errors2) > 0 {
		t.Errorf("Expected no errors for valid domain usage, got: %v", errors2)
	}

	// Valid: Both together (complementary)
	prog3 := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{
				{
					ID: "ShopAPI",
					Containers: []*language.Container{
						{ID: "WebApp"},
					},
				},
			},
			Domains: []*language.DomainBlock{
				{
					ID: "ECommerce",
					Contexts: []*language.ContextBlock{
						{
							ID: "Orders",
							Aggregates: []*language.Aggregate{
								{ID: "Order"},
							},
						},
					},
				},
			},
		},
	}

	errors3 := rule.Validate(prog3)
	if len(errors3) > 0 {
		t.Errorf("Expected no errors when both system and domain coexist, got: %v", errors3)
	}
}

func containsString(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}


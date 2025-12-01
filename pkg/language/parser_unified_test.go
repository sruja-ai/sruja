package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

const (
	testSecurityID = "Security"
	testCartPage   = "Cart Page"
)

func TestParser_UnifiedDesign(t *testing.T) {
	dsl := `
// Governance (Global Policy)
policy Security "Must encrypt data" {
	rule Encryption "Data Encryption" {
		check "tags contains 'encrypted'"
	}
}

architecture "Unified Design" follows Security {
	system Shop {
		// Policy in System (still allowed)
		policy Audit "Keep logs for 1 year"

		// Enhanced ADR
		adr DB_Choice "Database" {
			status "Accepted"
			option Mongo {
				pro "Flexible schema"
			}
			decision Postgres
			reason "Relational data model fits best"
		}

		// User Story
		story Checkout "Checkout Flow" {
			User "adds item" {
				at "Cart Page"
			}
			User -> "Cart Page" "clicks checkout" {
				using "Mobile App"
			}
			"Cart Page" -> Shop "validates"
		}

		// DFD Flow
		flow OrderProcess "Order Processing" {
			Customer -> Shop "Order Details"
			Shop -> DB_Choice "Save Order"
		}
		

	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("NewParser failed: %v", err)
	}
	file, err := parser.Parse("unified.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	// PostProcess is already called by Parse
	arch := file.Architecture

	// Verify Global Policy (processed into Requirements)
	var globalPolicy *language.Requirement
	for _, req := range arch.Requirements {
		if req.ID == testSecurityID {
			globalPolicy = req
			break
		}
	}

	if globalPolicy == nil {
		t.Errorf("Expected global policy 'Security' not found in architecture requirements")
	} else if *globalPolicy.Type != "policy" {
		t.Errorf("Expected policy Type 'policy', got '%s'", *globalPolicy.Type)
	}

	// Verify Architecture Follows
	if len(arch.Follows) != 1 {
		t.Errorf("Expected architecture to follow 1 policy, got %d", len(arch.Follows))
	} else if arch.Follows[0] != "Security" {
		t.Errorf("Expected architecture to follow 'Security', got '%s'", arch.Follows[0])
	}

	// Verify System Items
	if len(arch.Systems) != 1 {
		t.Errorf("Expected 1 system, got %d", len(arch.Systems))
	}
	sys := arch.Systems[0]

	if len(sys.Requirements) != 1 {
		t.Errorf("Expected 1 system policy, got %d", len(sys.Requirements))
	} else {
		// System policy is added as requirement
		req := sys.Requirements[0]
		if req.ID != "Audit" {
			t.Errorf("Expected requirement ID 'Audit', got '%s'", req.ID)
		}
	}

	// Verify Global Policy Rules (accessed via System Requirements)
	// Since global policies are added to architecture requirements, and architecture requirements
	// are not directly in System struct unless inherited/merged.
	// In this test setup, the parser merges global policies into Architecture.Requirements.
	// But we are checking `sys` (System).
	// Let's check `arch.Requirements`.
	if len(arch.Requirements) != 1 {
		t.Errorf("Expected 1 global requirement, got %d", len(arch.Requirements))
	} else {
		req := arch.Requirements[0]
		if req.ID != "Security" {
			t.Errorf("Expected requirement ID 'Security', got '%s'", req.ID)
		}
		if len(req.Rules) != 1 {
			t.Errorf("Expected 1 rule in requirement, got %d", len(req.Rules))
		} else {
			rule := req.Rules[0]
			if rule.ID != "Encryption" {
				t.Errorf("Expected rule ID 'Encryption', got '%s'", rule.ID)
			}
			if rule.Check == nil || *rule.Check != "tags contains 'encrypted'" {
				t.Errorf("Expected check 'tags contains encrypted', got '%v'", rule.Check)
			}
		}
	}

	// Verify ADR Options
	if len(sys.ADRs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(sys.ADRs))
	} else {
		adr := sys.ADRs[0]
		if adr.Body == nil {
			t.Errorf("ADR body is nil")
		} else {
			if len(adr.Body.Options) != 1 {
				t.Errorf("Expected 1 ADR option, got %d", len(adr.Body.Options))
			}
			if adr.Body.Decision == nil || *adr.Body.Decision != "Postgres" {
				t.Errorf("Expected decision 'Postgres', got '%s'", *adr.Body.Decision)
			}
			if adr.Body.Reason == nil || *adr.Body.Reason != "Relational data model fits best" {
				t.Errorf("Expected reason 'Relational data model fits best', got '%v'", adr.Body.Reason)
			}
		}
	}

	// Verify Story (Scenario)
	if len(sys.Scenarios) != 1 {
		t.Errorf("Expected 1 scenario in system, got %d", len(sys.Scenarios))
	} else {
		story := sys.Scenarios[0]
		if story.ID != "Checkout" {
			t.Errorf("Expected story ID 'Checkout', got '%s'", story.ID)
		}
		if len(story.Steps) != 3 {
			t.Errorf("Expected 3 steps, got %d", len(story.Steps))
		}
		// Verify Qualified Step
		if story.Steps[0].From != "User" || story.Steps[0].To != nil {
			t.Errorf("Step 1 mismatch: %+v", story.Steps[0])
		}
		if len(story.Steps[0].Props) != 1 || story.Steps[0].Props[0].Key != "at" || story.Steps[0].Props[0].Value != testCartPage {
			t.Errorf("Step 1 props mismatch: %+v", story.Steps[0].Props)
		}

		// Verify String Literal Step
		if story.Steps[1].From != "User" || story.Steps[1].To == nil || *story.Steps[1].To != testCartPage {
			toVal := "nil"
			if story.Steps[1].To != nil {
				toVal = *story.Steps[1].To
			}
			t.Errorf("Step 2 mismatch: From='%s', To='%s'", story.Steps[1].From, toVal)
		}
		if len(story.Steps[1].Props) != 1 || story.Steps[1].Props[0].Key != "using" || story.Steps[1].Props[0].Value != "Mobile App" {
			t.Errorf("Step 2 props mismatch: %+v", story.Steps[1].Props)
		}

		// Verify Mixed Step
		if story.Steps[2].From != testCartPage || story.Steps[2].To == nil || *story.Steps[2].To != "Shop" {
			toVal := "nil"
			if story.Steps[2].To != nil {
				toVal = *story.Steps[2].To
			}
			t.Errorf("Step 3 mismatch: From='%s', To='%s'", story.Steps[2].From, toVal)
		}
	}

	// Verify Flow
	if len(sys.Flows) != 1 {
		t.Errorf("Expected 1 flow, got %d", len(sys.Flows))
	} else {
		flow := sys.Flows[0]
		if flow.ID != "OrderProcess" {
			t.Errorf("Expected flow ID 'OrderProcess', got '%s'", flow.ID)
		}
		if len(flow.Steps) != 2 {
			t.Errorf("Expected 2 flow steps, got %d", len(flow.Steps))
		}
		if flow.Steps[0].From != "Customer" || flow.Steps[0].To != "Shop" {
			t.Errorf("Flow Step 1 mismatch: %+v", flow.Steps[0])
		}
	}
}

func TestParser_EcommerceDetailed(t *testing.T) {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("NewParser failed: %v", err)
	}
	// Assuming the example file exists relative to the test execution path
	// We will read the file content using absolute path for this verification step.
	file, err := parser.ParseFile("../../examples/ecommerce_detailed.sruja")
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}

	arch := file.Architecture
	if arch.Name != "Global E-Commerce Platform" {
		t.Errorf("Expected architecture name 'Global E-Commerce Platform', got '%s'", arch.Name)
	}

	if len(arch.Systems) != 3 {
		t.Fatalf("Expected 3 systems, got %d", len(arch.Systems))
	}

	var sys *language.System
	for _, s := range arch.Systems {
		if s.ID == "ECommerce" {
			sys = s
			break
		}
	}
	if sys == nil {
		t.Fatalf("ECommerce system not found")
	}

	// Verify Containers
	if len(sys.Containers) != 10 {
		t.Errorf("Expected 10 containers, got %d", len(sys.Containers))
	}

	// Verify Components in WebApp
	var webApp *language.Container
	for _, c := range sys.Containers {
		if c.ID == "WebApp" {
			webApp = c
			break
		}
	}
	if webApp == nil {
		t.Fatalf("WebApp container not found")
	}
	if len(webApp.Components) != 5 {
		t.Errorf("Expected 5 components in WebApp, got %d", len(webApp.Components))
	}

	// Verify Stories
	if len(sys.Scenarios) != 5 {
		t.Errorf("Expected 5 stories, got %d", len(sys.Scenarios))
	}

	// Verify Flows
	if len(sys.Flows) != 2 {
		t.Errorf("Expected 2 flows, got %d", len(sys.Flows))
	}

	// Verify Views
	if len(arch.Views) != 3 {
		t.Errorf("Expected 3 views, got %d", len(arch.Views))
	}

	// Check ContextView
	var contextView *language.View
	for _, v := range arch.Views {
		if v.ID == "ContextView" {
			contextView = v
			break
		}
	}
	if contextView == nil {
		t.Fatalf("ContextView not found")
	}
	if contextView.Body.Scope == nil || !contextView.Body.Scope.Global {
		t.Errorf("ContextView should have global scope")
	}
}

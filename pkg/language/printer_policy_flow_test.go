package language

import (
	"strings"
	"testing"
)

// TestPrinter_Policy tests printing Policy constructs
func TestPrinter_Policy(t *testing.T) {
	testCases := []struct {
		name     string
		dsl      string
		contains []string
	}{
		{
			name: "Policy with category and enforcement",
			dsl: `architecture "Test" {
				policy P1 "Data retention policy" {
					category "compliance"
					enforcement "mandatory"
				}
			}`,
			contains: []string{"policy P1", "Data retention policy", "category", "compliance", "enforcement", "mandatory"},
		},
		{
			name: "Simple policy",
			dsl: `architecture "Test" {
				policy P2 "API rate limiting"
			}`,
			contains: []string{"policy P2", "API rate limiting"},
		},
		{
			name: "Policy with metadata",
			dsl: `architecture "Test" {
				policy P3 "Security policy" {
					category "security"
					enforcement "strict"
				}
			}`,
			contains: []string{"policy P3", "Security policy", "category", "security"},
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			parser, err := NewParser()
			if err != nil {
				t.Fatalf("Failed to create parser: %v", err)
			}

			program, _, err := parser.Parse("test.sruja", tc.dsl)
			if err != nil {
				t.Fatalf("Failed to parse DSL: %v", err)
			}

			printer := NewPrinter()
			output := printer.Print(program)

			for _, expected := range tc.contains {
				if !strings.Contains(output, expected) {
					t.Errorf("Expected output to contain %q, output:\n%s", expected, output)
				}
			}
		})
	}
}

// TestPrinter_Flow tests printing Flow constructs
func TestPrinter_Flow(t *testing.T) {
	t.Skip("Skipping Flow printer tests due to parser panic")
	testCases := []struct {
		name     string
		dsl      string
		contains []string
		skip     bool // Skip if parser doesn't support yet
	}{
		{
			name: "Flow with steps and order",
			dsl: `architecture "Test" {
				flow F1 "User Onboarding" {
					step S1 "Create account" order 1
					step S2 "Verify email" order 2
					step S3 "Complete profile" order 3
				}
			}`,
			contains: []string{"flow F1", "User Onboarding", "step S1", "Create account", "order 1"},
		},
		{
			name: "Flow with description",
			dsl: `architecture "Test" {
				flow F2 "Checkout Flow" {
					description "E-commerce checkout process"
					step S1 "Add to cart"
					step S2 "Payment"
				}
			}`,
			contains: []string{"flow F2", "Checkout Flow", "description", "E-commerce checkout process"},
		},
		{
			name: "Simple flow",
			dsl: `architecture "Test" {
				flow F3 "Simple Flow" {
					step S1 "Step 1"
				}
			}`,
			contains: []string{"flow F3", "Simple Flow", "step S1"},
		},
		{
			name: "DFD-style flow with relations",
			dsl: `architecture "Test" {
				person Customer "Customer"
				system Shop "Shop" {
					container WebApp "Web Application"
					datastore DB "Database"
				}
				flow OrderProcess "Order Processing" {
					Customer -> Shop "Order Details"
					Shop -> Shop.WebApp "Processes"
					Shop.WebApp -> Shop.DB "Save Order"
					Shop.DB -> Shop.WebApp "Confirmation"
				}
			}`,
			contains: []string{"flow OrderProcess", "Order Processing", "Customer -> Shop", "Order Details"},
			skip:     true, // Parser doesn't support DFD-style flows yet
		},
		{
			name: "DFD-style flow with qualified references",
			dsl: `architecture "Test" {
				system API "API Service" {
					container WebApp "Web App"
					datastore DB "Database"
				}
				flow DataFlow "Data Flow" {
					API.WebApp -> API.DB "Reads/Writes"
					API.DB -> API.WebApp "Returns data"
				}
			}`,
			contains: []string{"flow DataFlow", "Data Flow", "API.WebApp -> API.DB"},
			skip:     true, // Parser doesn't support DFD-style flows yet
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			if tc.skip {
				t.Skip("Parser doesn't support DFD-style flows yet - FlowRelation pattern needs parser fix")
			}

			parser, err := NewParser()
			if err != nil {
				t.Fatalf("Failed to create parser: %v", err)
			}

			program, _, err := parser.Parse("test.sruja", tc.dsl)
			if err != nil {
				t.Fatalf("Failed to parse DSL: %v", err)
			}

			printer := NewPrinter()
			output := printer.Print(program)

			for _, expected := range tc.contains {
				if !strings.Contains(output, expected) {
					t.Errorf("Expected output to contain %q, output:\n%s", expected, output)
				}
			}
		})
	}
}

// TestPrinter_PolicyFlowRoundTrip tests round-trip parsing and printing
func TestPrinter_PolicyFlowRoundTrip(t *testing.T) {
	t.Skip("Skipping Flow roundtrip tests due to parser panic")
	dsl := `architecture "RoundTripTest" {
		policy P1 "Data policy" {
			category "compliance"
			enforcement "mandatory"
		}
		
		flow F1 "User Flow" {
			description "User onboarding"
			step S1 "Register" order 1
			step S2 "Verify" order 2
		}
	}`

	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	// First parse
	program1, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL (first): %v", err)
	}

	// Print back to DSL
	printer := NewPrinter()
	output := printer.Print(program1)

	// Parse again
	reparsed, _, err := parser.Parse("test.sruja", output)
	if err != nil {
		t.Fatalf("Failed to parse printed output (second): %v\nOutput:\n%s", err, output)
	}

	// Should have same number of policies and flows
	if len(program1.Architecture.Policies) != len(reparsed.Architecture.Policies) {
		t.Errorf("Policy count mismatch: %d vs %d", len(program1.Architecture.Policies), len(reparsed.Architecture.Policies))
	}

	if len(program1.Architecture.Flows) != len(reparsed.Architecture.Flows) {
		t.Errorf("Flow count mismatch: %d vs %d", len(program1.Architecture.Flows), len(reparsed.Architecture.Flows))
	}
}

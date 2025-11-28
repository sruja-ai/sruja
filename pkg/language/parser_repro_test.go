package language

import (
	"testing"
)

func TestParser_Lesson1Snippet(t *testing.T) {
	dsl := `
architecture "Payment System" {
    // Define an ADR
    adr ADR001 "Use Stripe for Payments" {
        status "Accepted"
        context "We need a reliable payment processor that supports global currencies."
        decision "We will use Stripe as our primary payment gateway."
        consequences "Vendor lock-in, but faster time to market."
    }

    system PaymentService "Payment Service" {
        // Link the ADR to the component it affects
        adr ADR001 "Dummy Title"
        description "Handles credit card processing."
    }
}
`
	parser, err := NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("lesson1.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	if len(program.Architecture.ADRs) != 1 {
		t.Fatalf("Expected 1 ADR, got %d", len(program.Architecture.ADRs))
	}

	adr := program.Architecture.ADRs[0]
	if adr.Body == nil {
		t.Fatal("Expected ADR body, got nil")
	}

	if *adr.Body.Status != "Accepted" {
		t.Errorf("Expected status 'Accepted', got %q", *adr.Body.Status)
	}
}

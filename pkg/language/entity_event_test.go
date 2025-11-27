//go:build newdsl

package language

import "testing"

func TestParseEntitiesAndEvents(t *testing.T) {
	src := `architecture Test {
      domain Payments {
        entities {
          entity Payment {
            fields { id: String amount: Float }
            lifecycle { PENDING -> AUTHORIZED AUTHORIZED -> COMPLETED }
          }
        }
        events {
          event PaymentAuthorized {
            version "1.0"
            entity Payment
            schema { paymentId: String amount: Float }
            lifecycle_effect { Payment.PENDING -> Payment.AUTHORIZED }
          }
        }
      }
    }`
	p, err := NewParser()
	if err != nil {
		t.Fatalf("parser init: %v", err)
	}
	prog, err := p.Parse("test.sruja", src)
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}
	arch := prog.Architecture
	if arch == nil {
		t.Fatalf("no architecture parsed")
	}
	if len(arch.Entities) != 1 {
		t.Fatalf("expected 1 entity, got %d", len(arch.Entities))
	}
	if len(arch.Events) != 1 {
		t.Fatalf("expected 1 event, got %d", len(arch.Events))
	}
	ev := arch.Events[0]
	if ev.Body == nil || ev.Body.LifecycleEffect == nil {
		t.Fatalf("event missing lifecycle effect")
	}
	if ev.Body.LifecycleEffect.From.Entity != "Payment" {
		t.Fatalf("unexpected lifecycle entity")
	}
}

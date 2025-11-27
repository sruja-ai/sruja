// pkg/kernel/simulation_test.go
// Tests for event simulation engine

package kernel

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestBuildFSMFromEntity(t *testing.T) {
	engine := NewSimulationEngine()

	entity := &language.Entity{
		Name: "Payment",
		Body: &language.EntityBody{
			Lifecycle: &language.LifecycleBlock{
				Transitions: []*language.LifecycleTransition{
					{From: "PENDING", To: "AUTHORIZED"},
					{From: "AUTHORIZED", To: "COMPLETED"},
					{From: "AUTHORIZED", To: "FAILED"},
				},
			},
		},
	}

	fsm, err := engine.BuildFSMFromEntity(entity)
	if err != nil {
		t.Fatalf("Failed to build FSM: %v", err)
	}

	if fsm.EntityName != "Payment" {
		t.Errorf("Expected entity name 'Payment', got '%s'", fsm.EntityName)
	}

	if len(fsm.States) != 4 {
		t.Errorf("Expected 4 states, got %d", len(fsm.States))
	}

	// Check transitions
	transitions, ok := fsm.Transitions["PENDING"]
	if !ok || len(transitions) != 1 || transitions[0] != "AUTHORIZED" {
		t.Errorf("Invalid transitions from PENDING: %v", transitions)
	}

	transitions, ok = fsm.Transitions["AUTHORIZED"]
	if !ok || len(transitions) != 2 {
		t.Errorf("Invalid transitions from AUTHORIZED: %v", transitions)
	}
}

func TestRegisterEventEffect(t *testing.T) {
	engine := NewSimulationEngine()

	event := &language.DomainEvent{
		Name: "PaymentAuthorized",
		Body: &language.EventBody{
			LifecycleEffect: &language.EventLifecycleEffect{
				From: &language.QualifiedState{
					Entity: "Payment",
					State:  "PENDING",
				},
				To: &language.QualifiedState{
					Entity: "Payment",
					State:  "AUTHORIZED",
				},
			},
		},
	}

	engine.RegisterEventEffect(event)

	effect, ok := engine.eventEffects["PaymentAuthorized"]
	if !ok {
		t.Fatal("Event effect not registered")
	}

	if effect.EntityName != "Payment" {
		t.Errorf("Expected entity 'Payment', got '%s'", effect.EntityName)
	}

	if effect.FromState != "PENDING" {
		t.Errorf("Expected from state 'PENDING', got '%s'", effect.FromState)
	}

	if effect.ToState != "AUTHORIZED" {
		t.Errorf("Expected to state 'AUTHORIZED', got '%s'", effect.ToState)
	}
}

func TestSimulate(t *testing.T) {
	engine := NewSimulationEngine()

	// Build FSM
	entity := &language.Entity{
		Name: "Payment",
		Body: &language.EntityBody{
			Lifecycle: &language.LifecycleBlock{
				Transitions: []*language.LifecycleTransition{
					{From: "PENDING", To: "AUTHORIZED"},
					{From: "AUTHORIZED", To: "COMPLETED"},
				},
			},
		},
	}

	_, err := engine.BuildFSMFromEntity(entity)
	if err != nil {
		t.Fatalf("Failed to build FSM: %v", err)
	}

	// Register event effects
	event1 := &language.DomainEvent{
		Name: "PaymentAuthorized",
		Body: &language.EventBody{
			LifecycleEffect: &language.EventLifecycleEffect{
				From: &language.QualifiedState{
					Entity: "Payment",
					State:  "PENDING",
				},
				To: &language.QualifiedState{
					Entity: "Payment",
					State:  "AUTHORIZED",
				},
			},
		},
	}

	event2 := &language.DomainEvent{
		Name: "PaymentCompleted",
		Body: &language.EventBody{
			LifecycleEffect: &language.EventLifecycleEffect{
				From: &language.QualifiedState{
					Entity: "Payment",
					State:  "AUTHORIZED",
				},
				To: &language.QualifiedState{
					Entity: "Payment",
					State:  "COMPLETED",
				},
			},
		},
	}

	engine.RegisterEventEffect(event1)
	engine.RegisterEventEffect(event2)

	// Simulate valid sequence
	sim, err := engine.Simulate("Payment", "PENDING", []string{"PaymentAuthorized", "PaymentCompleted"})
	if err != nil {
		t.Fatalf("Simulation failed: %v", err)
	}

	if sim.FinalState != "COMPLETED" {
		t.Errorf("Expected final state 'COMPLETED', got '%s'", sim.FinalState)
	}

	if len(sim.InvalidTransitions) > 0 {
		t.Errorf("Expected no invalid transitions, got %d", len(sim.InvalidTransitions))
	}

	if len(sim.StateHistory) != 3 {
		t.Errorf("Expected 3 state snapshots, got %d", len(sim.StateHistory))
	}
}

func TestSimulateInvalidTransition(t *testing.T) {
	engine := NewSimulationEngine()

	// Build FSM
	entity := &language.Entity{
		Name: "Payment",
		Body: &language.EntityBody{
			Lifecycle: &language.LifecycleBlock{
				Transitions: []*language.LifecycleTransition{
					{From: "PENDING", To: "AUTHORIZED"},
					{From: "AUTHORIZED", To: "COMPLETED"},
				},
			},
		},
	}

	_, err := engine.BuildFSMFromEntity(entity)
	if err != nil {
		t.Fatalf("Failed to build FSM: %v", err)
	}

	// Register event that tries invalid transition
	event := &language.DomainEvent{
		Name: "InvalidEvent",
		Body: &language.EventBody{
			LifecycleEffect: &language.EventLifecycleEffect{
				From: &language.QualifiedState{
					Entity: "Payment",
					State:  "PENDING",
				},
				To: &language.QualifiedState{
					Entity: "Payment",
					State:  "COMPLETED", // Invalid: can't go directly from PENDING to COMPLETED
				},
			},
		},
	}

	engine.RegisterEventEffect(event)

	// Simulate invalid sequence
	sim, err := engine.Simulate("Payment", "PENDING", []string{"InvalidEvent"})
	if err != nil {
		t.Fatalf("Simulation failed: %v", err)
	}

	if len(sim.InvalidTransitions) == 0 {
		t.Error("Expected invalid transition, but none were detected")
	}
}

func TestParseSimulationCommand(t *testing.T) {
	tests := []struct {
		name    string
		command string
		want    *SimulationCommand
		wantErr bool
	}{
		{
			name:    "basic simulation",
			command: "simulate Payment from PENDING events: PaymentAuthorized, PaymentCompleted",
			want: &SimulationCommand{
				EntityName:   "Payment",
				InitialState: "PENDING",
				Events:       []string{"PaymentAuthorized", "PaymentCompleted"},
			},
			wantErr: false,
		},
		{
			name:    "simulation without initial state",
			command: "simulate Payment events: PaymentAuthorized",
			want: &SimulationCommand{
				EntityName:   "Payment",
				InitialState: "",
				Events:       []string{"PaymentAuthorized"},
			},
			wantErr: false,
		},
		{
			name:    "simulation without events",
			command: "simulate Payment from PENDING",
			want: &SimulationCommand{
				EntityName:   "Payment",
				InitialState: "PENDING",
				Events:       []string{},
			},
			wantErr: false,
		},
		{
			name:    "invalid command",
			command: "invalid command",
			want:    nil,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ParseSimulationCommand(tt.command)
			if (err != nil) != tt.wantErr {
				t.Errorf("ParseSimulationCommand() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if !tt.wantErr {
				if got.EntityName != tt.want.EntityName {
					t.Errorf("EntityName = %v, want %v", got.EntityName, tt.want.EntityName)
				}
				if got.InitialState != tt.want.InitialState {
					t.Errorf("InitialState = %v, want %v", got.InitialState, tt.want.InitialState)
				}
				if len(got.Events) != len(tt.want.Events) {
					t.Errorf("Events length = %v, want %v", len(got.Events), len(tt.want.Events))
				}
			}
		})
	}
}

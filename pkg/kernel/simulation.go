// pkg/kernel/simulation.go
// Event simulation engine for lifecycle transitions

package kernel

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// LifecycleFSM represents a finite state machine for entity lifecycle.
type LifecycleFSM struct {
	EntityName   string
	States       []string
	Transitions  map[string][]string // from state -> []to states
	InitialState string
}

// EventSimulation represents a simulation of event sequences.
type EventSimulation struct {
	EntityName         string
	InitialState       string
	Events             []string
	StateHistory       []StateSnapshot
	FinalState         string
	InvalidTransitions []InvalidTransition
	Warnings           []string
}

// StateSnapshot represents the state at a point in the simulation.
type StateSnapshot struct {
	State     string
	Event     string
	Timestamp int
}

// InvalidTransition represents an invalid state transition attempt.
type InvalidTransition struct {
	FromState string
	ToState   string
	Event     string
	Reason    string
}

// SimulationEngine handles event-driven lifecycle simulation.
type SimulationEngine struct {
	fsms         map[string]*LifecycleFSM         // entity name -> FSM
	eventEffects map[string]*EventLifecycleEffect // event name -> lifecycle effect
}

// EventLifecycleEffect represents how an event affects entity lifecycle.
type EventLifecycleEffect struct {
	EntityName string
	FromState  string
	ToState    string
	EventName  string
}

// NewSimulationEngine creates a new simulation engine.
func NewSimulationEngine() *SimulationEngine {
	return &SimulationEngine{
		fsms:         make(map[string]*LifecycleFSM),
		eventEffects: make(map[string]*EventLifecycleEffect),
	}
}

// BuildFSMFromEntity extracts lifecycle FSM from an entity definition.
func (se *SimulationEngine) BuildFSMFromEntity(entity *language.Entity) (*LifecycleFSM, error) {
	if entity.Body == nil || entity.Body.Lifecycle == nil {
		return nil, fmt.Errorf("entity %s has no lifecycle definition", entity.Name)
	}

	fsm := &LifecycleFSM{
		EntityName:   entity.Name,
		States:       []string{},
		Transitions:  make(map[string][]string),
		InitialState: "",
	}

	// Extract states from transitions
	stateSet := make(map[string]bool)
	for _, transition := range entity.Body.Lifecycle.Transitions {
		stateSet[transition.From] = true
		stateSet[transition.To] = true
	}

	// Convert set to slice
	for state := range stateSet {
		fsm.States = append(fsm.States, state)
	}

	// Build transition map
	for _, transition := range entity.Body.Lifecycle.Transitions {
		if fsm.Transitions[transition.From] == nil {
			fsm.Transitions[transition.From] = []string{}
		}
		fsm.Transitions[transition.From] = append(fsm.Transitions[transition.From], transition.To)
	}

	// Determine initial state (first state alphabetically, or first in transitions)
	if len(fsm.States) > 0 {
		// Try to find a state with no incoming transitions (initial state)
		hasIncoming := make(map[string]bool)
		for _, toStates := range fsm.Transitions {
			for _, to := range toStates {
				hasIncoming[to] = true
			}
		}

		for _, state := range fsm.States {
			if !hasIncoming[state] {
				fsm.InitialState = state
				break
			}
		}

		// If no initial state found, use first state
		if fsm.InitialState == "" {
			fsm.InitialState = fsm.States[0]
		}
	}

	se.fsms[entity.Name] = fsm
	return fsm, nil
}

// RegisterEventEffect registers an event's lifecycle effect.
func (se *SimulationEngine) RegisterEventEffect(event *language.DomainEvent) {
	if event.Body == nil || event.Body.LifecycleEffect == nil {
		return
	}

	effect := event.Body.LifecycleEffect
	if effect.From != nil && effect.To != nil {
		se.eventEffects[event.Name] = &EventLifecycleEffect{
			EntityName: effect.From.Entity,
			FromState:  effect.From.State,
			ToState:    effect.To.State,
			EventName:  event.Name,
		}
	}
}

// Simulate simulates a sequence of events for an entity.
func (se *SimulationEngine) Simulate(entityName, initialState string, eventNames []string) (*EventSimulation, error) {
	fsm, ok := se.fsms[entityName]
	if !ok {
		return nil, fmt.Errorf("no FSM found for entity: %s", entityName)
	}

	sim := &EventSimulation{
		EntityName:         entityName,
		InitialState:       initialState,
		Events:             eventNames,
		StateHistory:       []StateSnapshot{},
		InvalidTransitions: []InvalidTransition{},
		Warnings:           []string{},
	}

	// Use provided initial state or FSM default
	currentState := initialState
	if currentState == "" {
		currentState = fsm.InitialState
	}
	sim.InitialState = currentState

	// Add initial state snapshot
	sim.StateHistory = append(sim.StateHistory, StateSnapshot{
		State:     currentState,
		Event:     "",
		Timestamp: 0,
	})

	// Simulate each event
	for i, eventName := range eventNames {
		effect, hasEffect := se.eventEffects[eventName]

		if !hasEffect {
			sim.Warnings = append(sim.Warnings, fmt.Sprintf("Event '%s' has no lifecycle effect defined", eventName))
			continue
		}

		// Check if event applies to this entity
		if effect.EntityName != entityName {
			sim.Warnings = append(sim.Warnings, fmt.Sprintf("Event '%s' does not affect entity '%s' (affects '%s')", eventName, entityName, effect.EntityName))
			continue
		}

		// Check if current state matches expected from state
		if effect.FromState != currentState {
			sim.InvalidTransitions = append(sim.InvalidTransitions, InvalidTransition{
				FromState: currentState,
				ToState:   effect.ToState,
				Event:     eventName,
				Reason:    fmt.Sprintf("Event '%s' expects state '%s' but entity is in state '%s'", eventName, effect.FromState, currentState),
			})
			continue
		}

		// Check if transition is valid in FSM
		validTransitions, ok := fsm.Transitions[currentState]
		if !ok {
			sim.InvalidTransitions = append(sim.InvalidTransitions, InvalidTransition{
				FromState: currentState,
				ToState:   effect.ToState,
				Event:     eventName,
				Reason:    fmt.Sprintf("No transitions defined from state '%s'", currentState),
			})
			continue
		}

		// Check if target state is in valid transitions
		transitionValid := false
		for _, validTo := range validTransitions {
			if validTo == effect.ToState {
				transitionValid = true
				break
			}
		}

		if !transitionValid {
			sim.InvalidTransitions = append(sim.InvalidTransitions, InvalidTransition{
				FromState: currentState,
				ToState:   effect.ToState,
				Event:     eventName,
				Reason:    fmt.Sprintf("Invalid transition from '%s' to '%s' (valid transitions: %v)", currentState, effect.ToState, validTransitions),
			})
			continue
		}

		// Transition is valid - apply it
		currentState = effect.ToState
		sim.StateHistory = append(sim.StateHistory, StateSnapshot{
			State:     currentState,
			Event:     eventName,
			Timestamp: i + 1,
		})
	}

	sim.FinalState = currentState
	return sim, nil
}

// GetFSM returns the FSM for an entity.
func (se *SimulationEngine) GetFSM(entityName string) (*LifecycleFSM, bool) {
	fsm, ok := se.fsms[entityName]
	return fsm, ok
}

// FormatSimulationResult formats simulation results as human-readable text.
func FormatSimulationResult(sim *EventSimulation) string {
	var b strings.Builder

	b.WriteString(fmt.Sprintf("Simulation Results for Entity: %s\n", sim.EntityName))
	b.WriteString(fmt.Sprintf("Initial State: %s\n", sim.InitialState))
	b.WriteString(fmt.Sprintf("Final State: %s\n\n", sim.FinalState))

	if len(sim.Events) > 0 {
		b.WriteString("Event Sequence:\n")
		for i, event := range sim.Events {
			b.WriteString(fmt.Sprintf("  %d. %s\n", i+1, event))
		}
		b.WriteString("\n")
	}

	if len(sim.StateHistory) > 0 {
		b.WriteString("State History:\n")
		for _, snapshot := range sim.StateHistory {
			if snapshot.Event == "" {
				b.WriteString(fmt.Sprintf("  [%d] State: %s (initial)\n", snapshot.Timestamp, snapshot.State))
			} else {
				b.WriteString(fmt.Sprintf("  [%d] Event: %s → State: %s\n", snapshot.Timestamp, snapshot.Event, snapshot.State))
			}
		}
		b.WriteString("\n")
	}

	if len(sim.InvalidTransitions) > 0 {
		b.WriteString("⚠️  Invalid Transitions:\n")
		for _, invalid := range sim.InvalidTransitions {
			b.WriteString(fmt.Sprintf("  - %s\n", invalid.Reason))
		}
		b.WriteString("\n")
	}

	if len(sim.Warnings) > 0 {
		b.WriteString("⚠️  Warnings:\n")
		for _, warning := range sim.Warnings {
			b.WriteString(fmt.Sprintf("  - %s\n", warning))
		}
		b.WriteString("\n")
	}

	if len(sim.InvalidTransitions) == 0 && len(sim.Warnings) == 0 {
		b.WriteString("✅ Simulation completed successfully - all transitions valid!\n")
	}

	return b.String()
}

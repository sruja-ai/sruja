package engine

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

type OrphanDetectionRule struct{}

func (r *OrphanDetectionRule) Name() string {
	return "OrphanDetection"
}

func (r *OrphanDetectionRule) Validate(program *language.Program) []ValidationError {
	errors := []ValidationError{}
	defined := map[string]language.SourceLocation{}
	used := map[string]bool{}

	arch := program.Architecture
	if arch == nil {
		return errors
	}

	parent := map[string]string{}

	for _, sys := range arch.Systems {
		defined[sys.ID] = sys.Location()
		if len(sys.Requirements) > 0 || len(sys.ADRs) > 0 || len(sys.Contracts) > 0 {
			used[sys.ID] = true
		}
		for _, cont := range sys.Containers {
			defined[cont.ID] = cont.Location()
			parent[cont.ID] = sys.ID
			if len(cont.Requirements) > 0 || len(cont.ADRs) > 0 || len(cont.Contracts) > 0 {
				used[cont.ID] = true
			}
			for _, comp := range cont.Components {
				defined[comp.ID] = comp.Location()
				parent[comp.ID] = cont.ID
				if len(comp.Requirements) > 0 || len(comp.ADRs) > 0 {
					used[comp.ID] = true
				}
			}
			for _, ds := range cont.DataStores {
				defined[ds.ID] = ds.Location()
				parent[ds.ID] = cont.ID
			}
			for _, q := range cont.Queues {
				defined[q.ID] = q.Location()
				parent[q.ID] = cont.ID
			}
		}
		for _, comp := range sys.Components {
			defined[comp.ID] = comp.Location()
			parent[comp.ID] = sys.ID
			if len(comp.Requirements) > 0 || len(comp.ADRs) > 0 {
				used[comp.ID] = true
			}
		}
		for _, ds := range sys.DataStores {
			defined[ds.ID] = ds.Location()
			parent[ds.ID] = sys.ID
		}
		for _, q := range sys.Queues {
			defined[q.ID] = q.Location()
			parent[q.ID] = sys.ID
		}
	}
	for _, p := range arch.Persons {
		defined[p.ID] = p.Location()
	}

	markRel := func(from, to string) { used[from] = true; used[to] = true }
	for _, r := range arch.Relations {
		markRel(r.From, r.To)
	}
	for _, s := range arch.Systems {
		for _, r := range s.Relations {
			markRel(r.From, r.To)
		}
		for _, c := range s.Containers {
			for _, r := range c.Relations {
				markRel(r.From, r.To)
			}
		}
		for _, comp := range s.Components {
			for _, r := range comp.Relations {
				markRel(r.From, r.To)
			}
		}
	}

	// Helper to check steps
	checkSteps := func(steps []*language.ScenarioStep) {
		for _, step := range steps {
			if step.To != nil {
				markRel(step.From, *step.To)
			} else {
				// Mark actor as used
				used[step.From] = true
			}
		}
	}

	// Helper to check flow steps
	checkFlowSteps := func(steps []*language.FlowStep) {
		for _, step := range steps {
			markRel(step.From, step.To)
		}
	}

	// Check Scenarios (Top-level)
	for _, s := range arch.Scenarios {
		checkSteps(s.Steps)
	}

	// Check Flows (Top-level)
	for _, f := range arch.Flows {
		checkFlowSteps(f.Steps)
	}

	// Check Nested Scenarios and Flows
	for _, sys := range arch.Systems {
		for _, s := range sys.Scenarios {
			checkSteps(s.Steps)
		}
		for _, f := range sys.Flows {
			checkFlowSteps(f.Steps)
		}
	}

	// Propagate usage to parents
	// Iterate multiple times or just ensure we cover the depth (max 3: Component -> Container -> System)
	// Simple loop over defined elements to propagate
	for i := 0; i < 3; i++ {
		for id := range defined {
			if used[id] {
				if pID, ok := parent[id]; ok {
					used[pID] = true
				}
			}
		}
	}

	for id, loc := range defined {
		if !used[id] {
			errors = append(errors, ValidationError{Message: fmt.Sprintf("Orphan element '%s' is defined but never used in any relation.", id), Line: loc.Line, Column: loc.Column})
		}
	}
	return errors
}

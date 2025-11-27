package engine

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

type ValidReferenceRule struct{}

func (r *ValidReferenceRule) Name() string {
	return "Valid References"
}

func (r *ValidReferenceRule) Validate(program *language.Program) []ValidationError {
	errors := []ValidationError{}
	defined := map[string]bool{}

	arch := program.Architecture
	if arch == nil {
		return errors
	}

	for _, sys := range arch.Systems {
		defined[sys.ID] = true
		for _, cont := range sys.Containers {
			defined[cont.ID] = true
			for _, comp := range cont.Components {
				defined[comp.ID] = true
			}
			for _, ds := range cont.DataStores {
				defined[ds.ID] = true
			}
			for _, q := range cont.Queues {
				defined[q.ID] = true
			}
		}
		for _, comp := range sys.Components {
			defined[comp.ID] = true
		}
		for _, ds := range sys.DataStores {
			defined[ds.ID] = true
		}
		for _, q := range sys.Queues {
			defined[q.ID] = true
		}
	}
	for _, p := range arch.Persons {
		defined[p.ID] = true
	}

	checkRel := func(rel *language.Relation) {
		if rel == nil {
			return
		}
		if !defined[rel.From] {
			loc := rel.Location()
			errors = append(errors, ValidationError{Message: fmt.Sprintf("Reference to undefined element '%s'", rel.From), Line: loc.Line, Column: loc.Column})
		}
		if !defined[rel.To] {
			loc := rel.Location()
			errors = append(errors, ValidationError{Message: fmt.Sprintf("Reference to undefined element '%s'", rel.To), Line: loc.Line, Column: loc.Column})
		}
	}
	for _, r := range arch.Relations {
		checkRel(r)
	}
	for _, s := range arch.Systems {
		for _, r := range s.Relations {
			checkRel(r)
		}
		for _, c := range s.Containers {
			for _, r := range c.Relations {
				checkRel(r)
			}
		}
		for _, comp := range s.Components {
			for _, r := range comp.Relations {
				checkRel(r)
			}
		}
	}

	// Current Requirement struct has no Implements field; skip implements checks
	return errors
}

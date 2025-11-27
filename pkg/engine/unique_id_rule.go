package engine

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

type UniqueIDRule struct{}

func (r *UniqueIDRule) Name() string {
	return "Unique IDs"
}

func (r *UniqueIDRule) Validate(program *language.Program) []ValidationError {
	errors := []ValidationError{}
	seenIDs := make(map[string]language.SourceLocation)

	checkID := func(id string, loc language.SourceLocation) {
		if id == "" {
			return
		}
		if existing, ok := seenIDs[id]; ok {
			errors = append(errors, ValidationError{
				Message: fmt.Sprintf("Duplicate identifier '%s'. Previously defined at line %d.", id, existing.Line),
				Line:    loc.Line,
				Column:  loc.Column,
			})
		} else {
			seenIDs[id] = loc
		}
	}

	arch := program.Architecture
	if arch == nil {
		return errors
	}

	for _, sys := range arch.Systems {
		checkID(sys.ID, sys.Location())
		for _, cont := range sys.Containers {
			checkID(cont.ID, cont.Location())
			for _, comp := range cont.Components {
				checkID(comp.ID, comp.Location())
			}
			for _, ds := range cont.DataStores {
				checkID(ds.ID, ds.Location())
			}
			for _, q := range cont.Queues {
				checkID(q.ID, q.Location())
			}
		}
		for _, comp := range sys.Components {
			checkID(comp.ID, comp.Location())
		}
		for _, ds := range sys.DataStores {
			checkID(ds.ID, ds.Location())
		}
		for _, q := range sys.Queues {
			checkID(q.ID, q.Location())
		}
	}

	for _, p := range arch.Persons {
		checkID(p.ID, p.Location())
	}
	for _, req := range arch.Requirements {
		checkID(req.ID, req.Location())
	}
	for _, adr := range arch.ADRs {
		checkID(adr.ID, adr.Location())
	}

	return errors
}

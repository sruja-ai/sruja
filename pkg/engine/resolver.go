package engine

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Resolver updates the AST to use fully qualified names for ambiguous-free references.
type Resolver struct {
	defined   map[string]bool
	suffixMap map[string][]string
}

// NewResolver creates a new resolver instance.
func NewResolver(arch *language.Architecture) *Resolver {
	r := &Resolver{
		defined:   make(map[string]bool),
		suffixMap: make(map[string][]string),
	}
	r.index(arch)
	return r
}

// index builds the lookup maps.
func (r *Resolver) index(arch *language.Architecture) {
	addID := func(id string) {
		r.defined[id] = true
		parts := strings.Split(id, ".")
		suffix := parts[len(parts)-1]
		r.suffixMap[suffix] = append(r.suffixMap[suffix], id)
	}

	// Top-level elements
	for _, cont := range arch.Containers {
		addID(cont.ID)
		for _, comp := range cont.Components {
			addID(cont.ID + "." + comp.ID)
		}
		for _, ds := range cont.DataStores {
			addID(cont.ID + "." + ds.ID)
		}
		for _, q := range cont.Queues {
			addID(cont.ID + "." + q.ID)
		}
	}
	for _, comp := range arch.Components {
		addID(comp.ID)
	}
	for _, ds := range arch.DataStores {
		addID(ds.ID)
	}
	for _, q := range arch.Queues {
		addID(q.ID)
	}
	for _, p := range arch.Persons {
		addID(p.ID)
	}

	// Nested elements under systems
	for _, sys := range arch.Systems {
		addID(sys.ID)
		for _, cont := range sys.Containers {
			contID := sys.ID + "." + cont.ID
			addID(contID)
			for _, comp := range cont.Components {
				addID(contID + "." + comp.ID)
			}
			for _, ds := range cont.DataStores {
				addID(contID + "." + ds.ID)
			}
			for _, q := range cont.Queues {
				addID(contID + "." + q.ID)
			}
		}
		for _, comp := range sys.Components {
			addID(sys.ID + "." + comp.ID)
		}
		for _, ds := range sys.DataStores {
			addID(sys.ID + "." + ds.ID)
		}
		for _, q := range sys.Queues {
			addID(sys.ID + "." + q.ID)
		}
	}
}

// resolve returns the fully qualified ID if the reference is unique/valid,
// or returns the original ref if ambiguous or unknown.
func (r *Resolver) resolveID(ref string) string {
	if ref == "" {
		return ""
	}
	if r.defined[ref] {
		return ref
	}

	var suffix string
	if !strings.Contains(ref, ".") {
		suffix = ref
	} else {
		parts := strings.Split(ref, ".")
		suffix = parts[len(parts)-1]
	}

	matches := r.suffixMap[suffix]
	if len(matches) == 1 {
		return matches[0]
	}

	return ref
}

// Resolve updates the program architecture in place.
func (r *Resolver) Resolve(arch *language.Architecture) {
	if arch == nil {
		return
	}

	// Helper to update ref
	updateRef := func(id *language.QualifiedIdent) {
		if id == nil {
			return
		}
		original := id.String()
		resolved := r.resolveID(original)
		if resolved != original && resolved != "" {
			// Update the AST node
			id.Parts = strings.Split(resolved, ".")
		}
	}

	// Update Flows
	for _, flow := range arch.Flows {
		for _, item := range flow.Items {
			if item.Step != nil {
				updateRef(&item.Step.From)
				updateRef(&item.Step.To)
			}
		}
	}

	// Update Scenarios
	for _, sc := range arch.Scenarios {
		for _, step := range sc.Steps {
			updateRef(&step.From)
			updateRef(&step.To)
		}
	}

	// Also update Relation targets if they use short names?
	// DSL relations usually are scoped, but sometimes they might use global short names.
	// Let's safe-update them too.
	updateRelation := func(rel *language.Relation) {
		if rel == nil {
			return
		}
		updateRef(&rel.From)
		updateRef(&rel.To)
	}

	for _, rel := range arch.Relations {
		updateRelation(rel)
	}

	for _, s := range arch.Systems {
		for _, r := range s.Relations {
			updateRelation(r)
		}
		for _, c := range s.Containers {
			for _, r := range c.Relations {
				updateRelation(r)
			}
		}
		for _, c := range s.Components {
			for _, r := range c.Relations {
				updateRelation(r)
			}
		}
	}
}

// RunResolution is a convenience entry point
func RunResolution(program *language.Program) {
	if program == nil || program.Architecture == nil {
		return
	}
	r := NewResolver(program.Architecture)
	r.Resolve(program.Architecture)
}

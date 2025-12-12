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
	// Estimate capacity to reduce allocations
	estimatedElements := estimateElementCount(arch)
	r := &Resolver{
		defined:   make(map[string]bool, estimatedElements),
		suffixMap: make(map[string][]string, estimatedElements/2),
	}
	r.index(arch)
	return r
}

// estimateElementCount provides a rough estimate of total elements for map pre-allocation.
func estimateElementCount(arch *language.Architecture) int {
	if arch == nil {
		return 16
	}
	count := len(arch.Containers) + len(arch.Components) + len(arch.DataStores) + len(arch.Queues) + len(arch.Persons)
	for _, sys := range arch.Systems {
		count += 1 + len(sys.Containers) + len(sys.Components) + len(sys.DataStores) + len(sys.Queues)
		for _, cont := range sys.Containers {
			count += len(cont.Components) + len(cont.DataStores) + len(cont.Queues)
		}
	}
	// Add some buffer for nested elements
	return count*2 + 32
}

// index builds the lookup maps.
func (r *Resolver) index(arch *language.Architecture) {
	addID := func(id string) {
		if id == "" {
			return
		}
		r.defined[id] = true
		// Use LastIndex for better performance than Split when we only need the last part
		lastDot := strings.LastIndex(id, ".")
		var suffix string
		if lastDot == -1 {
			suffix = id
		} else {
			suffix = id[lastDot+1:]
		}
		r.suffixMap[suffix] = append(r.suffixMap[suffix], id)
	}

	// Helper to build qualified IDs efficiently
	buildQualifiedID := func(prefix, id string) string {
		if prefix == "" {
			return id
		}
		// Pre-allocate with estimated capacity
		buf := make([]byte, 0, len(prefix)+len(id)+1)
		buf = append(buf, prefix...)
		buf = append(buf, '.')
		buf = append(buf, id...)
		return string(buf)
	}

	// Top-level elements
	for _, cont := range arch.Containers {
		addID(cont.ID)
		for _, comp := range cont.Components {
			addID(buildQualifiedID(cont.ID, comp.ID))
		}
		for _, ds := range cont.DataStores {
			addID(buildQualifiedID(cont.ID, ds.ID))
		}
		for _, q := range cont.Queues {
			addID(buildQualifiedID(cont.ID, q.ID))
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
			contID := buildQualifiedID(sys.ID, cont.ID)
			addID(contID)
			for _, comp := range cont.Components {
				addID(buildQualifiedID(contID, comp.ID))
			}
			for _, ds := range cont.DataStores {
				addID(buildQualifiedID(contID, ds.ID))
			}
			for _, q := range cont.Queues {
				addID(buildQualifiedID(contID, q.ID))
			}
		}
		for _, comp := range sys.Components {
			addID(buildQualifiedID(sys.ID, comp.ID))
		}
		for _, ds := range sys.DataStores {
			addID(buildQualifiedID(sys.ID, ds.ID))
		}
		for _, q := range sys.Queues {
			addID(buildQualifiedID(sys.ID, q.ID))
		}
	}
}

// resolveID returns the fully qualified ID if the reference is unique/valid,
// or returns the original ref if ambiguous or unknown.
func (r *Resolver) resolveID(ref string) string {
	if ref == "" {
		return ""
	}
	if r.defined[ref] {
		return ref
	}

	// Use LastIndex for better performance than Split when we only need the last part
	lastDot := strings.LastIndex(ref, ".")
	var suffix string
	if lastDot == -1 {
		suffix = ref
	} else {
		suffix = ref[lastDot+1:]
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

package engine

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Resolver updates the AST to use fully qualified names for ambiguous-free references.
// It maintains an internal cache for O(1) repeated lookups.
type Resolver struct {
	defined      map[string]bool
	suffixMap    map[string][]string
	resolveCache map[string]string   // Cache for resolved IDs (ref -> resolved)
	partsCache   map[string][]string // Cache for ID parts (FQN -> []string)
}

// NewResolverFromModel creates a new resolver instance from a LikeC4 Model.
func NewResolverFromModel(model *language.ModelBlock) *Resolver {
	if model == nil {
		return &Resolver{
			defined:      make(map[string]bool, 16),
			suffixMap:    make(map[string][]string, 8),
			resolveCache: make(map[string]string, 16),
			partsCache:   make(map[string][]string, 16),
		}
	}

	// Collect all elements
	defined, _ := collectLikeC4Elements(model)

	estimatedElements := len(defined)
	if estimatedElements < 16 {
		estimatedElements = 16
	}

	r := &Resolver{
		defined:      make(map[string]bool, estimatedElements),
		suffixMap:    make(map[string][]string, estimatedElements/2),
		resolveCache: make(map[string]string, estimatedElements),
		partsCache:   make(map[string][]string, estimatedElements),
	}
	r.indexFromModel(model)
	return r
}

// indexFromModel builds the lookup maps from LikeC4 Model.
func (r *Resolver) indexFromModel(model *language.ModelBlock) {
	if model == nil {
		return
	}

	addID := func(id string) {
		if id == "" {
			return
		}
		r.defined[id] = true
		lastDot := strings.LastIndex(id, ".")
		var suffix string
		if lastDot == -1 {
			suffix = id
		} else {
			suffix = id[lastDot+1:]
		}
		r.suffixMap[suffix] = append(r.suffixMap[suffix], id)
	}

	// Collect all element IDs from Model
	var collectIDs func(elem *language.LikeC4ElementDef, parentFQN string)
	collectIDs = func(elem *language.LikeC4ElementDef, parentFQN string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		fqn := id
		if parentFQN != "" {
			fqn = buildQualifiedID(parentFQN, id)
		}

		addID(fqn)

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					collectIDs(bodyItem.Element, fqn)
				}
			}
		}
	}

	// Process all top-level elements
	for _, item := range model.Items {
		if item.ElementDef != nil {
			collectIDs(item.ElementDef, "")
		}
	}
}

// resolveID returns the fully qualified ID if the reference is unique/valid,
// or returns the original ref if ambiguous or unknown.
// Results are cached for O(1) repeated lookups.
func (r *Resolver) resolveID(ref string) string {
	if ref == "" {
		return ""
	}

	// Check cache first (O(1) lookup)
	if cached, ok := r.resolveCache[ref]; ok {
		return cached
	}

	// Already defined - cache and return
	if r.defined[ref] {
		r.resolveCache[ref] = ref
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
	var result string
	if len(matches) == 1 {
		result = matches[0]
	} else {
		result = ref
	}

	// Cache the result
	r.resolveCache[ref] = result
	return result
}

// getParts returns the parts of a fully qualified name, cached to avoid repeated splitting.
func (r *Resolver) getParts(fqn string) []string {
	if parts, ok := r.partsCache[fqn]; ok {
		return parts
	}

	// Split and cache
	parts := strings.Split(fqn, ".")
	r.partsCache[fqn] = parts
	return parts
}

// ResolveModel updates the program model in place (LikeC4 syntax).
func (r *Resolver) ResolveModel(model *language.ModelBlock) {
	if model == nil {
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
			// Update the AST node using cached parts
			id.Parts = r.getParts(resolved)
		}
	}

	// Update relations in Model
	var updateRelationsInElement func(elem *language.LikeC4ElementDef)
	updateRelationsInElement = func(elem *language.LikeC4ElementDef) {
		body := elem.GetBody()
		if body == nil {
			return
		}

		for _, bodyItem := range body.Items {
			if bodyItem.Relation != nil {
				updateRef(&bodyItem.Relation.From)
				updateRef(&bodyItem.Relation.To)
			}
			if bodyItem.Element != nil {
				updateRelationsInElement(bodyItem.Element)
			}
		}
	}

	// Process all items in model
	for _, item := range model.Items {
		if item.Relation != nil {
			updateRef(&item.Relation.From)
			updateRef(&item.Relation.To)
		}
		if item.ElementDef != nil {
			updateRelationsInElement(item.ElementDef)
		}
		// Update scenarios and flows
		if item.Scenario != nil {
			for _, step := range item.Scenario.Steps {
				updateRef(&step.From)
				updateRef(&step.To)
			}
		}
		if item.Flow != nil {
			for _, step := range item.Flow.Steps {
				updateRef(&step.From)
				updateRef(&step.To)
			}
		}
	}
}

// RunResolution is a convenience entry point for a single program
func RunResolution(program *language.Program) {
	if program == nil || program.Model == nil {
		return
	}
	r := NewResolverFromModel(program.Model)
	r.ResolveModel(program.Model)
}

// RunWorkspaceResolution resolves references across all programs in a workspace
func RunWorkspaceResolution(ws *language.Workspace) {
	if ws == nil {
		return
	}
	merged := ws.MergedProgram()
	if merged == nil || merged.Model == nil {
		return
	}
	// Index all elements across the entire workspace
	r := NewResolverFromModel(merged.Model)

	// Resolve each program's model independently using the global index
	for _, prog := range ws.Programs {
		if prog != nil && prog.Model != nil {
			r.ResolveModel(prog.Model)
		}
	}
}

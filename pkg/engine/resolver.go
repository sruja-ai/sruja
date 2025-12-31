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

// NewResolverFromModel creates a new resolver instance from a Model.
func NewResolverFromModel(model *language.Model) *Resolver {
	if model == nil {
		return &Resolver{
			defined:      make(map[string]bool, 16),
			suffixMap:    make(map[string][]string, 8),
			resolveCache: make(map[string]string, 16),
			partsCache:   make(map[string][]string, 16),
		}
	}

	// Collect all elements using our optimized iterative function
	defined, _ := collectElements(model)

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

// indexFromModel builds the lookup maps from Model using iterative traversal.
func (r *Resolver) indexFromModel(model *language.Model) {
	if model == nil {
		return
	}

	// Use explicit stack for iterative traversal
	type frame struct {
		elem      *language.ElementDef
		parentFQN string
	}
	stack := make([]frame, 0, 16)

	// Initialize with top-level elements
	for _, item := range model.Items {
		if item.ElementDef != nil {
			stack = append(stack, frame{elem: item.ElementDef, parentFQN: ""})
		}
	}

	// Iterative traversal
	for len(stack) > 0 {
		// Pop
		f := stack[len(stack)-1]
		stack = stack[:len(stack)-1]

		elem := f.elem
		if elem == nil {
			continue
		}

		id := elem.GetID()
		if id == "" {
			continue
		}

		// Build FQN
		fqn := id
		if f.parentFQN != "" {
			fqn = buildQualifiedID(f.parentFQN, id)
		}

		// Add to defined and suffix maps
		r.defined[fqn] = true
		suffix := extractSuffix(fqn)
		r.suffixMap[suffix] = append(r.suffixMap[suffix], fqn)

		// Push children
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					stack = append(stack, frame{elem: bodyItem.Element, parentFQN: fqn})
				}
			}
		}
	}
}

// extractSuffix extracts the last part of a qualified ID.
// Uses string slicing instead of Split for efficiency.
func extractSuffix(id string) string {
	lastDot := strings.LastIndex(id, ".")
	if lastDot == -1 {
		return id
	}
	return id[lastDot+1:]
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

	// Extract suffix and look up
	suffix := extractSuffix(ref)
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

// ResolveModel updates the program model in place (Sruja syntax).
// Uses iterative traversal to avoid closure allocation overhead.
func (r *Resolver) ResolveModel(model *language.Model) {
	if model == nil {
		return
	}

	// Use explicit stack for iterative traversal
	type frame struct {
		elem *language.ElementDef
	}
	stack := make([]frame, 0, 16)

	// Process top-level relations
	for _, item := range model.Items {
		if item.Relation != nil {
			r.updateRef(&item.Relation.From)
			r.updateRef(&item.Relation.To)
		}
		if item.ElementDef != nil {
			stack = append(stack, frame{elem: item.ElementDef})
		}
	}

	// Iterative traversal for nested elements
	for len(stack) > 0 {
		// Pop
		f := stack[len(stack)-1]
		stack = stack[:len(stack)-1]

		elem := f.elem
		body := elem.GetBody()
		if body == nil {
			continue
		}

		for _, bodyItem := range body.Items {
			if bodyItem.Relation != nil {
				r.updateRef(&bodyItem.Relation.From)
				r.updateRef(&bodyItem.Relation.To)
			}
			if bodyItem.Element != nil {
				stack = append(stack, frame{elem: bodyItem.Element})
			}
		}
	}
}

// updateRef updates a QualifiedIdent to use the resolved fully qualified name.
func (r *Resolver) updateRef(id *language.QualifiedIdent) {
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

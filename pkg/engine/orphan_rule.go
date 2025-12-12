package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

type OrphanDetectionRule struct{}

func (r *OrphanDetectionRule) Name() string {
	return "OrphanDetection"
}

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *OrphanDetectionRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Architecture == nil {
		return nil
	}
	arch := program.Architecture

	// Maps to store defined elements and their locations
	// Estimate capacity: Systems + Containers (avg 5/sys?) + Components (avg 5/cont?)
	// It's a rough guess but better than 0.
	capacityEstimate := len(arch.Systems) * 10
	if capacityEstimate < 100 {
		capacityEstimate = 100
	}
	defined := make(map[string]bool, capacityEstimate)
	definedLoc := make(map[string]language.SourceLocation, capacityEstimate)
	used := make(map[string]bool, capacityEstimate)

	// Parent mapping for immediate usage propagation
	parent := make(map[string]string, capacityEstimate)

	// Pre-allocate diagnostics slice with estimated capacity
	diags := make([]diagnostics.Diagnostic, 0, capacityEstimate/10)

	// Helper to build qualified IDs efficiently
	buildQualifiedID := func(prefix, id string) string {
		if prefix == "" {
			return id
		}
		buf := make([]byte, 0, len(prefix)+len(id)+1)
		buf = append(buf, prefix...)
		buf = append(buf, '.')
		buf = append(buf, id...)
		return string(buf)
	}

	// --- 1. Populate defined elements & parent relationships ---

	// Helpers to register elements
	register := func(id string, loc language.SourceLocation, parentID string) {
		defined[id] = true
		definedLoc[id] = loc
		if parentID != "" {
			parent[id] = parentID
		}
	}

	// Iterate Systems
	for _, sys := range arch.Systems {
		register(sys.ID, sys.Location(), "")
		if len(sys.Contracts) > 0 {
			used[sys.ID] = true
		}

		for _, cont := range sys.Containers {
			contID := buildQualifiedID(sys.ID, cont.ID)
			register(contID, cont.Location(), sys.ID)
			if len(cont.Contracts) > 0 {
				used[contID] = true
			}

			for _, comp := range cont.Components {
				compID := buildQualifiedID(contID, comp.ID)
				register(compID, comp.Location(), contID)
				// component-level requirements/ADRs removed (root-only policy)
			}
			for _, ds := range cont.DataStores {
				dsID := buildQualifiedID(contID, ds.ID)
				register(dsID, ds.Location(), contID)
			}
			for _, q := range cont.Queues {
				qID := buildQualifiedID(contID, q.ID)
				register(qID, q.Location(), contID)
			}
		}

		for _, comp := range sys.Components {
			compID := buildQualifiedID(sys.ID, comp.ID)
			register(compID, comp.Location(), sys.ID)
			// component-level requirements/ADRs removed (root-only policy)
		}
		for _, ds := range sys.DataStores {
			dsID := buildQualifiedID(sys.ID, ds.ID)
			register(dsID, ds.Location(), sys.ID)
		}
		for _, q := range sys.Queues {
			qID := buildQualifiedID(sys.ID, q.ID)
			register(qID, q.Location(), sys.ID)
		}
	}

	// Iterate top-level Containers
	for _, cont := range arch.Containers {
		register(cont.ID, cont.Location(), "")
		if len(cont.Contracts) > 0 {
			used[cont.ID] = true
		}

		for _, comp := range cont.Components {
			compID := buildQualifiedID(cont.ID, comp.ID)
			register(compID, comp.Location(), cont.ID)
			// component-level requirements/ADRs removed (root-only policy)
		}
		for _, ds := range cont.DataStores {
			dsID := buildQualifiedID(cont.ID, ds.ID)
			register(dsID, ds.Location(), cont.ID)
		}
		for _, q := range cont.Queues {
			qID := buildQualifiedID(cont.ID, q.ID)
			register(qID, q.Location(), cont.ID)
		}
	}

	// Iterate other top-level elements
	for _, comp := range arch.Components {
		register(comp.ID, comp.Location(), "")
		// component-level requirements/ADRs removed (root-only policy)
	}
	for _, ds := range arch.DataStores {
		register(ds.ID, ds.Location(), "")
	}
	for _, q := range arch.Queues {
		register(q.ID, q.Location(), "")
	}
	for _, p := range arch.Persons {
		register(p.ID, p.Location(), "")
	}

	// Mark used and propagate up to parents
	markUsed := func(id string) {
		curr := id
		for curr != "" {
			if used[curr] {
				// Already marked, and since we propagate up immediately,
				// parents are also marked. We can stop.
				break
			}
			used[curr] = true
			curr = parent[curr]
		}
	}

	// --- 2. Resolve references ---

	// resolve finds the fully qualified ID for a reference 'ref' from 'scope'.
	resolve := func(ref, scope string) string {
		// 1. Try absolute/global match (fast path)
		if defined[ref] {
			return ref
		}

		// 2. Try relative to scope, walking up
		// e.g., scope="A.B", ref="C" -> check "A.B.C", then "A.C", then "C" (global check again)
		// Optimization: string manipulation without Split/Join
		currScope := scope
		for {
			if currScope == "" {
				// We've reached the top.
				// We already checked the global 'ref' at step 1.
				// But we need to check suffix matches for global fallback logic?
				// The original code had a global fallback loop:
				// "For architecture-level relations (scope=""), search all defined elements..."
				// If scope was empty at start, we fell through to that loop.
				// If we walked up to empty scope, do we do the same?
				// Logic: If I am in "A", referencing "B". Check "A.B". If not found, check "B".
				// "B" is handled by step 1 if "B" is a top-level ID.
				// What if "B" is "Sys1.B"? (ambiguous short ID).
				// Original code: if scope=="" { search suffix }
				// So if we walk up to root, we are effectively at scope="".
				break
			}

			// Check canonical candidate: scope + "." + ref
			// Build efficiently without string concatenation
			candidateLen := len(currScope) + len(ref) + 1
			candidateBuf := make([]byte, 0, candidateLen)
			candidateBuf = append(candidateBuf, currScope...)
			candidateBuf = append(candidateBuf, '.')
			candidateBuf = append(candidateBuf, ref...)
			candidate := string(candidateBuf)
			if defined[candidate] {
				return candidate
			}

			// Move up one level
			lastDot := strings.LastIndexByte(currScope, '.')
			if lastDot == -1 {
				currScope = ""
			} else {
				currScope = currScope[:lastDot]
			}
		}

		// 3. Global Suffix Search (fallback)
		// Only if scope was empty or became empty?
		// Original logic: "if scope == ''". This implies this fallback only runs if the *call* had empty scope.
		// If we are inside a scope "A", and fail to resolve "A.ref" or "ref" (step 1),
		// should we search for "SomeOtherSys.ref"?
		// The original code only ran this block `if scope == ""`.
		// So we preserve that behavior.
		if scope == "" {
			// This is O(N) where N = total defined elements.
			// But usually only runs for top-level relations provided without full qualification.
			// Optimization: check if 'ref' usage is common enough to optimize?
			// Ideally we'd have a reverse index: shortName -> []fullNames.
			// But for now, let's keep the loop but make it slightly cleaner?
			// Or better: build the index lazily? No, concurrency issues if rule reused? No, strictly local.
			// Let's stick to the loop but optimize the check.

			for id := range defined {
				// We need id that ends with "." + ref
				// e.g. id="Sys.Comp", ref="Comp" -> Match.
				// id="Comp", ref="Comp" -> Handled by step 1.

				// string.HasSuffix check
				if strings.HasSuffix(id, ref) {
					// Must be preceded by dot, to avoid partial match (e.g. "MyComp" matching "Comp")
					// We need len(id) > len(ref) + 1 (for dot and at least one char before)
					suffixLen := len(ref)
					if len(id) > suffixLen+1 {
						if id[len(id)-suffixLen-1] == '.' {
							return id
						}
					}
				}
			}
		}

		return ""
	}

	markRel := func(from, to, scope string) {
		rFrom := resolve(from, scope)
		if rFrom != "" {
			markUsed(rFrom)
		}
		rTo := resolve(to, scope)
		if rTo != "" {
			markUsed(rTo)
		}
	}

	// --- 3. Process Relations ---

	// Global relations
	for _, r := range arch.Relations {
		markRel(r.From.String(), r.To.String(), "")
	}

	// System scope
	for _, s := range arch.Systems {
		sID := s.ID
		for _, r := range s.Relations {
			markRel(r.From.String(), r.To.String(), sID)
		}
		// Container scope
		for _, c := range s.Containers {
			contID := buildQualifiedID(s.ID, c.ID)
			for _, r := range c.Relations {
				markRel(r.From.String(), r.To.String(), contID)
			}
		}
		// Component scope
		for _, comp := range s.Components {
			compID := buildQualifiedID(s.ID, comp.ID)
			for _, r := range comp.Relations {
				markRel(r.From.String(), r.To.String(), compID)
			}
		}
	}

	// --- 4. Identify Orphans ---

	// Since we propagated usage up on the fly, 'used' map is already fully populated.
	// Just check defined vs used.

	for id := range defined {
		if used[id] {
			continue
		}
		loc := definedLoc[id]
		shortID := id
		lastDot := strings.LastIndexByte(id, '.')
		if lastDot != -1 {
			shortID = id[lastDot+1:]
		}
		// Build enhanced error message with suggestions
		var msgSb strings.Builder
		msgSb.Grow(len(shortID) + 80)
		msgSb.WriteString("Orphan element '")
		msgSb.WriteString(shortID)
		msgSb.WriteString("' is defined but never used in any relation")
		if id != shortID {
			msgSb.WriteString(fmt.Sprintf(" (full ID: '%s')", id))
		}
		msgSb.WriteString(".")

		var suggestions []string
		suggestions = append(suggestions, "Add a relation that uses this element (e.g., 'A -> "+shortID+"')")
		suggestions = append(suggestions, "If this element is intentionally unused, you can ignore this warning")
		suggestions = append(suggestions, "Consider removing the element if it's not needed")

		diags = append(diags, diagnostics.Diagnostic{
			Code:        diagnostics.CodeOrphanElement,
			Severity:    diagnostics.SeverityWarning,
			Message:     msgSb.String(),
			Location:    diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
			Suggestions: suggestions,
		})
	}

	return diags
}

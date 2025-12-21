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
	if program == nil || program.Model == nil {
		return nil
	}

	// Collect all elements from LikeC4 Model
	defined, _ := collectLikeC4Elements(program.Model)

	// Maps to store element locations and usage
	definedLoc := make(map[string]language.SourceLocation, len(defined))
	used := make(map[string]bool, len(defined))
	parent := make(map[string]string, len(defined))

	// Pre-allocate diagnostics slice
	diags := make([]diagnostics.Diagnostic, 0, len(defined)/10)

	// Helper to build qualified IDs efficiently
	buildQualifiedIDFromParts := func(prefix, id string) string {
		if prefix == "" {
			return id
		}
		return buildQualifiedID(prefix, id)
	}

	// --- 1. Populate defined elements & parent relationships ---
	var registerElement func(elem *language.LikeC4ElementDef, parentFQN string)
	registerElement = func(elem *language.LikeC4ElementDef, parentFQN string) {
		if elem == nil {
			return
		}

		id := elem.GetID()
		if id == "" {
			return
		}

		fqn := id
		if parentFQN != "" {
			fqn = buildQualifiedIDFromParts(parentFQN, id)
		}

		loc := elem.Location()
		definedLoc[fqn] = loc
		if parentFQN != "" {
			parent[fqn] = parentFQN
		}

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					registerElement(bodyItem.Element, fqn)
				}
			}
		}
	}

	// Register all top-level elements
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			registerElement(item.ElementDef, "")
		}
		// Also register other top-level items (scenarios, ADRs, etc.) if they can be referenced
		if item.Scenario != nil {
			definedLoc[item.Scenario.ID] = item.Scenario.Location()
		}
		if item.ADR != nil {
			definedLoc[item.ADR.ID] = item.ADR.Location()
		}
		if item.Requirement != nil {
			definedLoc[item.Requirement.ID] = item.Requirement.Location()
		}
	}

	// Mark used and propagate up to parents
	markUsed := func(id string) {
		curr := id
		for curr != "" {
			if used[curr] {
				break
			}
			used[curr] = true
			curr = parent[curr]
		}
	}

	// --- 2. Resolve references ---
	resolve := func(ref, scope string) string {
		// 1. Try absolute/global match (fast path)
		if defined[ref] {
			return ref
		}

		// 2. Try relative to scope, walking up
		currScope := scope
		for {
			if currScope == "" {
				break
			}
			candidate := buildQualifiedIDFromParts(currScope, ref)
			if defined[candidate] {
				return candidate
			}
			lastDot := strings.LastIndexByte(currScope, '.')
			if lastDot == -1 {
				currScope = ""
			} else {
				currScope = currScope[:lastDot]
			}
		}

		// 3. Global Suffix Search (fallback)
		if scope == "" {
			for id := range defined {
				if strings.HasSuffix(id, ref) {
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
	relationsWithScope := collectAllRelations(program.Model)
	for _, relScope := range relationsWithScope {
		rel := relScope.Relation
		markRel(rel.From.String(), rel.To.String(), relScope.Scope)
	}

	// --- 4. Identify Orphans ---
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

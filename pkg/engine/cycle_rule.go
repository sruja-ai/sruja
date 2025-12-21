package engine

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

type CycleDetectionRule struct{}

func (r *CycleDetectionRule) Name() string {
	return "CycleDetection"
}

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *CycleDetectionRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic
	if program == nil || program.Model == nil {
		return diags
	}

	// Collect all elements and relations from LikeC4 Model
	defined, relations := collectLikeC4Elements(program.Model)

	// Pre-allocate adjacency list with estimated capacity
	estimatedNodes := len(defined)
	if estimatedNodes < 16 {
		estimatedNodes = 16
	}
	adj := make(map[string][]string, estimatedNodes)

	// Build adjacency list from relations
	add := func(from, to string) {
		if from != "" && to != "" {
			adj[from] = append(adj[from], to)
		}
	}

	// Add all relations to adjacency list
	for _, rel := range relations {
		add(rel.From.String(), rel.To.String())
	}

	visited := make(map[string]bool, len(defined))
	recStack := make(map[string]bool, len(defined))

	// Build location map for better error reporting
	locMap := make(map[string]diagnostics.SourceLocation, len(defined))

	// Collect element locations from LikeC4 Model
	var collectLocations func(elem *language.LikeC4ElementDef, parentFQN string)
	collectLocations = func(elem *language.LikeC4ElementDef, parentFQN string) {
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

		loc := elem.Location()
		locMap[fqn] = diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column}

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					collectLocations(bodyItem.Element, fqn)
				}
			}
		}
	}

	// Collect locations from all top-level elements
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			collectLocations(item.ElementDef, "")
		}
	}

	// Optimization: Use a single path slice with backtracking to avoid unnecessary allocations
	path := make([]string, 0, 16) // Pre-allocate with small capacity

	var dfs func(u string)
	dfs = func(u string) {
		visited[u] = true
		recStack[u] = true
		path = append(path, u)

		for _, v := range adj[u] {
			if !visited[v] {
				dfs(v)
			} else if recStack[v] {
				// Found a cycle - construct the cycle path efficiently
				// Find start index of v in current path
				startIdx := -1
				for i := len(path) - 1; i >= 0; i-- {
					if path[i] == v {
						startIdx = i
						break
					}
				}

				if startIdx != -1 {
					// Create source location for the cycle start node
					loc := locMap[v]

					// Extract the cycle segment efficiently
					cycleLen := len(path) - startIdx + 1
					cycleNodes := make([]string, 0, cycleLen)
					cycleNodes = append(cycleNodes, path[startIdx:]...)
					cycleNodes = append(cycleNodes, v)

					// Build enhanced error message with cycle path
					var msgSb strings.Builder
					cyclePath := strings.Join(cycleNodes, " -> ")
					msgSb.Grow(len(cyclePath) + 120)
					msgSb.WriteString("Circular dependency detected: ")
					msgSb.WriteString(cyclePath)

					var suggestions []string
					suggestions = append(suggestions, "Cycles are valid for feedback loops, event-driven patterns, or mutual dependencies")
					suggestions = append(suggestions, "If this is unintended, consider breaking the cycle by introducing an intermediate element")
					suggestions = append(suggestions, "Review the architecture to ensure the cycle represents the intended design")

					diags = append(diags, diagnostics.Diagnostic{
						Code:        diagnostics.CodeCycleDetected,
						Severity:    diagnostics.SeverityInfo,
						Message:     msgSb.String(),
						Location:    loc,
						Suggestions: suggestions,
					})
				}
			}
		}

		// Backtrack
		recStack[u] = false
		path = path[:len(path)-1]
	}

	for node := range adj {
		if !visited[node] {
			dfs(node)
		}
	}
	return diags
}

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

// estimateNodeCount provides a rough estimate of nodes for map pre-allocation.
func estimateNodeCount(arch *language.Architecture) int {
	if arch == nil {
		return 16
	}
	count := len(arch.Relations)
	for _, sys := range arch.Systems {
		count += len(sys.Relations)
		for _, c := range sys.Containers {
			count += len(c.Relations)
		}
		for _, comp := range sys.Components {
			count += len(comp.Relations)
		}
	}
	// Estimate unique nodes (typically fewer than relations)
	return count/2 + 32
}

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *CycleDetectionRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic
	if program == nil || program.Architecture == nil {
		return diags
	}
	// Pre-allocate adjacency list with estimated capacity
	estimatedNodes := estimateNodeCount(program.Architecture)
	adj := make(map[string][]string, estimatedNodes)

	arch := program.Architecture
	add := func(from, to string) {
		if from != "" && to != "" {
			adj[from] = append(adj[from], to)
		}
	}
	for _, r := range arch.Relations {
		add(r.From.String(), r.To.String())
	}
	for _, s := range arch.Systems {
		for _, r := range s.Relations {
			add(r.From.String(), r.To.String())
		}
		for _, c := range s.Containers {
			for _, r := range c.Relations {
				add(r.From.String(), r.To.String())
			}
		}
		for _, comp := range s.Components {
			for _, r := range comp.Relations {
				add(r.From.String(), r.To.String())
			}
		}
	}

	visited := make(map[string]bool, estimatedNodes)
	recStack := make(map[string]bool, estimatedNodes)

	// Build location map for better error reporting
	// Estimate capacity based on systems and their nested elements
	estimatedLocations := len(arch.Systems) * 4 // Rough estimate
	locMap := make(map[string]diagnostics.SourceLocation, estimatedLocations)

	// Helper to build qualified IDs efficiently
	buildQualifiedID := func(parts ...string) string {
		if len(parts) == 0 {
			return ""
		}
		if len(parts) == 1 {
			return parts[0]
		}
		// Estimate total length
		totalLen := len(parts) - 1 // for dots
		for _, p := range parts {
			totalLen += len(p)
		}
		buf := make([]byte, 0, totalLen)
		buf = append(buf, parts[0]...)
		for i := 1; i < len(parts); i++ {
			buf = append(buf, '.')
			buf = append(buf, parts[i]...)
		}
		return string(buf)
	}

	for _, sys := range arch.Systems {
		l := sys.Location()
		locMap[sys.ID] = diagnostics.SourceLocation{File: l.File, Line: l.Line, Column: l.Column}
		for _, c := range sys.Containers {
			cl := c.Location()
			contID := buildQualifiedID(sys.ID, c.ID)
			locMap[contID] = diagnostics.SourceLocation{File: cl.File, Line: cl.Line, Column: cl.Column}
			for _, cmp := range c.Components {
				cmpl := cmp.Location()
				compID := buildQualifiedID(sys.ID, c.ID, cmp.ID)
				locMap[compID] = diagnostics.SourceLocation{File: cmpl.File, Line: cmpl.Line, Column: cmpl.Column}
			}
		}
		for _, cmp := range sys.Components {
			cmpl := cmp.Location()
			compID := buildQualifiedID(sys.ID, cmp.ID)
			locMap[compID] = diagnostics.SourceLocation{File: cmpl.File, Line: cmpl.Line, Column: cmpl.Column}
		}
	}
	// Add top-level containers/components/etc if needed, similar to orphan rule
	// Also include other elements like Person, etc if they can be part of cycles.
	// Previous code iterated Systems only, need to be comprehensive if relations exist elsewhere.
	// The previous `add` function iterated everything.

	// For now, let's keep it simple and just add the basic hierarchy as above.

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

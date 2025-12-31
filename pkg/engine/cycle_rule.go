package engine

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// CycleDetectionRule detects circular dependencies in the architecture.
type CycleDetectionRule struct{}

// Name returns the rule name.
func (r *CycleDetectionRule) Name() string {
	return "CycleDetection"
}

// cycleSuggestions are pre-allocated suggestions for cycle diagnostics.
// This avoids allocating the same strings repeatedly.
var cycleSuggestions = []string{
	"Cycles are valid for feedback loops, event-driven patterns, or mutual dependencies",
	"If this is unintended, consider breaking the cycle by introducing an intermediate element",
	"Review the architecture to ensure the cycle represents the intended design",
}

//nolint:funlen,gocyclo // Validation logic is long and complex
func (r *CycleDetectionRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Model == nil {
		return nil
	}

	// Collect all elements and relations from Model
	_, relations := collectElements(program.Model)

	// Use pooled adjacency list map
	adjPtr := GetStringSliceMap()
	adj := *adjPtr

	// Build adjacency list from relations
	for _, rel := range relations {
		from := rel.From.String()
		to := rel.To.String()
		if from != "" && to != "" {
			adj[from] = append(adj[from], to)
		}
	}

	// Use pooled visited and recStack maps
	visitedPtr := GetStringBoolMap()
	recStackPtr := GetStringBoolMap()
	visited := *visitedPtr
	recStack := *recStackPtr

	// Use pooled location map
	locMapPtr := GetSourceLocationMap()
	locMap := *locMapPtr

	// Collect element locations from Model using iterative approach
	// to avoid closure allocation overhead
	collectLocationsIterative(program.Model, locMap)

	// Use pooled path slice
	pathPtr := GetStringSlice()
	path := *pathPtr

	// Pre-allocate diagnostics slice
	var diags []diagnostics.Diagnostic

	// Iterative DFS using explicit stack to avoid closure overhead
	type stackFrame struct {
		node     string
		childIdx int
	}
	stack := make([]stackFrame, 0, 16)

	for startNode := range adj {
		if visited[startNode] {
			continue
		}

		// Push start node
		stack = append(stack, stackFrame{node: startNode, childIdx: 0})
		path = append(path, startNode)
		visited[startNode] = true
		recStack[startNode] = true

		for len(stack) > 0 {
			frame := &stack[len(stack)-1]
			node := frame.node
			children := adj[node]

			// Process next child
			foundUnvisited := false
			for frame.childIdx < len(children) {
				child := children[frame.childIdx]
				frame.childIdx++

				if !visited[child] {
					// Push child to stack
					stack = append(stack, stackFrame{node: child, childIdx: 0})
					path = append(path, child)
					visited[child] = true
					recStack[child] = true
					foundUnvisited = true
					break
				} else if recStack[child] {
					// Found a cycle - construct the cycle path
					diag := buildCycleDiagnostic(path, child, locMap)
					diags = append(diags, diag)
				}
			}

			if !foundUnvisited {
				// Pop from stack - all children processed
				recStack[node] = false
				path = path[:len(path)-1]
				stack = stack[:len(stack)-1]
			}
		}
	}

	// Return pooled resources
	*pathPtr = path
	PutStringSlice(pathPtr)
	PutStringBoolMap(visitedPtr)
	PutStringBoolMap(recStackPtr)
	PutSourceLocationMap(locMapPtr)
	PutStringSliceMap(adjPtr)

	return diags
}

// collectLocationsIterative collects element locations without closures.
func collectLocationsIterative(model *language.Model, locMap map[string]diagnostics.SourceLocation) {
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

		// Store location
		loc := elem.Location()
		locMap[fqn] = diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column}

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

// buildCycleDiagnostic constructs a diagnostic for a detected cycle.
func buildCycleDiagnostic(path []string, cycleStart string, locMap map[string]diagnostics.SourceLocation) diagnostics.Diagnostic {
	// Find start index of cycle in path
	startIdx := -1
	for i := len(path) - 1; i >= 0; i-- {
		if path[i] == cycleStart {
			startIdx = i
			break
		}
	}

	if startIdx == -1 {
		startIdx = 0
	}

	// Build cycle path string efficiently
	cycleLen := len(path) - startIdx + 1
	var sb strings.Builder
	// Estimate: each node ~20 chars + " -> " (4 chars)
	sb.Grow(cycleLen*24 + 30)
	sb.WriteString("Circular dependency detected: ")

	for i := startIdx; i < len(path); i++ {
		if i > startIdx {
			sb.WriteString(" -> ")
		}
		sb.WriteString(path[i])
	}
	sb.WriteString(" -> ")
	sb.WriteString(cycleStart)

	return diagnostics.Diagnostic{
		Code:        diagnostics.CodeCycleDetected,
		Severity:    diagnostics.SeverityInfo,
		Message:     sb.String(),
		Location:    locMap[cycleStart],
		Suggestions: cycleSuggestions, // Use pre-allocated suggestions
	}
}

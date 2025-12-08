package engine

import (
	"fmt"
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
	if program == nil || program.Architecture == nil {
		return diags
	}
	adj := make(map[string][]string)

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

	visited := make(map[string]bool)
	recStack := make(map[string]bool)

	// Build location map for better error reporting
	locMap := make(map[string]diagnostics.SourceLocation)
	for _, sys := range arch.Systems {
		l := sys.Location()
		locMap[sys.ID] = diagnostics.SourceLocation{File: l.File, Line: l.Line, Column: l.Column}
		for _, c := range sys.Containers {
			cl := c.Location()
			locMap[sys.ID+"."+c.ID] = diagnostics.SourceLocation{File: cl.File, Line: cl.Line, Column: cl.Column}
			for _, cmp := range c.Components {
				cmpl := cmp.Location()
				locMap[sys.ID+"."+c.ID+"."+cmp.ID] = diagnostics.SourceLocation{File: cmpl.File, Line: cmpl.Line, Column: cmpl.Column}
			}
		}
		for _, cmp := range sys.Components {
			cmpl := cmp.Location()
			locMap[sys.ID+"."+cmp.ID] = diagnostics.SourceLocation{File: cmpl.File, Line: cmpl.Line, Column: cmpl.Column}
		}
	}
	// Add top-level containers/components/etc if needed, similar to orphan rule
	// Also include other elements like Person, etc if they can be part of cycles.
	// Previous code iterated Systems only, need to be comprehensive if relations exist elsewhere.
	// The previous `add` function iterated everything.

	// For now, let's keep it simple and just add the basic hierarchy as above.

	var dfs func(u string, path []string)
	dfs = func(u string, path []string) {
		visited[u] = true
		recStack[u] = true
		path = append(path, u)
		for _, v := range adj[u] {
			if !visited[v] {
				dfs(v, path)
			} else if recStack[v] {
				// Found a cycle - construct the cycle path efficiently
				cyclePath := make([]string, 0, len(path)+1)
				startIdx := -1
				for i, node := range path {
					if node == v {
						startIdx = i
						break
					}
				}
				if startIdx != -1 {
					cyclePath = append(cyclePath, path[startIdx:]...)
					cyclePath = append(cyclePath, v)

					// Use location of the start node of the cycle if available
					loc := locMap[v]

					diags = append(diags, diagnostics.Diagnostic{
						Code:     diagnostics.CodeCycleDetected,
						Severity: diagnostics.SeverityInfo,
						Message:  fmt.Sprintf("Cycle detected: %s (this is valid for feedback loops, event-driven patterns, or mutual dependencies)", strings.Join(cyclePath, " -> ")),
						Location: loc,
					})
				}
			}
		}
		recStack[u] = false
	}

	for node := range adj {
		if !visited[node] {
			dfs(node, []string{})
		}
	}
	return diags
}

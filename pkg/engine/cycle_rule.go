package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

type CycleDetectionRule struct{}

func (r *CycleDetectionRule) Name() string {
	return "CycleDetection"
}

func (r *CycleDetectionRule) Validate(program *language.Program) []ValidationError {
	errors := []ValidationError{}
	adj := make(map[string][]string)

	arch := program.Architecture
	if arch == nil {
		return errors
	}
	add := func(from, to string) {
		if from != "" && to != "" {
			adj[from] = append(adj[from], to)
		}
	}
	for _, r := range arch.Relations {
		add(r.From, r.To)
	}
	for _, s := range arch.Systems {
		for _, r := range s.Relations {
			add(r.From, r.To)
		}
		for _, c := range s.Containers {
			for _, r := range c.Relations {
				add(r.From, r.To)
			}
		}
		for _, comp := range s.Components {
			for _, r := range comp.Relations {
				add(r.From, r.To)
			}
		}
	}

	visited := make(map[string]bool)
	recStack := make(map[string]bool)

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
					errors = append(errors, ValidationError{
						Message: fmt.Sprintf("Cycle detected: %s", strings.Join(cyclePath, " -> ")),
						Line:    0,
						Column:  0,
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
	return errors
}

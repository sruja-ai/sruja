package json

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

// buildQualifiedIDForView builds a qualified ID efficiently for view keys.
func buildQualifiedIDForView(parts ...string) string {
	if len(parts) == 0 {
		return ""
	}
	if len(parts) == 1 {
		return parts[0]
	}
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

// buildEdgeID builds an edge ID efficiently.
func buildEdgeID(fromID, toID string, index int) string {
	// Estimate capacity: "edge-" (5) + fromID + "-" (1) + toID + "-" (1) + index (up to 10 digits) + null term
	estimatedLen := 5 + len(fromID) + 1 + len(toID) + 1 + 10
	buf := make([]byte, 0, estimatedLen)
	buf = append(buf, "edge-"...)
	buf = append(buf, fromID...)
	buf = append(buf, '-')
	buf = append(buf, toID...)
	buf = append(buf, '-')
	// Convert index to string efficiently
	if index == 0 {
		buf = append(buf, '0')
	} else {
		indexStr := fmt.Sprintf("%d", index)
		buf = append(buf, indexStr...)
	}
	return string(buf)
}

// Helper functions

func labelOrID(label, id string) string {
	if label != "" {
		return label
	}
	return id
}

func ptrStr(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

// getRelationLabel returns the relation's label, falling back to verb if label is not set.
// Most DSL relations only specify verb (first string after arrow), not label (second string).
func getRelationLabel(r *language.Relation) string {
	if r.Label != nil && *r.Label != "" {
		return *r.Label
	}
	if r.Verb != nil && *r.Verb != "" {
		return *r.Verb
	}
	return ""
}

func getContainerTechnology(c *language.Container) string {
	// Technology is stored in Items
	for i := range c.Items {
		item := c.Items[i]
		if item.Technology != nil {
			return *item.Technology
		}
	}
	return ""
}

func getLastPart(id string) string {
	for i := len(id) - 1; i >= 0; i-- {
		if id[i] == '.' {
			return id[i+1:]
		}
	}
	return id
}

func getL1ID(id string) string {
	// For L1 view, we want the first part of FQN (system ID)
	for i := 0; i < len(id); i++ {
		if id[i] == '.' {
			return id[:i]
		}
	}
	return id
}

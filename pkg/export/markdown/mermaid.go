// pkg/export/markdown/mermaid.go
// Package markdown provides Mermaid diagram generation.
//
//nolint:gocritic // Use WriteString for consistency
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// generateSystemDiagram generates a Mermaid flowchart for system context (C4 Level 1)
func generateSystemDiagram(arch *language.Architecture, config MermaidConfig) string {
	var sb strings.Builder

	writeMermaidConfig(&sb, config)
	sb.WriteString(fmt.Sprintf("graph %s\n", graphDirection(config)))
	sb.WriteString("\n")

	// Define C4-style classes with light backgrounds and dark text for readability
	sb.WriteString("    classDef personStyle fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000\n")
	sb.WriteString("    classDef systemStyle fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000\n")
	sb.WriteString("\n")

	// Add persons as nodes
	for _, person := range arch.Persons {
		nodeID := sanitizeNodeID(person.ID)
		label := person.Label
		if label == "" {
			label = person.ID
		}
		// Escape quotes in label
		label = strings.ReplaceAll(label, `"`, `\"`)
		sb.WriteString(fmt.Sprintf("    %s[\"%s\"]\n", nodeID, label))
		sb.WriteString(fmt.Sprintf("    class %s personStyle\n", nodeID))
	}

	// Add systems as nodes (context only)
	for _, sys := range arch.Systems {
		nodeID := sanitizeNodeID(sys.ID)
		label := sys.Label
		if label == "" {
			label = sys.ID
		}
		label = strings.ReplaceAll(label, `"`, `\"`)
		sb.WriteString(fmt.Sprintf("    %s[\"%s\"]\n", nodeID, label))
		sb.WriteString(fmt.Sprintf("    class %s systemStyle\n", nodeID))
	}

	sb.WriteString("\n")

	// Add relations aggregated to system/person level
	containerToSystem := buildContainerSystemMap(arch)
	for _, rel := range arch.Relations {
		fromNode, fromOk := resolveOverviewNode(arch, containerToSystem, rel.From.String())
		toNode, toOk := resolveOverviewNode(arch, containerToSystem, rel.To.String())
		if !fromOk || !toOk {
			continue
		}
		// Map any qualified endpoints to top-level system/person nodes
		if fromNode == toNode {
			continue
		}
		fromID := sanitizeNodeID(fromNode)
		toID := sanitizeNodeID(toNode)

		label := ""
		if rel.Verb != nil {
			label = *rel.Verb
		} else if rel.Label != nil {
			label = *rel.Label
		}
		label = strings.ReplaceAll(label, `"`, `\"`)
		if label != "" {
			sb.WriteString(fmt.Sprintf("    %s -->|%s| %s\n", fromID, label, toID))
		} else {
			sb.WriteString(fmt.Sprintf("    %s --> %s\n", fromID, toID))
		}
	}

	return sb.String()
}

// generateSystemContainerDiagram generates a Mermaid flowchart for container view (C4 Level 2)
func generateSystemContainerDiagram(sys *language.System, arch *language.Architecture, config MermaidConfig) string {
	var sb strings.Builder

	writeMermaidConfig(&sb, config)
	sb.WriteString(fmt.Sprintf("graph %s\n", graphDirection(config)))
	sb.WriteString("\n")

	writeMermaidStyles(&sb)

	// Add the system as a subgraph boundary
	sysID := sanitizeNodeID(sys.ID)
	sysLabel := sys.Label
	if sysLabel == "" {
		sysLabel = sys.ID
	}
	sysLabel = strings.ReplaceAll(sysLabel, "\"", "\\\"")
	sb.WriteString(fmt.Sprintf("    subgraph %s[\"%s\"]\n", sysID, sysLabel))

	renderContainerNodes(&sb, sys)

	sb.WriteString("    end\n")
	sb.WriteString("\n")

	// Map component IDs to their parent container
	compToContainer := make(map[string]string)
	for _, cont := range sys.Containers {
		for _, comp := range cont.Components {
			compToContainer[comp.ID] = cont.ID
		}
	}

	// Collect edges
	edgeSet := make(map[string]bool)

	// Helper to add edges to set
	addEdge := func(fromRaw, toRaw, lbl string) {
		fromName, okFrom := resolveSystemElement(sys, fromRaw, compToContainer)
		toName, okTo := resolveSystemElement(sys, toRaw, compToContainer)
		if !okFrom || !okTo {
			return
		}
		key := fmt.Sprintf("%s=>%s|%s", fromName, toName, lbl)
		if !edgeSet[key] {
			edgeSet[key] = true
		}
	}

	// Internal relations
	for _, rel := range sys.Relations {
		addEdge(rel.From.String(), rel.To.String(), relationLabel(rel))
	}
	if arch != nil {
		for _, rel := range arch.Relations {
			addEdge(rel.From.String(), rel.To.String(), relationLabel(rel))
		}
	}
	for _, cont := range sys.Containers {
		for _, rel := range cont.Relations {
			addEdge(rel.From.String(), rel.To.String(), relationLabel(rel))
		}
	}

	// Render internal edges
	for key := range edgeSet {
		parts := strings.Split(key, "=>")
		if len(parts) != 2 {
			continue
		}
		fromName := parts[0]
		seg := strings.Split(parts[1], "|")
		toName := seg[0]
		label := ""
		if len(seg) > 1 {
			label = seg[1]
		}
		renderEdge(&sb, fromName, toName, label)
	}

	// External interactions
	containerToSystem := buildContainerSystemMap(arch)
	resolver := func(name string) (string, bool) {
		return resolveSystemElement(sys, name, compToContainer)
	}
	renderExternalRelations(&sb, arch, resolver, containerToSystem)

	return sb.String()
}

// generateContainerComponentDiagram generates a Mermaid flowchart for component view (C4 Level 3)
//
//nolint:funlen,gocyclo // Mermaid generation is complex
func generateContainerComponentDiagram(container *language.Container, sys *language.System, arch *language.Architecture, config MermaidConfig) string {
	var sb strings.Builder

	writeMermaidConfig(&sb, config)
	sb.WriteString(fmt.Sprintf("graph %s\n", graphDirection(config)))
	sb.WriteString("\n")

	writeMermaidStyles(&sb)

	// Components
	compIDs := renderComponentNodes(&sb, container)

	// System-level resources (for external dependencies)
	sysElementIDs := make(map[string]string) // id -> kind
	for _, ds := range sys.DataStores {
		sysElementIDs[ds.ID] = "db"
	}
	for _, q := range sys.Queues {
		sysElementIDs[q.ID] = "queue"
	}
	for _, c := range sys.Containers {
		sysElementIDs[c.ID] = "container"
	}

	// Add external resource nodes referenced by relations
	externalAdded := make(map[string]bool)
	addExternalNode := func(name string) string {
		parts := strings.Split(name, ".")
		base := parts[len(parts)-1]
		if kind, ok := sysElementIDs[base]; ok {
			if externalAdded[base] {
				return base
			}
			id := sanitizeNodeID(base)
			safeBase := strings.ReplaceAll(base, "\"", "\\\"")
			switch kind {
			case "db":
				sb.WriteString(fmt.Sprintf("    %s[(\"%s\")]\n", id, safeBase))
				sb.WriteString(fmt.Sprintf("    class %s databaseStyle\n", id))
			case "queue":
				sb.WriteString(fmt.Sprintf("    %s[\"%s\"]\n", id, safeBase))
				sb.WriteString(fmt.Sprintf("    class %s queueStyle\n", id))
			case "container":
				sb.WriteString(fmt.Sprintf("    %s[\"%s\"]\n", id, safeBase))
				sb.WriteString(fmt.Sprintf("    class %s containerStyle\n", id))
			}
			externalAdded[base] = true
			return base
		}
		return ""
	}

	// Collect edges from component-level relations
	edgeSet := make(map[string]bool)
	for _, rel := range container.Relations {
		fromParts := strings.Split(rel.From.String(), ".")
		toParts := strings.Split(rel.To.String(), ".")
		from := fromParts[len(fromParts)-1]
		to := toParts[len(toParts)-1]

		var fromNode, toNode string
		if compIDs[from] {
			fromNode = from
		}
		if compIDs[to] {
			toNode = to
		}

		if fromNode == "" {
			fromNode = addExternalNode(from)
		}
		if toNode == "" {
			toNode = addExternalNode(to)
		}

		if fromNode == "" || toNode == "" {
			continue
		}

		key := fmt.Sprintf("%s=>%s|%s", fromNode, toNode, relationLabel(rel))
		if !edgeSet[key] {
			edgeSet[key] = true
		}
	}

	// Render edges
	for key := range edgeSet {
		parts := strings.Split(key, "=>")
		if len(parts) != 2 {
			continue
		}
		from := parts[0]
		seg := strings.Split(parts[1], "|")
		to := seg[0]
		label := ""
		if len(seg) > 1 {
			label = seg[1]
		}
		renderEdge(&sb, from, to, label)
	}

	// Resolver for component/internal or summarized system resources
	resolveElementComp := func(name string) (string, bool) {
		parts := strings.Split(name, ".")
		base := parts[len(parts)-1]
		if compIDs[base] {
			return base, true
		}
		if _, ok := sysElementIDs[base]; ok {
			return base, true
		}
		return "", false
	}

	// External interactions
	containerToSystem := buildContainerSystemMap(arch)
	renderExternalRelations(&sb, arch, resolveElementComp, containerToSystem)

	return sb.String()
}

// generateScenarioDiagram generates a Mermaid sequence diagram for a scenario
func generateScenarioDiagram(scenario *language.Scenario, config MermaidConfig) string {
	var sb strings.Builder

	writeMermaidConfig(&sb, config)
	sb.WriteString("sequenceDiagram\n")

	// Collect unique participants
	participants := make(map[string]bool)
	for _, step := range scenario.Steps {
		participants[step.From.String()] = true
		participants[step.To.String()] = true
	}

	// Add participants
	for participant := range participants {
		sb.WriteString(fmt.Sprintf("    participant %s\n", sanitizeNodeID(participant)))
	}

	// Add steps
	for _, step := range scenario.Steps {
		fromID := sanitizeNodeID(step.From.String())
		toID := sanitizeNodeID(step.To.String())
		label := "interaction"
		if step.Description != nil {
			label = *step.Description
		}
		sb.WriteString(fmt.Sprintf("    %s->>%s: %s\n", fromID, toID, label))
	}

	return sb.String()
}

// sanitizeNodeID sanitizes a node ID for use in Mermaid diagrams
func sanitizeNodeID(id string) string {
	// Replace spaces and special characters with underscores
	result := strings.ReplaceAll(id, " ", "_")
	result = strings.ReplaceAll(result, "-", "_")
	result = strings.ReplaceAll(result, ".", "_")
	// Remove any remaining special characters
	var sb strings.Builder
	for _, r := range result {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || r == '_' {
			sb.WriteRune(r)
		}
	}
	return sb.String()
}

func isPersonOrSystem(arch *language.Architecture, id string) bool {
	for _, person := range arch.Persons {
		if person.ID == id {
			return true
		}
	}
	for _, sys := range arch.Systems {
		if sys.ID == id {
			return true
		}
	}
	return false
}

func isSystem(arch *language.Architecture, id string) bool {
	for _, sys := range arch.Systems {
		if sys.ID == id {
			return true
		}
	}
	return false
}

func systemRepID(sysID string) string { return sysID + "_ctx" }

func buildContainerSystemMap(arch *language.Architecture) map[string]string {
	m := make(map[string]string)
	for _, sys := range arch.Systems {
		for _, c := range sys.Containers {
			m[c.ID] = sys.ID
		}
		for _, d := range sys.DataStores {
			m[d.ID] = sys.ID
		}
		for _, q := range sys.Queues {
			m[q.ID] = sys.ID
		}
	}
	return m
}

func resolveOverviewNode(arch *language.Architecture, containerToSystem map[string]string, id string) (string, bool) {
	if isPersonOrSystem(arch, id) {
		return id, true
	}
	parts := strings.Split(id, ".")
	if len(parts) >= 2 {
		return parts[0], true
	}
	if sys, ok := containerToSystem[id]; ok {
		return sys, true
	}
	return "", false
}

// generateDeploymentDiagram generates a Mermaid flowchart diagram for deployment view
// Uses flowchart with subgraphs for better compatibility across Mermaid versions
func generateDeploymentDiagram(deployment *language.DeploymentNode, config MermaidConfig) string {
	var sb strings.Builder

	writeMermaidConfig(&sb, config)
	sb.WriteString(fmt.Sprintf("graph %s\n", graphDirection(config)))
	sb.WriteString("\n")

	// Recursively add deployment nodes as subgraphs and services as nodes
	addDeploymentSubgraphs(&sb, deployment, 0)

	return sb.String()
}

func addDeploymentSubgraphs(sb *strings.Builder, node *language.DeploymentNode, indent int) {
	nodeID := sanitizeNodeID(node.ID)
	label := node.Label
	if label == "" {
		label = node.ID
	}
	// Escape quotes in label if present
	label = strings.ReplaceAll(label, `"`, `\"`)

	indentStr := strings.Repeat("    ", indent)

	// Start subgraph - always show label if it exists, using format: subgraph ID[Label]
	// Use quotes for labels that contain special characters or spaces
	if strings.ContainsAny(label, " -") || label != nodeID {
		sb.WriteString(fmt.Sprintf("%ssubgraph %s[\"%s\"]\n", indentStr, nodeID, label))
	} else {
		// Simple label without quotes
		sb.WriteString(fmt.Sprintf("%ssubgraph %s[%s]\n", indentStr, nodeID, label))
	}

	// Add container instances as nodes within this subgraph
	for _, instance := range node.ContainerInstances {
		instanceID := sanitizeNodeID(instance.ContainerID)
		instanceLabel := instance.ContainerID
		if instance.InstanceID != nil {
			instanceLabel = fmt.Sprintf("%s #%s", instance.ContainerID, *instance.InstanceID)
		}
		// Escape quotes in label if present
		instanceLabel = strings.ReplaceAll(instanceLabel, `"`, `\"`)
		sb.WriteString(fmt.Sprintf("%s    %s[\"%s\"]\n", indentStr, instanceID, instanceLabel))
	}

	// Add infrastructure nodes as nodes within this subgraph
	for _, infra := range node.Infrastructure {
		infraID := sanitizeNodeID(infra.ID)
		infraLabel := infra.Label
		if infraLabel == "" {
			infraLabel = infra.ID
		}
		// Escape quotes in label if present
		infraLabel = strings.ReplaceAll(infraLabel, `"`, `\"`)
		sb.WriteString(fmt.Sprintf("%s    %s[\"%s\"]\n", indentStr, infraID, infraLabel))
	}

	// Recursively add child nodes as nested subgraphs
	for _, child := range node.Children {
		addDeploymentSubgraphs(sb, child, indent+1)
	}

	// Close subgraph
	sb.WriteString(fmt.Sprintf("%send\n", indentStr))
}

func graphDirection(config MermaidConfig) string {
	switch strings.ToUpper(config.Direction) {
	case "LR", "TB", "BT", "RL":
		return strings.ToUpper(config.Direction)
	default:
		return "LR"
	}
}

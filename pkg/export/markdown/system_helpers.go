// pkg/export/markdown/system_helpers.go
// Package markdown provides helper functions for system section rendering.
//
//nolint:gocritic // preferFprint false positive or style choice
package markdown

//nolint:gocritic // Use WriteString for consistency

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// writeSystemHeader writes the system header and description
func writeSystemHeader(sb *strings.Builder, sys *language.System) {
	sb.WriteString(fmt.Sprintf("### %s\n\n", sys.Label))
	if sys.Description != nil {
		sb.WriteString(fmt.Sprintf("%s\n\n", *sys.Description))
	}
}

// writeSystemContainerDiagram writes the container view diagram if needed
func writeSystemContainerDiagram(sb *strings.Builder, sys *language.System, arch *language.Architecture, config MermaidConfig) {
	if len(sys.Containers) == 0 && len(sys.DataStores) == 0 && len(sys.Queues) == 0 {
		return
	}
	sb.WriteString("#### Container View (C4 L2)\n\n")
	diagram := generateSystemContainerDiagram(sys, arch, config)
	sb.WriteString(fmt.Sprintf("```mermaid\n%s\n```\n\n", diagram))
}

// writeSystemComponentDiagrams writes component view diagrams for each container
func writeSystemComponentDiagrams(sb *strings.Builder, sys *language.System, arch *language.Architecture, config MermaidConfig) {
	if len(sys.Containers) == 0 {
		return
	}
	sb.WriteString("#### Component View (C4 L3)\n\n")
	for _, cont := range sys.Containers {
		if len(cont.Components) == 0 && len(cont.Relations) == 0 {
			continue
		}
		d := generateContainerComponentDiagram(cont, sys, arch, config)
		sb.WriteString(fmt.Sprintf("##### %s (C4 L3)\n\n", cont.Label))
		sb.WriteString(fmt.Sprintf("```mermaid\n%s\n```\n\n", d))
	}
}

// writeSystemElements writes containers, components, data stores, and queues
func (e *Exporter) writeSystemElements(sb *strings.Builder, sys *language.System) {
	if len(sys.Containers) > 0 {
		sb.WriteString("#### Containers\n\n")
		for _, cont := range sys.Containers {
			e.writeContainer(sb, cont)
		}
	}

	if len(sys.Components) > 0 {
		sb.WriteString("#### Components\n\n")
		for _, comp := range sys.Components {
			e.writeComponent(sb, comp)
		}
	}

	if len(sys.DataStores) > 0 {
		sb.WriteString("#### Data Stores\n\n")
		for _, ds := range sys.DataStores {
			e.writeDataStore(sb, ds)
		}
	}

	if len(sys.Queues) > 0 {
		sb.WriteString("#### Queues\n\n")
		for _, q := range sys.Queues {
			e.writeQueue(sb, q)
		}
	}
}




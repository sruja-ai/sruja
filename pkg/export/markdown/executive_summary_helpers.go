// pkg/export/markdown/executive_summary_helpers.go
// Package markdown provides helper functions for executive summary section.
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

func (e *Exporter) writeExecutiveSummary(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("## Executive Summary\n\n")

	// Overview from description
	if arch.Description != nil {
		sb.WriteString("### Overview\n\n")
		sb.WriteString(fmt.Sprintf("%s\n\n", *arch.Description))
	}

	// Key Metrics extracted from requirements and metadata
	hasMetrics := false
	sb.WriteString("### Key Metrics\n\n")

	// Extract scale from requirements
	scaleReqs := []string{}
	for _, req := range arch.Requirements {
		if req.Type != nil && *req.Type == "performance" {
			if req.Description != nil {
				scaleReqs = append(scaleReqs, *req.Description)
			}
		}
	}
	if len(scaleReqs) > 0 {
		for _, req := range scaleReqs {
			sb.WriteString(fmt.Sprintf("- %s\n", req))
			hasMetrics = true
		}
	}

	// Extract availability from requirements
	availabilityReqs := []string{}
	for _, req := range arch.Requirements {
		if req.Type != nil && *req.Type == "reliability" {
			if req.Description != nil {
				availabilityReqs = append(availabilityReqs, *req.Description)
			}
		}
	}
	if len(availabilityReqs) > 0 {
		for _, req := range availabilityReqs {
			sb.WriteString(fmt.Sprintf("- %s\n", req))
			hasMetrics = true
		}
	}

	// Extract from metadata
	if scale, ok := arch.MetaString("scale"); ok && scale != "" {
		sb.WriteString(fmt.Sprintf("- **Scale**: %s\n", scale))
		hasMetrics = true
	}
	if availability, ok := arch.MetaString("availability"); ok && availability != "" {
		sb.WriteString(fmt.Sprintf("- **Availability**: %s\n", availability))
		hasMetrics = true
	}
	if performance, ok := arch.MetaString("performance"); ok && performance != "" {
		sb.WriteString(fmt.Sprintf("- **Performance**: %s\n", performance))
		hasMetrics = true
	}
	if cost, ok := arch.MetaString("cost"); ok && cost != "" {
		sb.WriteString(fmt.Sprintf("- **Cost**: %s\n", cost))
		hasMetrics = true
	}

	if !hasMetrics {
		sb.WriteString("Not specified\n")
	}
	sb.WriteString("\n")

	// Architecture Highlights
	sb.WriteString("### Architecture Highlights\n\n")
	highlights := []string{}

	// Count systems and containers
	if len(arch.Systems) > 0 {
		totalContainers := 0
		for _, sys := range arch.Systems {
			totalContainers += len(sys.Containers)
		}
		if totalContainers > 1 {
			highlights = append(highlights, fmt.Sprintf("Microservices architecture with %d core services", totalContainers))
		}
	}

	// Check for event-driven architecture
	for _, sys := range arch.Systems {
		if len(sys.Queues) > 0 {
			highlights = append(highlights, "Event-driven communication")
			break
		}
	}

	// Check for multi-region from deployment nodes
	if len(arch.DeploymentNodes) > 1 {
		highlights = append(highlights, "Multi-region deployment")
	}

	// Extract from metadata
	if architecture, ok := arch.MetaString("architecture"); ok && architecture != "" {
		highlights = append(highlights, architecture)
	}

	if len(highlights) == 0 {
		sb.WriteString("Not specified\n")
	} else {
		for _, highlight := range highlights {
			sb.WriteString(fmt.Sprintf("- %s\n", highlight))
		}
	}
	sb.WriteString("\n")

	// Risk Summary
	sb.WriteString("### Risk Summary\n\n")
	risks := []string{}

	// Check for external dependencies
	externalCount := 0
	for _, sys := range arch.Systems {
		for _, meta := range sys.Metadata {
			if meta.Key == "tags" && meta.Value != nil {
				if strings.Contains(*meta.Value, "external") {
					externalCount++
				}
			}
		}
	}
	if externalCount > 0 {
		risks = append(risks, fmt.Sprintf("**High Risk**: %d external service dependencies", externalCount))
	}

	// Extract from metadata
	if highRisk, ok := arch.MetaString("highRisk"); ok && highRisk != "" {
		risks = append(risks, fmt.Sprintf("**High Risk**: %s", highRisk))
	}
	if mediumRisk, ok := arch.MetaString("mediumRisk"); ok && mediumRisk != "" {
		risks = append(risks, fmt.Sprintf("**Medium Risk**: %s", mediumRisk))
	}
	if lowRisk, ok := arch.MetaString("lowRisk"); ok && lowRisk != "" {
		risks = append(risks, fmt.Sprintf("**Low Risk**: %s", lowRisk))
	}

	if len(risks) == 0 {
		sb.WriteString("Not specified\n")
	} else {
		for _, risk := range risks {
			sb.WriteString(fmt.Sprintf("- %s\n", risk))
		}
	}
	sb.WriteString("\n")
}

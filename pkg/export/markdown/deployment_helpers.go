// pkg/export/markdown/deployment_helpers.go
// Package markdown provides markdown generation.
//
//nolint:gocritic // Use WriteString for consistency
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// writeDeploymentHeader writes deployment header and description
func writeDeploymentHeader(sb *strings.Builder, deployment *language.DeploymentNode) {
	sb.WriteString(fmt.Sprintf("### %s\n\n", deployment.Label))
	if deployment.Description != nil {
		sb.WriteString(fmt.Sprintf("%s\n\n", *deployment.Description))
	}
}

// writeDeploymentDiagram writes deployment diagram if needed
func writeDeploymentDiagram(sb *strings.Builder, deployment *language.DeploymentNode, config MermaidConfig) {
	if len(deployment.Children) == 0 && len(deployment.ContainerInstances) == 0 {
		return
	}
	diagram := generateDeploymentDiagram(deployment, config)
	sb.WriteString(fmt.Sprintf("```mermaid\n%s\n```\n\n", diagram))
}

// writeContainerInstances writes container instances section
func writeContainerInstances(sb *strings.Builder, instances []*language.ContainerInstance) {
	if len(instances) == 0 {
		return
	}
	sb.WriteString("#### Container Instances\n\n")
	for _, instance := range instances {
		sb.WriteString(fmt.Sprintf("- **%s**", instance.ContainerID))
		if instance.InstanceID != nil {
			sb.WriteString(fmt.Sprintf(" (Instance: %s)", *instance.InstanceID))
		}
		sb.WriteString("\n")
	}
	sb.WriteString("\n")
}




package markdown

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestWriteDeploymentHeader(t *testing.T) {
	var sb strings.Builder

	// Test with description
	desc := "Production environment"
	deployment := &language.DeploymentNode{
		Label:       "Production",
		Description: &desc,
	}
	writeDeploymentHeader(&sb, deployment)
	output := sb.String()

	if !strings.Contains(output, "### Production") {
		t.Error("Expected deployment label")
	}
	if !strings.Contains(output, "Production environment") {
		t.Error("Expected deployment description")
	}

	// Test without description
	sb.Reset()
	deployment = &language.DeploymentNode{
		Label:       "Staging",
		Description: nil,
	}
	writeDeploymentHeader(&sb, deployment)
	if !strings.Contains(sb.String(), "### Staging") {
		t.Error("Expected staging label")
	}
}

func TestWriteContainerInstances(t *testing.T) {
	var sb strings.Builder

	// Test with no instances
	writeContainerInstances(&sb, nil)
	if sb.Len() > 0 {
		t.Error("Expected empty output for nil instances")
	}

	sb.Reset()
	writeContainerInstances(&sb, []*language.ContainerInstance{})
	if sb.Len() > 0 {
		t.Error("Expected empty output for empty instances")
	}

	// Test with instances
	sb.Reset()
	instanceID := "instance-1"
	instances := []*language.ContainerInstance{
		{ContainerID: "web", InstanceID: &instanceID},
		{ContainerID: "api", InstanceID: nil},
	}
	writeContainerInstances(&sb, instances)
	output := sb.String()

	if !strings.Contains(output, "#### Container Instances") {
		t.Error("Expected container instances header")
	}
	if !strings.Contains(output, "**web**") {
		t.Error("Expected web container")
	}
	if !strings.Contains(output, "(Instance: instance-1)") {
		t.Error("Expected instance ID")
	}
	if !strings.Contains(output, "**api**") {
		t.Error("Expected api container")
	}
}

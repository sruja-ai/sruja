// pkg/kernel/diagram_parser.go
// Diagram command parsing for notebook cells

package kernel

import (
	"strings"
)

// DiagramCommand represents a parsed diagram command.
type DiagramCommand struct {
	Format     string // "mermaid", "d2", or "" (default)
	TargetType string // "system", "container", "component", or "" (all)
	TargetID   string // Specific element ID or "" (all)
}

// ParseDiagramCommand parses a diagram command from cell source.
//
// Supported formats:
//   - "diagram" - Generate diagram of entire architecture
//   - "diagram system Billing" - Generate diagram for specific system
//   - "diagram mermaid" - Generate Mermaid format
//   - "diagram d2 system Billing" - Generate D2 format for specific system
//
// Parameters:
//   - source: The diagram cell source code
//
// Returns:
//   - *DiagramCommand: Parsed command
func ParseDiagramCommand(source string) *DiagramCommand {
	cmd := &DiagramCommand{}
	parts := strings.Fields(strings.TrimSpace(source))

	// Default: generate entire architecture diagram
	if len(parts) == 0 || (len(parts) == 1 && parts[0] == "diagram") {
		return cmd
	}

	// Skip "diagram" keyword if present
	startIdx := 0
	if len(parts) > 0 && strings.ToLower(parts[0]) == "diagram" {
		startIdx = 1
	}

	if startIdx >= len(parts) {
		return cmd
	}

	// Check for format specification (mermaid, d2)
	if strings.ToLower(parts[startIdx]) == "mermaid" {
		cmd.Format = "mermaid"
		startIdx++
	} else if strings.ToLower(parts[startIdx]) == "d2" {
		cmd.Format = "d2"
		startIdx++
	}

	if startIdx >= len(parts) {
		return cmd
	}

	// Check for target type (system, container, component)
	next := strings.ToLower(parts[startIdx])
	switch next {
	case "system", "systems":
		cmd.TargetType = "system"
		startIdx++
	case "container", "containers":
		cmd.TargetType = "container"
		startIdx++
	case "component", "components":
		cmd.TargetType = "component"
		startIdx++
	}

	// Remaining part is the target ID
	if startIdx < len(parts) {
		cmd.TargetID = parts[startIdx]
	}

	return cmd
}


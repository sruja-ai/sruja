// pkg/kernel/validation_parser.go
// Validation command parsing for notebook cells

package kernel

import (
	"strings"
)

// ValidationCommand represents a parsed validation command.
type ValidationCommand struct {
	TargetType string // "system", "container", "component", "entity", "event", or "" (all)
	TargetID   string // Specific element ID or "" (all of type)
	RuleName   string // Specific rule name or "" (all rules)
}

// ParseValidationCommand parses a validation command from cell source.
//
// Supported formats:
//   - "validate" or "validate all" - Validate entire architecture
//   - "validate system Billing" - Validate specific system
//   - "validate container BillingAPI" - Validate specific container
//   - "validate component PaymentService" - Validate specific component
//   - "validate entity Payment" - Validate specific entity
//   - "validate event PaymentCompleted" - Validate specific event
//   - "validate rule UniqueIDs" - Validate using specific rule
//
// Parameters:
//   - source: The validation cell source code
//
// Returns:
//   - *ValidationCommand: Parsed command
func ParseValidationCommand(source string) *ValidationCommand {
	cmd := &ValidationCommand{}
	parts := strings.Fields(strings.TrimSpace(source))

	// Default: validate all
	if len(parts) == 0 || (len(parts) == 1 && (strings.ToLower(parts[0]) == "validate" || strings.ToLower(parts[0]) == "validate" && len(parts) == 2 && strings.ToLower(parts[1]) == "all")) {
		return cmd
	}

	// Skip "validate" keyword if present
	startIdx := 0
	if len(parts) > 0 && strings.ToLower(parts[0]) == "validate" {
		startIdx = 1
	}

	if startIdx >= len(parts) {
		return cmd
	}

	// Check for "all" keyword
	if strings.ToLower(parts[startIdx]) == "all" {
		startIdx++
		if startIdx >= len(parts) {
			return cmd
		}
	}

	// Check for "rule" keyword (specific rule validation)
	if strings.ToLower(parts[startIdx]) == "rule" {
		startIdx++
		if startIdx < len(parts) {
			cmd.RuleName = parts[startIdx]
		}
		return cmd
	}

	// Check for target type
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
	case "entity", "entities":
		cmd.TargetType = "entity"
		startIdx++
	case "event", "events":
		cmd.TargetType = "event"
		startIdx++
	}

	// Remaining part is the target ID
	if startIdx < len(parts) {
		cmd.TargetID = parts[startIdx]
	}

	return cmd
}


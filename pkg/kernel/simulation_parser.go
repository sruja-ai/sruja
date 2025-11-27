// pkg/kernel/simulation_parser.go
// Parser for simulation commands

package kernel

import (
	"fmt"
	"strings"
)

// SimulationCommand represents a parsed simulation command.
type SimulationCommand struct {
	EntityName   string
	InitialState string
	Events       []string
}

// ParseSimulationCommand parses a simulation command.
//
// Supported formats:
//   - "simulate Payment from PENDING events: PaymentAuthorized, PaymentCompleted"
//   - "simulate Payment events: PaymentAuthorized, PaymentCompleted"
//   - "simulate Payment from PENDING"
//
// Parameters:
//   - source: The simulation command string
//
// Returns:
//   - *SimulationCommand: Parsed command, or error if invalid
func ParseSimulationCommand(source string) (*SimulationCommand, error) {
	source = strings.TrimSpace(source)

	// Remove "simulate" prefix
	if !strings.HasPrefix(strings.ToLower(source), "simulate") {
		return nil, fmt.Errorf("command must start with 'simulate'")
	}

	parts := strings.Fields(source)
	if len(parts) < 2 {
		return nil, fmt.Errorf("simulation command requires entity name")
	}

	cmd := &SimulationCommand{
		EntityName: parts[1],
		Events:     []string{},
	}

	// Parse optional "from <state>"
	if len(parts) >= 4 && strings.ToLower(parts[2]) == "from" {
		cmd.InitialState = parts[3]
		parts = parts[4:] // Remove "from <state>"
	} else {
		parts = parts[2:] // Skip "simulate <entity>"
	}

	// Parse "events: <event1>, <event2>, ..." or "events <event1>, <event2>, ..."
	if len(parts) > 0 {
		eventsKeyword := strings.ToLower(parts[0])
		if eventsKeyword == "events" || strings.HasPrefix(eventsKeyword, "events:") {
			// Join remaining parts and split by comma
			eventsStr := strings.Join(parts[1:], " ")
			// Handle "events:" format
			if strings.HasPrefix(parts[0], "events:") {
				// Include the part after "events:" from the first part
				afterColon := strings.TrimPrefix(parts[0], "events:")
				eventsStr = afterColon + " " + eventsStr
			}
			eventsStr = strings.TrimSpace(eventsStr)

			eventParts := strings.Split(eventsStr, ",")
			for _, event := range eventParts {
				event = strings.TrimSpace(event)
				if event != "" {
					cmd.Events = append(cmd.Events, event)
				}
			}
		}
	}

	return cmd, nil
}

// pkg/kernel/magic.go
// Magic command parsing and execution for notebook cells

package kernel

import (
	"strings"
)

// MagicCommand represents a parsed magic command.
type MagicCommand struct {
	Command   string   // Main command (ir, snapshot, variant, validate, reset)
	SubCommand string   // Sub-command (list, create, load, apply)
	Args      []string // Command arguments
}

// ParseMagicCommand parses a magic command from cell source.
//
// Supported formats:
//   - "%ir" - Show current IR
//   - "%snapshot name" - Create snapshot
//   - "%snapshot list" - List snapshots
//   - "%snapshot load name" - Load snapshot
//   - "%variant list" - List variants
//   - "%variant create name [base]" - Create variant
//   - "%variant apply name" - Apply variant
//   - "%validate all" - Validate architecture
//   - "%reset" - Reset kernel
//
// Parameters:
//   - source: The cell source code
//
// Returns:
//   - *MagicCommand: Parsed command, or nil if not a magic command
func ParseMagicCommand(source string) *MagicCommand {
	source = strings.TrimSpace(source)
	
	// Check if it's a magic command (starts with %)
	if !strings.HasPrefix(source, "%") {
		return nil
	}

	// Remove % prefix
	commandLine := strings.TrimPrefix(source, "%")
	parts := strings.Fields(commandLine)
	
	if len(parts) == 0 {
		return nil
	}

	cmd := &MagicCommand{
		Command: parts[0],
		Args:    []string{},
	}

	if len(parts) > 1 {
		// Check if second part is a sub-command
		subCommands := []string{"list", "create", "load", "apply", "delete"}
		if contains(subCommands, strings.ToLower(parts[1])) {
			cmd.SubCommand = strings.ToLower(parts[1])
			if len(parts) > 2 {
				cmd.Args = parts[2:]
			}
		} else {
			// No sub-command, all remaining parts are args
			cmd.Args = parts[1:]
		}
	}

	return cmd
}

// contains checks if a slice contains a string.
func contains(slice []string, item string) bool {
	for _, s := range slice {
		if s == item {
			return true
		}
	}
	return false
}

// IsMagicCommand checks if the source is a magic command.
func IsMagicCommand(source string) bool {
	return ParseMagicCommand(source) != nil
}


// pkg/kernel/magic_test.go
package kernel

import (
	"strings"
	"testing"
)

func TestParseMagicCommand(t *testing.T) {
	tests := []struct {
		name        string
		source      string
		expected    *MagicCommand
		shouldParse bool
	}{
		{
			name:   "ir command",
			source: "%ir",
			expected: &MagicCommand{
				Command:    "ir",
				SubCommand: "",
				Args:       []string{},
			},
			shouldParse: true,
		},
		{
			name:   "snapshot create",
			source: "%snapshot test-snap",
			expected: &MagicCommand{
				Command:    "snapshot",
				SubCommand: "",
				Args:       []string{"test-snap"},
			},
			shouldParse: true,
		},
		{
			name:   "snapshot list",
			source: "%snapshot list",
			expected: &MagicCommand{
				Command:    "snapshot",
				SubCommand: "list",
				Args:       []string{},
			},
			shouldParse: true,
		},
		{
			name:   "variant list",
			source: "%variant list",
			expected: &MagicCommand{
				Command:    "variant",
				SubCommand: "list",
				Args:       []string{},
			},
			shouldParse: true,
		},
		{
			name:   "reset command",
			source: "%reset",
			expected: &MagicCommand{
				Command:    "reset",
				SubCommand: "",
				Args:       []string{},
			},
			shouldParse: true,
		},
		{
			name:        "not a magic command",
			source:      "regular code",
			expected:    nil,
			shouldParse: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cmd := ParseMagicCommand(tt.source)

			if !tt.shouldParse {
				if cmd != nil {
					t.Errorf("Expected nil for non-magic command, got: %+v", cmd)
				}
				return
			}

			if cmd == nil {
				t.Fatal("Expected magic command to parse, got nil")
			}

			if cmd.Command != tt.expected.Command {
				t.Errorf("Expected command %q, got %q", tt.expected.Command, cmd.Command)
			}

			if cmd.SubCommand != tt.expected.SubCommand {
				t.Errorf("Expected sub-command %q, got %q", tt.expected.SubCommand, cmd.SubCommand)
			}

			if len(cmd.Args) != len(tt.expected.Args) {
				t.Errorf("Expected %d args, got %d", len(tt.expected.Args), len(cmd.Args))
			}
		})
	}
}

func TestIsMagicCommand(t *testing.T) {
	tests := []struct {
		source string
		want   bool
	}{
		{"%ir", true},
		{"%snapshot test", true},
		{"%variant list", true},
		{"%reset", true},
		{"regular code", false},
		{"  %ir", true}, // Whitespace before % is OK after trim
		{"no % here", false},
	}

	for _, tt := range tests {
		t.Run(tt.source, func(t *testing.T) {
			got := IsMagicCommand(tt.source)
			if got != tt.want {
				t.Errorf("IsMagicCommand(%q) = %v, want %v", tt.source, got, tt.want)
			}
		})
	}
}

func TestExecuteMagicCommand(t *testing.T) {
	k, err := NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	// First, add some architecture
	dslSource := `architecture "Test System" {
  system Billing {
    container BillingAPI {}
  }
}`
	_, err = k.ExecuteCell("cell-1", CellTypeDSL, dslSource)
	if err != nil {
		t.Fatalf("Failed to execute DSL cell: %v", err)
	}

	// Test %ir command
	irCommand := `%ir`
	result, err := k.ExecuteCell("cell-2", CellTypeDSL, irCommand)
	if err != nil {
		t.Fatalf("Failed to execute %%ir command: %v", err)
	}

	if !result.Success {
		t.Errorf("%%ir command should succeed, got error: %s", result.Error)
	}

	// Check for IR output
	var irOutput *CellOutput
	for i := range result.Outputs {
		if result.Outputs[i].OutputType == "application/sruja-ir+json" {
			irOutput = &result.Outputs[i]
			break
		}
	}

	if irOutput == nil {
		t.Error("Expected IR JSON output from %ir command")
	}

	// Test %snapshot create
	snapshotCommand := `%snapshot test-snapshot-1`
	result2, err := k.ExecuteCell("cell-3", CellTypeDSL, snapshotCommand)
	if err != nil {
		t.Fatalf("Failed to execute %%snapshot command: %v", err)
	}

	if !result2.Success {
		t.Errorf("%%snapshot command should succeed, got error: %s", result2.Error)
	}

	// Test %snapshot list
	snapshotListCommand := `%snapshot list`
	result3, err := k.ExecuteCell("cell-4", CellTypeDSL, snapshotListCommand)
	if err != nil {
		t.Fatalf("Failed to execute %%snapshot list: %v", err)
	}

	if !result3.Success {
		t.Errorf("%%snapshot list should succeed, got error: %s", result3.Error)
	}

	// Check for snapshot in list
	var listOutput *CellOutput
	for i := range result3.Outputs {
		if result3.Outputs[i].OutputType == "text" {
			listOutput = &result3.Outputs[i]
			break
		}
	}

	if listOutput != nil {
		listText := listOutput.Data.(string)
		if !strings.Contains(listText, "test-snapshot-1") {
			t.Errorf("Expected snapshot list to contain 'test-snapshot-1', got: %s", listText)
		}
	}

	// Test %variant list (should be empty)
	variantListCommand := `%variant list`
	result4, err := k.ExecuteCell("cell-5", CellTypeDSL, variantListCommand)
	if err != nil {
		t.Fatalf("Failed to execute %%variant list: %v", err)
	}

	if !result4.Success {
		t.Errorf("%%variant list should succeed, got error: %s", result4.Error)
	}

	// Test %reset
	resetCommand := `%reset`
	result5, err := k.ExecuteCell("cell-6", CellTypeDSL, resetCommand)
	if err != nil {
		t.Fatalf("Failed to execute %%reset: %v", err)
	}

	if !result5.Success {
		t.Errorf("%%reset should succeed, got error: %s", result5.Error)
	}

	// Verify kernel was reset
	currentModel := k.GetModel()
	if len(currentModel.Architecture.Elements) != 0 {
		t.Error("Expected empty model after %reset")
	}
}


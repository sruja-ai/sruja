package language

import (
	"strings"
	"testing"
)

func TestPrinter_PrintChangeBlock(t *testing.T) {
	version := "1.0.0"
	status := "approved"
	req := "REQ-001"
	adr := "ADR-001"

	change := &ChangeBlock{
		ID:          "CHG-001",
		Version:     &version,
		Status:      &status,
		Requirement: &req,
		ADR:         &adr,
		Add: &ArchitectureBlock{
			Items: []ArchitectureItem{
				{System: &System{ID: "S1", Label: "System 1"}},
			},
		},
	}

	printer := NewPrinter()
	var sb strings.Builder
	printer.printChangeBlock(&sb, change)
	output := sb.String()

	if !strings.Contains(output, `change "CHG-001" {`) {
		t.Error("Should print change header")
	}
	if !strings.Contains(output, `version "1.0.0"`) {
		t.Error("Should print version")
	}
	if !strings.Contains(output, `add {`) {
		t.Error("Should print add block")
	}
	if !strings.Contains(output, `system S1 "System 1"`) {
		t.Error("Should print system in add block")
	}
}

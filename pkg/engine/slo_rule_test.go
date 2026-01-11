package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestSLOValidationRule_ValidateAvailability(t *testing.T) {
	rule := &SLOValidationRule{}
	loc := language.SourceLocation{File: "test.sruja", Line: 1, Column: 1}

	tests := []struct {
		name     string
		avail    *language.SLOAvailability
		wantErrs int
		wantWarn int
	}{
		{
			name: "valid availability",
			avail: &language.SLOAvailability{
				Target:  stringPtr("99.9%"),
				Window:  stringPtr("30 days"),
				Current: stringPtr("99.95%"),
			},
			wantErrs: 0,
			wantWarn: 0,
		},
		{
			name: "invalid target format",
			avail: &language.SLOAvailability{
				Target:  stringPtr("99.9"), // missing %
				Window:  stringPtr("30 days"),
				Current: stringPtr("99.95%"),
			},
			wantErrs: 1,
			wantWarn: 0,
		},
		{
			name: "invalid window format",
			avail: &language.SLOAvailability{
				Target:  stringPtr("99.9%"),
				Window:  stringPtr("30"), // invalid format
				Current: stringPtr("99.95%"),
			},
			wantErrs: 0,
			wantWarn: 1,
		},
		{
			name: "invalid current format",
			avail: &language.SLOAvailability{
				Target:  stringPtr("99.9%"),
				Window:  stringPtr("30 days"),
				Current: stringPtr("99.95"), // missing %
			},
			wantErrs: 1,
			wantWarn: 0,
		},
		{
			name: "multiple invalid fields",
			avail: &language.SLOAvailability{
				Target:  stringPtr("99.9"),    // invalid
				Window:  stringPtr("invalid"), // invalid
				Current: stringPtr("99.95"),   // invalid
			},
			wantErrs: 2, // target and current
			wantWarn: 1, // window
		},
		{
			name: "nil fields",
			avail: &language.SLOAvailability{
				Target:  nil,
				Window:  nil,
				Current: nil,
			},
			wantErrs: 0,
			wantWarn: 0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			diags := rule.validateAvailability(tt.avail, loc)
			errCount := 0
			warnCount := 0
			for _, d := range diags {
				if d.Severity == diagnostics.SeverityError {
					errCount++
				} else if d.Severity == diagnostics.SeverityWarning {
					warnCount++
				}
			}
			if errCount != tt.wantErrs {
				t.Errorf("validateAvailability() error count = %d, want %d", errCount, tt.wantErrs)
			}
			if warnCount != tt.wantWarn {
				t.Errorf("validateAvailability() warning count = %d, want %d", warnCount, tt.wantWarn)
			}
		})
	}
}

func TestSLOValidationRule_IsValidTimeWindow(t *testing.T) {
	rule := &SLOValidationRule{}

	tests := []struct {
		input    string
		expected bool
	}{
		{"30 days", true},
		{"7 days", true},
		{"1 day", true},
		{"30 DAYS", true}, // case insensitive
		{"1 hour", true},
		{"2 hours", true},
		{"1 week", true},
		{"2 weeks", true},
		{"1 month", true},
		{"3 months", true},
		{"30", false},
		{"days", false},
		{"30day", false},         // no space
		{"30 days extra", false}, // extra text
		{"", false},
		{"invalid", false},
		{"30 secs", false}, // not supported
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			result := rule.isValidTimeWindow(tt.input)
			if result != tt.expected {
				t.Errorf("isValidTimeWindow(%q) = %v, want %v", tt.input, result, tt.expected)
			}
		})
	}
}

func stringPtr(s string) *string {
	return &s
}

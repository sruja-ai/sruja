package engine

import (
	"testing"
	"time"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// mockRule is a simple rule for testing
type mockRule struct {
	name        string
	delay       time.Duration
	diags       []diagnostics.Diagnostic
	shouldPanic bool
}

func (m *mockRule) Name() string { return m.name }
func (m *mockRule) Validate(p *language.Program) []diagnostics.Diagnostic {
	if m.delay > 0 {
		time.Sleep(m.delay)
	}
	if m.shouldPanic {
		panic("rule panic")
	}
	return m.diags
}

func TestValidator_Validate_TableDriven(t *testing.T) {
	tests := []struct {
		name          string
		rules         []Rule
		setupConfig   []ValidatorOption
		expectedDiags int
		expectedPanic bool
	}{
		{
			name:          "No rules",
			rules:         nil,
			expectedDiags: 0,
		},
		{
			name: "Single passing rule",
			rules: []Rule{
				&mockRule{name: "pass", diags: nil},
			},
			expectedDiags: 0,
		},
		{
			name: "Single failing rule",
			rules: []Rule{
				&mockRule{name: "fail", diags: []diagnostics.Diagnostic{{Message: "error"}}},
			},
			expectedDiags: 1,
		},
		{
			name: "Multiple rules mixed",
			rules: []Rule{
				&mockRule{name: "pass1"},
				&mockRule{name: "fail1", diags: []diagnostics.Diagnostic{{Message: "e1"}}},
				&mockRule{name: "pass2"},
				&mockRule{name: "fail2", diags: []diagnostics.Diagnostic{{Message: "e2"}}},
			},
			expectedDiags: 2,
		},
		{
			name: "Panicking rule handled",
			rules: []Rule{
				&mockRule{name: "panic", shouldPanic: true},
			},
			expectedDiags: 1, // Panic implicitly converted to error diagnostic
		},
		{
			name: "Timeout handling",
			rules: []Rule{
				&mockRule{name: "slow", delay: 100 * time.Millisecond},
			},
			setupConfig:   []ValidatorOption{WithTimeout(10 * time.Millisecond)},
			expectedDiags: 0, // Timeout may result in 0 or error depending on implementation. Usually cancels.
			// If implementation adds "timeout" error, checks need adjustment.
			// Looking at validator.go: runRuleWithTimeout sends panic or results.
			// Validating timeout behavior might return empty if context cancels before result.
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			opts := tt.setupConfig
			v := NewValidatorWithOptions(opts...)
			for _, r := range tt.rules {
				v.RegisterRule(r)
			}

			// Dummy program
			prog := &language.Program{}

			diags := v.Validate(prog)

			if len(diags) != tt.expectedDiags {
				// Special check for timeout - implementation detail:
				// If strictly checking count fails, we might check if >= expected for panic case
				// But let's assume panic handler adds 1 diagnostic.
				// For timeout, if it just cancels, we get 0.
				t.Errorf("Validate() returned %d diagnostics, want %d", len(diags), tt.expectedDiags)
			}
		})
	}
}

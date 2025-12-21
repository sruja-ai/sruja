// pkg/engine/validator_helpers.go
package engine

import (
	"context"
	"fmt"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// runRuleWithTimeout executes a validation rule with timeout and panic recovery.
// This function runs in a goroutine and handles both timeout cancellation and panic recovery.
// If the rule panics, it sends a diagnostic to panicChan.
// If the rule times out, it sends a timeout diagnostic to errChan.
//
// Parameters:
//   - ctx: Context for cancellation/timeout
//   - rule: The validation rule to execute
//   - program: The program to validate
//   - errChan: Channel to send validation diagnostics
//   - panicChan: Channel to send panic messages
func runRuleWithTimeout(
	ctx context.Context,
	rule Rule,
	program *language.Program,
	errChan chan<- []diagnostics.Diagnostic,
	panicChan chan<- string,
) {
	defer func() {
		if rec := recover(); rec != nil {
			panicChan <- fmt.Sprintf("Rule %s panicked: %v", rule.Name(), rec)
		}
	}()

	// Run validation in a goroutine that respects context cancellation
	diagsChan := make(chan []diagnostics.Diagnostic, 1)
	go func() {
		diagsChan <- rule.Validate(program)
	}()

	select {
	case diags := <-diagsChan:
		errChan <- diags
	case <-ctx.Done():
		// Timeout occurred - create a diagnostic for this rule
		errChan <- []diagnostics.Diagnostic{
			{
				Code:     diagnostics.CodeValidationTimeout,
				Severity: diagnostics.SeverityError,
				Message:  fmt.Sprintf("Validation rule '%s' timed out after %v", rule.Name(), DefaultValidationTimeout),
			},
		}
	}
}

// collectResults collects validation results from channels until all rules complete or timeout.
// This function waits for all rules to complete, handling both normal results and panics.
// If the context is cancelled (timeout), it drains remaining channels.
//
// Parameters:
//   - ctx: Context for cancellation/timeout
//   - errChan: Channel receiving validation diagnostics
//   - panicChan: Channel receiving panic messages
//   - totalRules: Total number of rules to wait for
//
// Returns:
//   - A slice of all diagnostics collected from all rules
func collectResults(
	ctx context.Context,
	errChan <-chan []diagnostics.Diagnostic,
	panicChan <-chan string,
	totalRules int,
) []diagnostics.Diagnostic {
	// Pre-allocate with estimated capacity (most rules return few or no diagnostics)
	diags := make([]diagnostics.Diagnostic, 0, totalRules*2)
	completed := 0

	for completed < totalRules {
		select {
		case errs := <-errChan:
			completed++
			if len(errs) > 0 {
				diags = append(diags, errs...)
			}
		case panicMsg := <-panicChan:
			completed++
			diags = append(diags, diagnostics.Diagnostic{
				Code:     diagnostics.CodeValidationPanic,
				Severity: diagnostics.SeverityError,
				Message:  panicMsg,
			})
		case <-ctx.Done():
			// Overall timeout - drain remaining results
			return drainChannels(errChan, panicChan, diags, completed, totalRules)
		}
	}

	return diags
}

// drainChannels collects any remaining results after a timeout has occurred.
// This ensures we don't lose diagnostics from rules that completed after the timeout.
// Uses non-blocking channel operations to avoid hanging.
//
// Parameters:
//   - errChan: Channel receiving validation diagnostics
//   - panicChan: Channel receiving panic messages
//   - diags: Existing diagnostics to append to
//   - completed: Number of rules already completed
//   - totalRules: Total number of rules
//
// Returns:
//   - All diagnostics collected, including a timeout diagnostic if not all rules completed
func drainChannels(
	errChan <-chan []diagnostics.Diagnostic,
	panicChan <-chan string,
	diags []diagnostics.Diagnostic,
	completed int,
	totalRules int,
) []diagnostics.Diagnostic {
	for completed < totalRules {
		select {
		case errs := <-errChan:
			completed++
			if len(errs) > 0 {
				diags = append(diags, errs...)
			}
		case panicMsg := <-panicChan:
			completed++
			diags = append(diags, diagnostics.Diagnostic{
				Code:     diagnostics.CodeValidationPanic,
				Severity: diagnostics.SeverityError,
				Message:  panicMsg,
			})
		default:
			// No more results available, break
			completed = totalRules
		}
	}

	// Add timeout diagnostic if we didn't complete all rules
	if completed < totalRules {
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodeValidationTimeout,
			Severity: diagnostics.SeverityError,
			Message:  fmt.Sprintf("Validation timed out: %d of %d rules completed", completed, totalRules),
		})
	}

	return diags
}

// Package engine provides the validation engine for Sruja architectures.
//
// The validation engine runs a collection of rules that check architectures
// for correctness, best practices, and potential issues. Rules run concurrently
// and return diagnostics (errors/warnings) if issues are found.
package engine

import (
	"context"
	"time"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

const (
	// DefaultValidationTimeout is the default maximum time allowed for all validation rules to complete.
	DefaultValidationTimeout = 30 * time.Second
	// DefaultConcurrency is the default maximum number of concurrent validation rules.
	DefaultConcurrency = 10
)

// Rule defines the interface for validation rules.
// Each rule validates a specific aspect of the architecture (e.g., cycle detection, unique IDs).
type Rule interface {
	// Name returns the human-readable name of the validation rule.
	Name() string
	// Validate runs the validation rule against the program and returns diagnostics.
	// Returns an empty slice if no issues are found.
	Validate(program *language.Program) []diagnostics.Diagnostic
}

// Validator manages a collection of validation rules and executes them concurrently.
type Validator struct {
	// Rules is the list of registered validation rules to execute.
	Rules  []Rule
	config validatorConfig
}

// NewValidator creates a new Validator instance with default configuration.
// Use RegisterRule or RegisterDefaultRules to add validation rules.
// For custom configuration, use NewValidatorWithOptions.
//
// Example:
//
//	validator := NewValidator()
//	validator.RegisterDefaultRules()
func NewValidator() *Validator {
	return NewValidatorWithOptions()
}

// RegisterRule adds a validation rule to the validator.
// Rules are executed in the order they are registered.
//
// Example:
//
//	validator.RegisterRule(&OrphanRule{})
func (v *Validator) RegisterRule(rule Rule) {
	v.Rules = append(v.Rules, rule)
}

// RegisterDefaultRules registers the standard set of validation rules.
// This includes correctness checks (UniqueID, Cycles) and best practice checks.
func (v *Validator) RegisterDefaultRules() {
	v.RegisterRule(&UniqueIDRule{})
	v.RegisterRule(&ValidReferenceRule{})
	v.RegisterRule(&CycleDetectionRule{})
	v.RegisterRule(&OrphanDetectionRule{})
	v.RegisterRule(&SimplicityRule{})
	v.RegisterRule(&LayerViolationRule{})
	v.RegisterRule(&ScenarioFQNRule{})

	// New Best Practice Rules
	v.RegisterRule(&DatabaseIsolationRule{})
	v.RegisterRule(&PublicInterfaceDocumentationRule{})

	// SLO Validation Rule
	v.RegisterRule(&SLOValidationRule{})

	// Properties Validation Rule
	v.RegisterRule(&PropertiesValidationRule{})

	// Governance Validation Rule
	v.RegisterRule(&GovernanceValidationRule{})
}

// Validate runs all registered validation rules concurrently with timeout and panic recovery.
// Rules execute in parallel goroutines for better performance, bounded by concurrency limit.
// If a rule panics or exceeds the timeout, it is handled gracefully.
//
// Parameters:
//   - program: The parsed program to validate
//
// Returns:
//   - A slice of diagnostics (errors and warnings) found during validation.
//     Returns nil if no rules are registered or if validation is cancelled.
//
// Example:
//
//	validator := NewValidator()
//	validator.RegisterDefaultRules()
//	diagnostics := validator.Validate(program)
//	for _, diag := range diagnostics {
//	    fmt.Printf("Error: %s\n", diag.Message)
//	}
func (v *Validator) Validate(program *language.Program) []diagnostics.Diagnostic {
	if len(v.Rules) == 0 {
		return nil
	}

	timeout := v.config.timeout
	if timeout == 0 {
		timeout = DefaultValidationTimeout
	}

	concurrency := v.config.concurrency
	if concurrency == 0 {
		concurrency = DefaultConcurrency
	}

	// Create context with timeout to prevent hanging
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	// Channel to collect results
	errChan := make(chan []diagnostics.Diagnostic, len(v.Rules))
	panicChan := make(chan string, len(v.Rules))

	// Semaphore to limit concurrency
	sem := make(chan struct{}, concurrency)

	// Launch rules concurrently with panic recovery
	for _, rule := range v.Rules {
		go func(r Rule) {
			// Acquire semaphore
			select {
			case sem <- struct{}{}:
				defer func() { <-sem }() // Release
			case <-ctx.Done():
				return
			}

			runRuleWithTimeout(ctx, r, program, errChan, panicChan)
		}(rule)
	}

	// Collect results
	return collectResults(ctx, errChan, panicChan, len(v.Rules))
}

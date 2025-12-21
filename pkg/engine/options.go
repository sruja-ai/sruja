// Package engine provides functional options for configuring validators and scorers.
// This follows the Google-style functional options pattern for clean, extensible APIs.
package engine

import (
	"time"
)

// ValidatorOption is a functional option for configuring a Validator.
type ValidatorOption func(*validatorConfig)

// validatorConfig holds configuration for creating a Validator.
type validatorConfig struct {
	timeout      time.Duration
	rules        []Rule
	defaultRules bool
}

// WithTimeout sets a custom timeout for validation.
// Default is 30 seconds.
//
// Example:
//
//	validator := NewValidatorWithOptions(WithTimeout(10 * time.Second))
func WithTimeout(d time.Duration) ValidatorOption {
	return func(c *validatorConfig) {
		c.timeout = d
	}
}

// WithRules adds specific rules to the validator.
// These rules are added in addition to default rules if WithDefaultRules is also used.
//
// Example:
//
//	validator := NewValidatorWithOptions(
//	    WithRules(&CycleDetectionRule{}, &OrphanDetectionRule{}),
//	)
func WithRules(rules ...Rule) ValidatorOption {
	return func(c *validatorConfig) {
		c.rules = append(c.rules, rules...)
	}
}

// WithDefaultRules configures the validator to include all default rules.
// This is useful when you want default rules plus additional custom rules.
//
// Example:
//
//	validator := NewValidatorWithOptions(
//	    WithDefaultRules(),
//	    WithRules(&CustomRule{}),
//	)
func WithDefaultRules() ValidatorOption {
	return func(c *validatorConfig) {
		c.defaultRules = true
	}
}

// ScorerOption is a functional option for configuring a Scorer.
type ScorerOption func(*scorerConfig)

// scorerConfig holds configuration for creating a Scorer.
type scorerConfig struct {
	validatorOptions []ValidatorOption
}

// WithValidatorOptions passes options to the underlying validator.
//
// Example:
//
//	scorer := NewScorerWithOptions(
//	    WithValidatorOptions(WithTimeout(5 * time.Second)),
//	)
func WithValidatorOptions(opts ...ValidatorOption) ScorerOption {
	return func(c *scorerConfig) {
		c.validatorOptions = append(c.validatorOptions, opts...)
	}
}

// NewValidatorWithOptions creates a new Validator with the given options.
// If no options are provided, creates an empty validator (use RegisterDefaultRules to add rules).
//
// Example:
//
//	// Create validator with custom timeout and default rules
//	validator := NewValidatorWithOptions(
//	    WithTimeout(10 * time.Second),
//	    WithDefaultRules(),
//	)
//
//	// Create validator with only specific rules
//	validator := NewValidatorWithOptions(
//	    WithRules(&CycleDetectionRule{}, &ValidReferenceRule{}),
//	)
func NewValidatorWithOptions(opts ...ValidatorOption) *Validator {
	config := &validatorConfig{
		timeout: DefaultValidationTimeout,
	}

	for _, opt := range opts {
		opt(config)
	}

	v := &Validator{
		Rules: make([]Rule, 0, len(config.rules)+16),
	}

	if config.defaultRules {
		v.RegisterDefaultRules()
	}

	for _, rule := range config.rules {
		v.RegisterRule(rule)
	}

	return v
}

// NewScorerWithOptions creates a new Scorer with the given options.
//
// Example:
//
//	scorer := NewScorerWithOptions(
//	    WithValidatorOptions(WithTimeout(5 * time.Second)),
//	)
func NewScorerWithOptions(opts ...ScorerOption) *Scorer {
	config := &scorerConfig{}

	for _, opt := range opts {
		opt(config)
	}

	// Create validator with options
	v := NewValidatorWithOptions(config.validatorOptions...)

	// Register scoring rules
	v.RegisterRule(&CycleDetectionRule{})
	v.RegisterRule(&OrphanDetectionRule{})
	v.RegisterRule(&LayerViolationRule{})
	v.RegisterRule(&ValidReferenceRule{})

	return &Scorer{
		validator: v,
	}
}

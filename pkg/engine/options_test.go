package engine

import (
	"testing"
	"time"
)

func TestWithConcurrency_Default(t *testing.T) {
	validator := NewValidatorWithOptions(WithConcurrency(0))

	if validator.config.concurrency != DefaultConcurrency {
		t.Errorf("Expected default concurrency %d, got %d", DefaultConcurrency, validator.config.concurrency)
	}
}

func TestWithConcurrency_Negative(t *testing.T) {
	validator := NewValidatorWithOptions(WithConcurrency(-5))

	if validator.config.concurrency != DefaultConcurrency {
		t.Errorf("Expected default concurrency for negative value, got %d", validator.config.concurrency)
	}
}

func TestWithConcurrency_Valid(t *testing.T) {
	validator := NewValidatorWithOptions(WithConcurrency(20))

	if validator.config.concurrency != 20 {
		t.Errorf("Expected concurrency 20, got %d", validator.config.concurrency)
	}
}

func TestWithConcurrency_Maximum(t *testing.T) {
	validator := NewValidatorWithOptions(WithConcurrency(150))

	if validator.config.concurrency != 100 {
		t.Errorf("Expected max concurrency 100, got %d", validator.config.concurrency)
	}
}

func TestWithConcurrency_Boundary(t *testing.T) {
	validator := NewValidatorWithOptions(WithConcurrency(100))

	if validator.config.concurrency != 100 {
		t.Errorf("Expected concurrency 100 at boundary, got %d", validator.config.concurrency)
	}
}

func TestWithTimeout_Zero(t *testing.T) {
	validator := NewValidatorWithOptions(WithTimeout(0))

	if validator.config.timeout != DefaultValidationTimeout {
		t.Errorf("Expected default timeout for zero value, got %v", validator.config.timeout)
	}
}

func TestWithTimeout_Negative(t *testing.T) {
	validator := NewValidatorWithOptions(WithTimeout(-10 * time.Second))

	if validator.config.timeout != DefaultValidationTimeout {
		t.Errorf("Expected default timeout for negative value, got %v", validator.config.timeout)
	}
}

func TestWithTimeout_ExceedsMaximum(t *testing.T) {
	validator := NewValidatorWithOptions(WithTimeout(10 * time.Minute))

	if validator.config.timeout != 5*time.Minute {
		t.Errorf("Expected max timeout 5m for value exceeding maximum, got %v", validator.config.timeout)
	}
}

func TestWithTimeout_AtMaximum(t *testing.T) {
	validator := NewValidatorWithOptions(WithTimeout(5 * time.Minute))

	if validator.config.timeout != 5*time.Minute {
		t.Errorf("Expected timeout 5m at maximum, got %v", validator.config.timeout)
	}
}

func TestWithTimeout_Valid(t *testing.T) {
	expected := 45 * time.Second
	validator := NewValidatorWithOptions(WithTimeout(expected))

	if validator.config.timeout != expected {
		t.Errorf("Expected timeout %v, got %v", expected, validator.config.timeout)
	}
}

func TestWithValidatorOptions_Single(t *testing.T) {
	scorer := NewScorerWithOptions(
		WithValidatorOptions(WithTimeout(10 * time.Second)),
	)

	if scorer.validator.config.timeout != 10*time.Second {
		t.Errorf("Expected timeout 10s in validator, got %v", scorer.validator.config.timeout)
	}
}

func TestWithValidatorOptions_Multiple(t *testing.T) {
	scorer := NewScorerWithOptions(
		WithValidatorOptions(
			WithTimeout(20*time.Second),
			WithConcurrency(15),
		),
	)

	if scorer.validator.config.timeout != 20*time.Second {
		t.Errorf("Expected timeout 20s in validator, got %v", scorer.validator.config.timeout)
	}
	if scorer.validator.config.concurrency != 15 {
		t.Errorf("Expected concurrency 15 in validator, got %d", scorer.validator.config.concurrency)
	}
}

func TestWithValidatorOptions_Empty(t *testing.T) {
	scorer := NewScorerWithOptions()

	if scorer.validator.config.timeout != DefaultValidationTimeout {
		t.Errorf("Expected default timeout, got %v", scorer.validator.config.timeout)
	}
}

func TestWithValidatorOptions_WithRules(t *testing.T) {
	scorer := NewScorerWithOptions(
		WithValidatorOptions(
			WithRules(&LayerViolationRule{}),
		),
	)

	layerRuleCount := 0
	for _, rule := range scorer.validator.Rules {
		if _, ok := rule.(*LayerViolationRule); ok {
			layerRuleCount++
		}
	}

	if layerRuleCount == 0 {
		t.Error("Expected at least 1 LayerViolationRule")
	}
}

func TestNewValidatorWithOptions_NoOptions(t *testing.T) {
	validator := NewValidatorWithOptions()

	if validator.config.timeout != DefaultValidationTimeout {
		t.Errorf("Expected default timeout, got %v", validator.config.timeout)
	}
	if validator.config.concurrency != DefaultConcurrency {
		t.Errorf("Expected default concurrency, got %d", validator.config.concurrency)
	}
	if len(validator.Rules) != 0 {
		t.Errorf("Expected no rules without options, got %d", len(validator.Rules))
	}
}

func TestNewValidatorWithOptions_OnlyDefaultRules(t *testing.T) {
	validator := NewValidatorWithOptions(WithDefaultRules())

	if len(validator.Rules) == 0 {
		t.Error("Expected default rules to be registered")
	}
}

func TestNewValidatorWithOptions_OnlyCustomRules(t *testing.T) {
	validator := NewValidatorWithOptions(
		WithRules(&CycleDetectionRule{}),
		WithRules(&OrphanDetectionRule{}),
	)

	ruleCount := 0
	for _, rule := range validator.Rules {
		switch rule.(type) {
		case *CycleDetectionRule, *OrphanDetectionRule:
			ruleCount++
		}
	}

	if ruleCount != 2 {
		t.Errorf("Expected 2 custom rules, got %d", ruleCount)
	}
}

func TestNewValidatorWithOptions_DefaultRulesAndCustom(t *testing.T) {
	validator := NewValidatorWithOptions(
		WithDefaultRules(),
		WithRules(&CycleDetectionRule{}),
	)

	if len(validator.Rules) == 0 {
		t.Error("Expected rules")
	}

	cycleCount := 0
	for _, rule := range validator.Rules {
		if _, ok := rule.(*CycleDetectionRule); ok {
			cycleCount++
		}
	}

	if cycleCount < 1 {
		t.Error("Expected at least 1 CycleDetectionRule (could be from defaults or custom)")
	}
}

func TestNewValidatorWithOptions_AllOptions(t *testing.T) {
	validator := NewValidatorWithOptions(
		WithTimeout(25*time.Second),
		WithConcurrency(30),
		WithDefaultRules(),
		WithRules(&CycleDetectionRule{}),
	)

	if validator.config.timeout != 25*time.Second {
		t.Errorf("Expected timeout 25s, got %v", validator.config.timeout)
	}
	if validator.config.concurrency != 30 {
		t.Errorf("Expected concurrency 30, got %d", validator.config.concurrency)
	}
	if len(validator.Rules) == 0 {
		t.Error("Expected rules")
	}
}

func TestNewScorerWithOptions_Default(t *testing.T) {
	scorer := NewScorerWithOptions()

	if scorer.validator == nil {
		t.Error("Expected validator to be created")
	}
	if len(scorer.validator.Rules) == 0 {
		t.Error("Expected scoring rules to be registered")
	}
}

func TestNewScorerWithOptions_CustomValidator(t *testing.T) {
	scorer := NewScorerWithOptions(
		WithValidatorOptions(WithTimeout(15 * time.Second)),
	)

	if scorer.validator.config.timeout != 15*time.Second {
		t.Errorf("Expected timeout 15s, got %v", scorer.validator.config.timeout)
	}

	hasCycleRule := false
	for _, rule := range scorer.validator.Rules {
		if _, ok := rule.(*CycleDetectionRule); ok {
			hasCycleRule = true
			break
		}
	}

	if !hasCycleRule {
		t.Error("Expected CycleDetectionRule to be registered in scorer")
	}
}

func TestWithConcurrency_LowBoundary(t *testing.T) {
	validator := NewValidatorWithOptions(WithConcurrency(1))

	if validator.config.concurrency != 1 {
		t.Errorf("Expected concurrency 1 at lower boundary, got %d", validator.config.concurrency)
	}
}

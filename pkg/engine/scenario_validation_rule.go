package engine

import (
	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// ScenarioValidationRule validates behavioral flows (scenarios, stories, flows).
type ScenarioValidationRule struct{}

func (r *ScenarioValidationRule) Name() string {
	return "Scenario Validation"
}

func (r *ScenarioValidationRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Model == nil {
		return nil
	}

	var diags []diagnostics.Diagnostic
	runner := NewScenarioRunner(program)

	// Search for scenarios, stories, and flows in the model
	for _, item := range program.Model.Items {
		// 1. Check Scenario/Story elements
		if item.Scenario != nil {
			diags = append(diags, runner.ValidateScenario(item.Scenario.ID, item.Scenario.Title, item.Scenario.Steps)...)
		}

		// 2. Check Flow elements
		if item.Flow != nil {
			diags = append(diags, runner.ValidateScenario(item.Flow.ID, item.Flow.Title, item.Flow.Steps)...)
		}

		// 3. Check inline steps in ElementDefs (experimental)
		if item.ElementDef != nil {
			diags = append(diags, r.validateInlineSteps(item.ElementDef, runner)...)
		}
	}

	return diags
}

func (r *ScenarioValidationRule) validateInlineSteps(elem *language.ElementDef, runner *ScenarioRunner) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic
	body := elem.GetBody()
	if body == nil {
		return nil
	}

	var inlineSteps []*language.ScenarioStep
	for _, item := range body.Items {
		if item.Step != nil {
			inlineSteps = append(inlineSteps, item.Step)
		}
		if item.Element != nil {
			diags = append(diags, r.validateInlineSteps(item.Element, runner)...)
		}
	}

	if len(inlineSteps) > 0 {
		diags = append(diags, runner.ValidateScenario(elem.GetID(), nil, inlineSteps)...)
	}

	return diags
}

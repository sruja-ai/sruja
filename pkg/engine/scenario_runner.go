package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// ScenarioRunner executes and validates behavioral flows (scenarios, stories, flows).
type ScenarioRunner struct {
	program *language.Program
	// FQN -> Element mapping
	elements map[string]*language.ElementDef
}

// NewScenarioRunner creates a new ScenarioRunner for a program.
func NewScenarioRunner(prog *language.Program) *ScenarioRunner {
	runner := &ScenarioRunner{
		program: prog,
	}
	if prog != nil && prog.Model != nil {
		runner.elements, _ = collectElements(prog.Model)
	}
	return runner
}

// ValidateScenario validates a single scenario/story/flow.
func (r *ScenarioRunner) ValidateScenario(_ string, _ *string, steps []*language.ScenarioStep) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic

	for _, step := range steps {
		fromFQN := strings.Join(step.FromParts, ".")
		toFQN := strings.Join(step.ToParts, ".")

		// 1. Check Reachability (Existence)
		fromExists := r.elementExists(fromFQN)
		toExists := r.elementExists(toFQN)

		if !fromExists {
			diags = append(diags, diagnostics.Diagnostic{
				Code:     diagnostics.CodeValidationRuleError,
				Severity: diagnostics.SeverityError,
				Message:  fmt.Sprintf("Step source '%s' not found in model", fromFQN),
				Location: diagnostics.SourceLocation{
					File:   step.Pos.Filename,
					Line:   step.Pos.Line,
					Column: step.Pos.Column,
				},
			})
		}

		if !toExists {
			diags = append(diags, diagnostics.Diagnostic{
				Code:     diagnostics.CodeValidationRuleError,
				Severity: diagnostics.SeverityError,
				Message:  fmt.Sprintf("Step target '%s' not found in model", toFQN),
				Location: diagnostics.SourceLocation{
					File:   step.Pos.Filename,
					Line:   step.Pos.Line,
					Column: step.Pos.Column,
				},
			})
		}

		if !fromExists || !toExists {
			continue
		}

		// 2. Policy Enforcement (Tag-based)
		diags = append(diags, r.checkPolicies(step, fromFQN, toFQN)...)
	}

	return diags
}

func (r *ScenarioRunner) elementExists(fqn string) bool {
	_, ok := r.elements[fqn]
	return ok
}

func (r *ScenarioRunner) checkPolicies(step *language.ScenarioStep, fromFQN, toFQN string) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic

	fromElem := r.elements[fromFQN]
	toElem := r.elements[toFQN]

	if fromElem == nil || toElem == nil {
		return nil
	}

	// Get tags
	fromTags := r.getTags(fromElem)
	toTags := r.getTags(toElem)

	// Example Policy: Security Boundary (External to Database)
	if r.hasTag(fromTags, "external") && r.hasTag(toTags, "database") {
		diags = append(diags, diagnostics.Diagnostic{
			Code:     diagnostics.CodePolicyViolation,
			Severity: diagnostics.SeverityError,
			Message:  fmt.Sprintf("Security Policy Violation: External node '%s' cannot talk directly to database '%s'", fromFQN, toFQN),
			Location: diagnostics.SourceLocation{
				File:   step.Pos.Filename,
				Line:   step.Pos.Line,
				Column: step.Pos.Column,
			},
			Suggestions: []string{
				"Route this request through an API Gateway or Service layer",
				"Ensure the database is not publicly accessible",
			},
		})
	}

	return diags
}

func (r *ScenarioRunner) getTags(elem *language.ElementDef) []string {
	var tags []string

	// 1. Check TagRefs on the assignment itself (e.g., MySystem #tag)
	if elem.Assignment != nil {
		for _, tagRef := range elem.Assignment.TagRefs {
			// Remove leading #
			tags = append(tags, strings.TrimPrefix(tagRef, "#"))
		}
	}

	body := elem.GetBody()
	if body == nil {
		return tags
	}

	for _, item := range body.Items {
		// 2. Check top-level tags in body (e.g., tags ["tag1", "tag2"])
		if len(item.Tags) > 0 {
			tags = append(tags, item.Tags...)
		}

		// 3. Check metadata tags
		if item.Metadata != nil {
			for _, m := range item.Metadata.Entries {
				if (m.Key == "tags" || m.Key == "tag") && m.Value != nil {
					val := strings.Trim(*m.Value, "\"")
					tags = append(tags, strings.Split(val, ",")...)
				}
			}
		}
	}
	return tags
}

func (r *ScenarioRunner) hasTag(tags []string, target string) bool {
	for _, t := range tags {
		if strings.TrimSpace(strings.ToLower(t)) == strings.ToLower(target) {
			return true
		}
	}
	return false
}

package engine

import (
	"fmt"
	"regexp"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

type PropertiesValidationRule struct{}

func (r *PropertiesValidationRule) Name() string { return "Properties Validation" }

func (r *PropertiesValidationRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	if program == nil || program.Architecture == nil {
		return nil
	}

	// Pre-allocate diagnostics slice
	estimatedDiags := 16
	if program.Architecture != nil {
		estimatedDiags = len(program.Architecture.Systems) * 4
	}
	diags := make([]diagnostics.Diagnostic, 0, estimatedDiags)

	// Built-in validators
	validators := map[string]func(string, map[string]string) bool{
		"capacity.instanceType":       validateInstanceType,
		"capacity.readReplicas":       func(s string, _ map[string]string) bool { return isInteger(s) },
		"obs.tracing.sampleRate":      func(s string, _ map[string]string) bool { return isPercentage(s) },
		"compliance.pci.level":        func(s string, _ map[string]string) bool { return isNonEmpty(s) },
		"cost.monthly.total":          func(s string, _ map[string]string) bool { return isCurrency(s) },
		"cost.monthly.compute":        func(s string, _ map[string]string) bool { return isCurrency(s) },
		"cost.perTransaction.average": func(s string, _ map[string]string) bool { return isCurrency(s) },
	}

	arch := program.Architecture
	diags = append(diags, r.validatePropsMap(arch.Properties, arch.Location(), validators)...)

	for _, sys := range arch.Systems {
		diags = append(diags, r.validatePropsMap(sys.Properties, sys.Location(), validators)...)
		for _, cont := range sys.Containers {
			diags = append(diags, r.validatePropsMap(cont.Properties, cont.Location(), validators)...)
			for _, comp := range cont.Components {
				diags = append(diags, r.validatePropsMap(comp.Properties, comp.Location(), validators)...)
			}
			for _, ds := range cont.DataStores {
				diags = append(diags, r.validatePropsMap(ds.Properties, ds.Location(), validators)...)
			}
			for _, q := range cont.Queues {
				diags = append(diags, r.validatePropsMap(q.Properties, q.Location(), validators)...)
			}
		}
		for _, comp := range sys.Components {
			diags = append(diags, r.validatePropsMap(comp.Properties, comp.Location(), validators)...)
		}
		for _, ds := range sys.DataStores {
			diags = append(diags, r.validatePropsMap(ds.Properties, ds.Location(), validators)...)
		}
		for _, q := range sys.Queues {
			diags = append(diags, r.validatePropsMap(q.Properties, q.Location(), validators)...)
		}
	}

	for _, p := range arch.Persons {
		diags = append(diags, r.validatePropsMap(p.Properties, p.Location(), validators)...)
	}

	return diags
}

func (r *PropertiesValidationRule) validatePropsMap(props map[string]string, loc language.SourceLocation, validators map[string]func(string, map[string]string) bool) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic
	if props == nil {
		return diags
	}
	for k, v := range props {
		if validate, ok := validators[k]; ok {
			if !validate(v, props) {
				// Build enhanced error message with suggestions
				var msgSb strings.Builder
				msgSb.Grow(len(k) + len(v) + 80)
				msgSb.WriteString("Property '")
				msgSb.WriteString(k)
				msgSb.WriteString("' has invalid value '")
				msgSb.WriteString(v)
				msgSb.WriteString("'")

				var suggestions []string
				// Provide specific suggestions based on property type
				if k == "port" {
					suggestions = append(suggestions, "Port must be a valid integer between 1 and 65535")
					suggestions = append(suggestions, "Example: '8080' or '443'")
				} else if k == "version" {
					suggestions = append(suggestions, "Version should follow semantic versioning (e.g., '1.0.0', '2.1.3')")
				} else if k == "url" {
					suggestions = append(suggestions, "URL must be a valid URL format (e.g., 'https://example.com')")
				} else {
					suggestions = append(suggestions, fmt.Sprintf("Check the expected format for property '%s'", k))
					suggestions = append(suggestions, "Refer to the DSL documentation for valid property values")
				}

				diags = append(diags, diagnostics.Diagnostic{
					Code:        diagnostics.CodeInvalidProperty,
					Severity:    diagnostics.SeverityError,
					Message:     msgSb.String(),
					Location:    diagnostics.SourceLocation{File: loc.File, Line: loc.Line, Column: loc.Column},
					Suggestions: suggestions,
				})
			}
		}
	}
	return diags
}

func isInteger(s string) bool {
	matched, _ := regexp.MatchString(`^\d+$`, s)
	return matched
}

func isPercentage(s string) bool {
	matched, _ := regexp.MatchString(`^\d+(\.\d+)?%$`, s)
	return matched
}

func isCurrency(s string) bool {
	matched, _ := regexp.MatchString(`^\$\d{1,3}(,\d{3})*(\.\d+)?$`, s)
	return matched
}

func isNonEmpty(s string) bool { return s != "" }

func validateInstanceType(s string, props map[string]string) bool {
	provider := props["capacity.instanceProvider"]
	if provider == "aws" {
		matched, _ := regexp.MatchString(`^[a-z][0-9][a-z]?\.(?:nano|micro|small|medium|large|xlarge|\d+xlarge)$`, s)
		return matched
	}
	if provider == "gcp" {
		matched, _ := regexp.MatchString(`^(?:n1|n2|e2|t2d|c2|c2d|m1|m2)-(?:standard|highcpu|highmem)-(?:\d+)$`, s)
		return matched
	}
	if provider == "azure" {
		matched, _ := regexp.MatchString(`^Standard_[A-Za-z0-9]+$`, s)
		return matched
	}
	return isNonEmpty(s)
}

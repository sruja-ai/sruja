// pkg/export/markdown/template.go
// Package markdown provides template-based markdown generation.
package markdown

import (
	"bytes"
	"embed"
	"fmt"
	"strings"
	"text/template"

	"github.com/sruja-ai/sruja/pkg/language"
)

//go:embed templates/*.tmpl
var templateFS embed.FS

// TemplateData holds all data needed for markdown templates
type TemplateData struct {
	Architecture  *language.Architecture
	MermaidConfig MermaidConfig
}

// templateFuncs provides helper functions for templates
var templateFuncs = template.FuncMap{
	"escapeMermaid": escapeMermaidLabel,
	"escapeMD":      escapeMarkdown,
	"anchor":        generateAnchor,
	"hasSystems": func(arch *language.Architecture) bool {
		return len(arch.Systems) > 0
	},
	"hasPersons": func(arch *language.Architecture) bool {
		return len(arch.Persons) > 0
	},
	"hasDeployments": func(arch *language.Architecture) bool {
		return len(arch.DeploymentNodes) > 0
	},
	"hasRequirements": func(arch *language.Architecture) bool {
		return len(arch.Requirements) > 0
	},
	"hasADRs": func(arch *language.Architecture) bool {
		return len(arch.ADRs) > 0
	},
	"hasScenarios": func(arch *language.Architecture) bool {
		return len(arch.Scenarios) > 0
	},
	"hasPolicies": func(arch *language.Architecture) bool {
		return len(arch.Policies) > 0
	},
	"hasConstraints": func(arch *language.Architecture) bool {
		return len(arch.Constraints) > 0
	},
	"hasConventions": func(arch *language.Architecture) bool {
		return len(arch.Conventions) > 0
	},
	"hasFlows": func(arch *language.Architecture) bool {
		return len(arch.Flows) > 0
	},
	"hasContracts": func(arch *language.Architecture) bool {
		return len(arch.Contracts) > 0
	},

	"hasRelations": func(arch *language.Architecture) bool {
		return len(arch.Relations) > 0
	},
	"hasMetadata": func(arch *language.Architecture) bool {
		return arch.Properties != nil || len(arch.Metadata) > 0
	},
	"diagramSystem":             generateSystemDiagram,
	"diagramSystemContainer":    generateSystemContainerDiagram,
	"diagramContainerComponent": generateContainerComponentDiagram,
	"diagramScenario":           generateScenarioDiagram,
	"diagramDeployment":         generateDeploymentDiagram,
	"title": func(s string) string {
		if s == "" {
			return s
		}
		// Simple title case: uppercase first letter, lowercase rest
		// For proper title case, consider using golang.org/x/text/cases
		return strings.ToUpper(s[:1]) + strings.ToLower(s[1:])
	},
	"ptrValue": func(s *string) string {
		if s == nil {
			return ""
		}
		return *s
	},
	"ptrOr": func(s *string, def string) string {
		if s == nil {
			return def
		}
		return *s
	},
}

// loadTemplate loads and parses a template from the embedded filesystem
func loadTemplate(name string) (*template.Template, error) {
	tmplContent, err := templateFS.ReadFile("templates/" + name + ".tmpl")
	if err != nil {
		return nil, fmt.Errorf("failed to read template %s: %w", name, err)
	}

	tmpl, err := template.New(name).Funcs(templateFuncs).Parse(string(tmplContent))
	if err != nil {
		return nil, fmt.Errorf("failed to parse template %s: %w", name, err)
	}

	return tmpl, nil
}

// executeTemplate executes a template with the given data
func executeTemplate(tmpl *template.Template, data interface{}) (string, error) {
	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, data); err != nil {
		return "", fmt.Errorf("failed to execute template: %w", err)
	}
	return buf.String(), nil
}

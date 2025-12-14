// pkg/import/json/dsl_generator.go
// DSL file generation from JSON import
package json

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// generateDSLFiles generates DSL files based on output format
func generateDSLFiles(arch *language.Architecture, format OutputFormat) ([]FileOutput, error) {
	var files []FileOutput
	printer := language.NewPrinter()

	switch format {
	case OutputFormatSingleFile:
		program := &language.Program{Architecture: arch}
		content := printer.Print(program)
		files = append(files, FileOutput{
			Path:    sanitizeFileName(arch.Name) + ".sruja",
			Content: content,
		})

	case OutputFormatMultipleFiles:
		archOnly := extractArchitectureOnly(arch)
		archProgram := &language.Program{Architecture: archOnly}
		archContent := printer.Print(archProgram)
		files = append(files, FileOutput{
			Path:    "architecture.sruja",
			Content: archContent,
		})

		if len(arch.Requirements) > 0 {
			reqOnly := extractRequirementsOnly(arch)
			reqProgram := &language.Program{Architecture: reqOnly}
			reqContent := printer.Print(reqProgram)
			files = append(files, FileOutput{
				Path:    "requirements.sruja",
				Content: reqContent,
			})
		}

		if len(arch.ADRs) > 0 {
			adrOnly := extractADRsOnly(arch)
			adrProgram := &language.Program{Architecture: adrOnly}
			adrContent := printer.Print(adrProgram)
			files = append(files, FileOutput{
				Path:    "decisions.sruja",
				Content: adrContent,
			})
		}

		if len(arch.Scenarios) > 0 {
			scenOnly := extractScenariosOnly(arch)
			scenProgram := &language.Program{Architecture: scenOnly}
			scenContent := printer.Print(scenProgram)
			files = append(files, FileOutput{
				Path:    "scenarios.sruja",
				Content: scenContent,
			})
		}
	}

	return files, nil
}

// sanitizeFileName converts a name to a valid filename
func sanitizeFileName(name string) string {
	name = strings.TrimSpace(name)
	if name == "" {
		return "architecture"
	}

	result := ""
	for _, r := range name {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || r == '-' || r == '_' {
			result += string(r)
		} else if r == ' ' {
			result += "_"
		}
	}
	if result == "" {
		result = "architecture"
	}
	return result
}

// extractArchitectureOnly creates a new Architecture with only structural elements
func extractArchitectureOnly(arch *language.Architecture) *language.Architecture {
	result := &language.Architecture{Name: arch.Name}
	for i := range arch.Items {
		item := arch.Items[i]
		if item.System != nil || item.Person != nil || item.Relation != nil ||
			item.Container != nil || item.Component != nil || item.DataStore != nil ||
			item.Queue != nil || item.DeploymentNode != nil {
			result.Items = append(result.Items, item)
		}
	}
	result.PostProcess()
	return result
}

// extractRequirementsOnly creates a new Architecture with only requirements
func extractRequirementsOnly(arch *language.Architecture) *language.Architecture {
	result := &language.Architecture{Name: arch.Name + " - Requirements"}
	for i := range arch.Items {
		item := arch.Items[i]
		if item.Requirement != nil {
			result.Items = append(result.Items, item)
		}
	}
	result.PostProcess()
	return result
}

// extractADRsOnly creates a new Architecture with only ADRs
func extractADRsOnly(arch *language.Architecture) *language.Architecture {
	result := &language.Architecture{Name: arch.Name + " - Decisions"}
	for i := range arch.Items {
		item := arch.Items[i]
		if item.ADR != nil {
			result.Items = append(result.Items, item)
		}
	}
	result.PostProcess()
	return result
}

// extractScenariosOnly creates a new Architecture with only scenarios
func extractScenariosOnly(arch *language.Architecture) *language.Architecture {
	result := &language.Architecture{Name: arch.Name + " - Scenarios"}
	for i := range arch.Items {
		item := arch.Items[i]
		if item.Scenario != nil {
			result.Items = append(result.Items, item)
		}
	}
	result.PostProcess()
	return result
}

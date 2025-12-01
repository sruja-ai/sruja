// pkg/export/svg/render_governance.go
// Governance rendering (requirements, ADRs, polices, constraints, conventions) for SVG export
package svg

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
)

// renderRequirements renders the requirements layer as an independent D2 view
func (e *Exporter) renderRequirements(arch *language.Architecture) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Create a temporary architecture with only requirements
	tempArch := &language.Architecture{
		Name:         arch.Name,
		Persons:      arch.Persons,
		Systems:      arch.Systems, // Keep systems for context
		Requirements: arch.Requirements,
	}

	// Add system requirements
	for _, sys := range arch.Systems {
		if len(sys.Requirements) > 0 {
			tempSys := &language.System{
				ID:           sys.ID,
				Label:        sys.Label,
				Requirements: sys.Requirements,
			}
			// Find or create system in tempArch
			found := false
			for i, s := range tempArch.Systems {
				if s.ID == sys.ID {
					tempArch.Systems[i] = tempSys
					found = true
					break
				}
			}
			if !found {
				tempArch.Systems = append(tempArch.Systems, tempSys)
			}
		}
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export requirements: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderADRs renders the ADRs layer as an independent D2 view
func (e *Exporter) renderADRs(arch *language.Architecture) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Create a temporary architecture with only ADRs
	tempArch := &language.Architecture{
		Name:    arch.Name,
		Persons: arch.Persons,
		Systems: arch.Systems, // Keep systems for context
		ADRs:    arch.ADRs,
	}

	// Add system ADRs
	for _, sys := range arch.Systems {
		if len(sys.ADRs) > 0 {
			tempSys := &language.System{
				ID:    sys.ID,
				Label: sys.Label,
				ADRs:  sys.ADRs,
			}
			// Find or create system in tempArch
			found := false
			for i, s := range tempArch.Systems {
				if s.ID == sys.ID {
					tempArch.Systems[i] = tempSys
					found = true
					break
				}
			}
			if !found {
				tempArch.Systems = append(tempArch.Systems, tempSys)
			}
		}
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export ADRs: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderPolicies renders policies as an independent D2 view
func (e *Exporter) renderPolicies(arch *language.Architecture, _ []*language.Policy) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Create a minimal architecture with just the policies
	tempArch := &language.Architecture{
		Name: arch.Name,
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export policies: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderConstraints renders constraints as an independent D2 view
func (e *Exporter) renderConstraints(arch *language.Architecture) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Create a temporary architecture with only constraints
	tempArch := &language.Architecture{
		Name:        arch.Name,
		Constraints: arch.Constraints,
	}

	// Add system constraints
	for _, sys := range arch.Systems {
		if len(sys.Constraints) > 0 {
			tempSys := &language.System{
				ID:          sys.ID,
				Label:       sys.Label,
				Constraints: sys.Constraints,
			}
			tempArch.Systems = append(tempArch.Systems, tempSys)
		}
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export constraints: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderConventions renders conventions as an independent D2 view
func (e *Exporter) renderConventions(arch *language.Architecture) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Create a temporary architecture with only conventions
	tempArch := &language.Architecture{
		Name:        arch.Name,
		Conventions: arch.Conventions,
	}

	// Add system conventions
	for _, sys := range arch.Systems {
		if len(sys.Conventions) > 0 {
			tempSys := &language.System{
				ID:          sys.ID,
				Label:       sys.Label,
				Conventions: sys.Conventions,
			}
			tempArch.Systems = append(tempArch.Systems, tempSys)
		}
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export conventions: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

// pkg/export/svg/render_scenarios.go
// Scenario, flow, and view rendering for SVG export
package svg

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
)

// renderScenario renders a single scenario overlaid on the architecture
// It shows the architecture elements involved in the scenario and highlights them
func (e *Exporter) renderScenario(arch *language.Architecture, scenario *language.Scenario) (string, error) {
	// Extract all architecture elements referenced in the scenario
	involvedElements := e.extractScenarioElements(arch, scenario)

	// Create a filtered architecture with only involved elements
	filteredArch := e.filterArchitectureForElements(arch, involvedElements)

	// Add the scenario to show the flow
	filteredArch.Scenarios = []*language.Scenario{scenario}

	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	d2Script, err := d2Exporter.Export(filteredArch)
	if err != nil {
		return "", fmt.Errorf("failed to export scenario: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderFlow renders a single flow overlaid on the architecture
// It shows the architecture elements involved in the flow and highlights them
func (e *Exporter) renderFlow(arch *language.Architecture, sys *language.System, flow *language.Flow) (string, error) {
	// Extract all architecture elements referenced in the flow
	involvedElements := e.extractFlowElements(arch, flow)

	// Create a filtered architecture with only involved elements
	filteredArch := e.filterArchitectureForElements(arch, involvedElements)

	// Add the flow to the appropriate system
	for _, s := range filteredArch.Systems {
		if s.ID == sys.ID {
			s.Flows = []*language.Flow{flow}
			break
		}
	}
	// If system not found, add it
	found := false
	for _, s := range filteredArch.Systems {
		if s.ID == sys.ID {
			found = true
			break
		}
	}
	if !found {
		tempSys := &language.System{
			ID:    sys.ID,
			Label: sys.Label,
			Flows: []*language.Flow{flow},
		}
		filteredArch.Systems = append(filteredArch.Systems, tempSys)
	}

	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	d2Script, err := d2Exporter.Export(filteredArch)
	if err != nil {
		return "", fmt.Errorf("failed to export flow: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderView renders a custom view as an independent D2 view
func (e *Exporter) renderView(arch *language.Architecture, view *language.View) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Views are already part of architecture, just export with the view
	tempArch := &language.Architecture{
		Name:    arch.Name,
		Persons: arch.Persons,
		Systems: arch.Systems,
		Views:   []*language.View{view},
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export view %s: %w", view.ID, err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderImports renders imports as an independent D2 view
func (e *Exporter) renderImports(arch *language.Architecture) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Create architecture with imports
	tempArch := &language.Architecture{
		Name:            arch.Name,
		ResolvedImports: arch.ResolvedImports,
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export imports: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

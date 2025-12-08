// pkg/export/svg/flow.go
package svg

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// ExportFlow exports a flow as an SVG diagram showing DFD-style data flows
// Flow is an alias to Scenario - uses the same export logic
func (e *Exporter) ExportFlow(arch *language.Architecture, flow *language.Flow) (string, error) {
	// Flow is an alias to Scenario - delegate to scenario exporter
	scenario := &language.Scenario{
		ID:          flow.ID,
		Title:       flow.Title,
		Description: flow.Description,
		Steps:       flow.Steps,
	}
	return e.ExportScenario(arch, scenario)
}

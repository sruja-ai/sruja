// pkg/export/svg/render_contracts.go
// Contracts and deployment rendering for SVG export
package svg

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
)

// renderContracts renders contracts as an independent D2 view
func (e *Exporter) renderContracts(arch *language.Architecture, groupID string, contracts []*language.Contract) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Create a minimal architecture with just the contracts
	// Contracts need to be attached to their parent, so we need to reconstruct the hierarchy
	tempArch := &language.Architecture{
		Name: arch.Name,
	}

	// Parse groupID to determine where contracts belong
	switch {
	case groupID == "arch":
		tempArch.Contracts = contracts
	case strings.HasPrefix(groupID, "system-"):
		sysID := strings.TrimPrefix(groupID, "system-")
		for _, sys := range arch.Systems {
			if sys.ID == sysID {
				tempSys := &language.System{
					ID:        sys.ID,
					Label:     sys.Label,
					Contracts: contracts,
				}
				tempArch.Systems = []*language.System{tempSys}
				break
			}
		}
	case strings.HasPrefix(groupID, "container-"):
		// Format: container-{sysID}-{contID}
		parts := strings.Split(strings.TrimPrefix(groupID, "container-"), "-")
		if len(parts) >= 2 {
			sysID := parts[0]
			contID := strings.Join(parts[1:], "-")
			for _, sys := range arch.Systems {
				if sys.ID == sysID {
					for _, cont := range sys.Containers {
						if cont.ID == contID {
							tempCont := &language.Container{
								ID:        cont.ID,
								Label:     cont.Label,
								Contracts: contracts,
							}
							tempSys := &language.System{
								ID:         sys.ID,
								Label:      sys.Label,
								Containers: []*language.Container{tempCont},
							}
							tempArch.Systems = []*language.System{tempSys}
							break
						}
					}
					break
				}
			}
		}
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export contracts: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderDeployment renders deployment nodes as an independent D2 view
func (e *Exporter) renderDeployment(arch *language.Architecture, deployment *language.DeploymentNode) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	tempArch := &language.Architecture{
		Name:            arch.Name,
		DeploymentNodes: []*language.DeploymentNode{deployment},
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export deployment: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderSharedArtifacts renders shared artifacts as an independent D2 view
func (e *Exporter) renderSharedArtifacts(arch *language.Architecture) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	tempArch := &language.Architecture{
		Name:            arch.Name,
		SharedArtifacts: arch.SharedArtifacts,
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export shared artifacts: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderLibraries renders libraries as an independent D2 view
func (e *Exporter) renderLibraries(arch *language.Architecture) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	tempArch := &language.Architecture{
		Name:      arch.Name,
		Libraries: arch.Libraries,
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export libraries: %w", err)
	}

	return e.compileAndRenderD2(d2Script)
}

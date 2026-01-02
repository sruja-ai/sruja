package mermaid

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// Config represents Mermaid diagram configuration.
type Config struct {
	Layout         string
	Theme          string
	Look           string
	Direction      string
	UseFrontmatter bool
	ViewLevel      int    // 1=Context, 2=Container, 3=Component
	TargetID       string // ID of the System (for L2) or Container (for L3) to focus on
}

// DefaultConfig returns the default Mermaid configuration.
func DefaultConfig() Config {
	return Config{
		Layout:    "elk",
		Theme:     "default",
		Direction: "LR",
		ViewLevel: 1,
	}
}

// Exporter handles Mermaid diagram generation.
type Exporter struct {
	Config Config
}

// NewExporter creates a new Mermaid exporter.
func NewExporter(config Config) *Exporter {
	return &Exporter{Config: config}
}

// Export generates a Mermaid diagram from a program.
func (e *Exporter) Export(prog *language.Program) string {
	if prog == nil || prog.Model == nil {
		return ""
	}

	// Dispatch based on view level
	switch e.Config.ViewLevel {
	case 2:
		return e.exportL2(prog)
	case 3:
		return e.exportL3(prog)
	default:
		return e.GenerateL1(prog)
	}
}

func (e *Exporter) exportL2(prog *language.Program) string {
	systems := extractSystemsFromModel(prog)
	var targetSys *language.System
	for _, sys := range systems {
		if sys.ID == e.Config.TargetID {
			targetSys = sys
			break
		}
	}

	// If no specific target found but L2 requested, maybe pick the first one?
	// Or strictly return empty? For now, let's allow finding by ID.
	// If ID is empty/not found, we can't generate L2 for "nothing".
	if targetSys == nil {
		return ""
	}

	return e.GenerateL2(targetSys, prog)
}

func (e *Exporter) exportL3(prog *language.Program) string {
	// Need to find the container. Containers are nested in Systems.
	// We need both the container and its parent system ID for GenerateL3 context.
	systems := extractSystemsFromModel(prog)
	var targetCont *language.Container
	var parentSysID string

	found := false
	for _, sys := range systems {
		for _, cont := range sys.Containers {
			// Container IDs might be full IDs or relative.
			// extractSystemsFromModel returns Systems with nested Containers properly populated.
			// Check full ID or relative ID match
			if cont.ID == e.Config.TargetID || sys.ID+"."+cont.ID == e.Config.TargetID {
				targetCont = cont
				parentSysID = sys.ID
				found = true
				break
			}
		}
		if found {
			break
		}
	}

	// Also check top-level containers just in case
	if !found {
		containers := extractTopLevelContainers(prog)
		for _, cont := range containers {
			if cont.ID == e.Config.TargetID {
				targetCont = cont
				found = true
				break
			}
		}
	}

	if targetCont == nil {
		return ""
	}

	return e.GenerateL3(targetCont, parentSysID, prog)
}

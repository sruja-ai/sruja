// pkg/export/svg/helpers.go
// Helper functions for SVG export
package svg

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

func (e *Exporter) nextDataIDString() string {
	id := fmt.Sprintf("data-%d", e.nextDataID)
	e.nextDataID++
	return id
}

func (e *Exporter) buildElementPaths(arch *language.Architecture) {
	for _, p := range arch.Persons {
		e.elementPaths[p.ID] = p.ID
	}
	for _, s := range arch.Systems {
		e.elementPaths[s.ID] = s.ID
		for _, c := range s.Containers {
			e.elementPaths[c.ID] = fmt.Sprintf("%s.%s", s.ID, c.ID)
			for _, comp := range c.Components {
				e.elementPaths[comp.ID] = fmt.Sprintf("%s.%s.%s", s.ID, c.ID, comp.ID)
			}
		}
		for _, comp := range s.Components {
			e.elementPaths[comp.ID] = fmt.Sprintf("%s.%s", s.ID, comp.ID)
		}
		for _, d := range s.DataStores {
			e.elementPaths[d.ID] = fmt.Sprintf("%s.%s", s.ID, d.ID)
		}
	}
}

func (e *Exporter) hasSystemRequirements(arch *language.Architecture) bool {
	for _, sys := range arch.Systems {
		if len(sys.Requirements) > 0 {
			return true
		}
	}
	return false
}

func (e *Exporter) hasSystemADRs(arch *language.Architecture) bool {
	for _, sys := range arch.Systems {
		if len(sys.ADRs) > 0 {
			return true
		}
	}
	return false
}

func (e *Exporter) hasTechnology(arch *language.Architecture) bool {
	for _, sys := range arch.Systems {
		for _, cont := range sys.Containers {
			for _, comp := range cont.Components {
				if comp.Technology != nil {
					return true
				}
			}
		}
		for _, ds := range sys.DataStores {
			if ds.Technology != nil {
				return true
			}
		}
		for _, q := range sys.Queues {
			if q.Technology != nil {
				return true
			}
		}
	}
	return false
}

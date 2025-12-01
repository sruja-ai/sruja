// pkg/export/svg/data.go
// Data store generation for documentation content
package svg

import (
	"fmt"
	"html"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

const defaultRequirementType = "Requirement"

func (e *Exporter) writeDataStore(sb *strings.Builder, arch *language.Architecture) {
	// Overview
	dataID := e.nextDataIDString()
	fmt.Fprintf(sb, "        <g id=\"%s\" style=\"display:none;\">\n", dataID)
	sb.WriteString("            <text x=\"0\" y=\"0\" class=\"section-header\">Architecture Overview</text>\n")
	fmt.Fprintf(sb, "            <text x=\"0\" y=\"40\" class=\"subtitle\">%s</text>\n", html.EscapeString(arch.Name))
	if arch.Description != nil {
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"80\" class=\"doc-text\">%s</text>\n", html.EscapeString(*arch.Description))
	}
	sb.WriteString("        </g>\n")

	// Persons
	for _, person := range arch.Persons {
		e.writePersonData(sb, person)
	}

	// Systems
	for _, sys := range arch.Systems {
		e.writeSystemData(sb, sys, arch)
	}

	// Requirements Summary
	if len(arch.Requirements) > 0 || e.hasSystemRequirements(arch) {
		e.writeRequirementsSummary(sb, arch)
	}

	// ADRs Summary
	if len(arch.ADRs) > 0 || e.hasSystemADRs(arch) {
		e.writeADRsSummary(sb, arch)
	}

	// Technology Summary
	if e.hasTechnology(arch) {
		e.writeTechnologySummary(sb, arch)
	}
}

func (e *Exporter) writePersonData(sb *strings.Builder, person *language.Person) {
	dataID := e.nextDataIDString()
	fmt.Fprintf(sb, "        <g id=\"%s\" style=\"display:none;\">\n", dataID)
	sb.WriteString("            <text x=\"0\" y=\"0\" class=\"section-header\">Person</text>\n")
	fmt.Fprintf(sb, "            <text x=\"0\" y=\"40\" class=\"subtitle\">%s</text>\n", html.EscapeString(person.Label))
	if person.Description != nil {
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"80\" class=\"doc-text\">%s</text>\n", html.EscapeString(*person.Description))
	}
	sb.WriteString("        </g>\n")
	e.elementPaths[person.ID+"_data"] = dataID
}

func (e *Exporter) writeSystemData(sb *strings.Builder, sys *language.System, arch *language.Architecture) {
	dataID := e.nextDataIDString()
	fmt.Fprintf(sb, "        <g id=\"%s\" style=\"display:none;\">\n", dataID)
	sb.WriteString("            <text x=\"0\" y=\"0\" class=\"section-header\">System</text>\n")
	fmt.Fprintf(sb, "            <text x=\"0\" y=\"40\" class=\"subtitle\">%s</text>\n", html.EscapeString(sys.Label))
	if sys.Description != nil {
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"80\" class=\"doc-text\">%s</text>\n", html.EscapeString(*sys.Description))
	}

	y := 120

	// Requirements
	if len(sys.Requirements) > 0 {
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"section-header\">Requirements</text>\n", y)
		y += 40
		for _, req := range sys.Requirements {
			reqType := defaultRequirementType
			if req.Type != nil {
				reqType = *req.Type
			}
			desc := ""
			if req.Description != nil {
				desc = *req.Description
			}
			fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-list-item\">- %s (%s): %s</text>\n", y, req.ID, reqType, html.EscapeString(desc))
			y += 25
		}
	}

	// ADRs
	if len(sys.ADRs) > 0 {
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"section-header\">Architectural Decisions</text>\n", y)
		y += 40
		for _, adr := range sys.ADRs {
			title := adr.ID
			if adr.Title != nil {
				title = *adr.Title
			}
			fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">%s</text>\n", y, html.EscapeString(title))
			y += 30
			if adr.Body != nil {
				if adr.Body.Status != nil {
					fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">Status: %s</text>\n", y, html.EscapeString(*adr.Body.Status))
					y += 30
				}
				if adr.Body.Decision != nil {
					fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">Decision: %s</text>\n", y, html.EscapeString(*adr.Body.Decision))
					y += 30
				}
			}
		}
	}

	sb.WriteString("        </g>\n")
	e.elementPaths[sys.ID+"_data"] = dataID

	// Containers
	for _, cont := range sys.Containers {
		e.writeContainerData(sb, cont, sys.ID)
	}

	// Components (direct in system)
	for _, comp := range sys.Components {
		e.writeComponentData(sb, comp, sys.ID)
	}
}

func (e *Exporter) writeContainerData(sb *strings.Builder, cont *language.Container, _ string) {
	dataID := e.nextDataIDString()
	fmt.Fprintf(sb, "        <g id=\"%s\" style=\"display:none;\">\n", dataID)
	sb.WriteString("            <text x=\"0\" y=\"0\" class=\"section-header\">Container</text>\n")
	fmt.Fprintf(sb, "            <text x=\"0\" y=\"40\" class=\"subtitle\">%s</text>\n", html.EscapeString(cont.Label))
	if cont.Description != nil {
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"80\" class=\"doc-text\">%s</text>\n", html.EscapeString(*cont.Description))
	}

	y := 120
	if len(cont.Requirements) > 0 {
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"section-header\">Requirements</text>\n", y)
		y += 40
		for _, req := range cont.Requirements {
			reqType := defaultRequirementType
			if req.Type != nil {
				reqType = *req.Type
			}
			desc := ""
			if req.Description != nil {
				desc = *req.Description
			}
			fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-list-item\">- %s (%s): %s</text>\n", y, req.ID, reqType, html.EscapeString(desc))
			y += 25
		}
	}

	sb.WriteString("        </g>\n")
	e.elementPaths[cont.ID+"_data"] = dataID

	// Components in container
	for _, comp := range cont.Components {
		e.writeComponentData(sb, comp, cont.ID)
	}
}

func (e *Exporter) writeComponentData(sb *strings.Builder, comp *language.Component, _ string) {
	dataID := e.nextDataIDString()
	fmt.Fprintf(sb, "        <g id=\"%s\" style=\"display:none;\">\n", dataID)
	sb.WriteString("            <text x=\"0\" y=\"0\" class=\"section-header\">Component</text>\n")
	fmt.Fprintf(sb, "            <text x=\"0\" y=\"40\" class=\"subtitle\">%s</text>\n", html.EscapeString(comp.Label))
	if comp.Description != nil {
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"80\" class=\"doc-text\">%s</text>\n", html.EscapeString(*comp.Description))
	}

	if comp.Technology != nil {
		sb.WriteString("            <text x=\"0\" y=\"120\" class=\"section-header\">Technology</text>\n")
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"160\" class=\"doc-list-item\">- %s</text>\n", html.EscapeString(*comp.Technology))
	}

	sb.WriteString("        </g>\n")
	e.elementPaths[comp.ID+"_data"] = dataID
}

func (e *Exporter) writeRequirementsSummary(sb *strings.Builder, arch *language.Architecture) {
	dataID := e.nextDataIDString()
	fmt.Fprintf(sb, "        <g id=\"%s\" style=\"display:none;\">\n", dataID)
	sb.WriteString("            <text x=\"0\" y=\"0\" class=\"section-header\">All Requirements</text>\n")
	sb.WriteString("            <text x=\"0\" y=\"40\" class=\"subtitle\">Functional, Constraint, and Performance Requirements</text>\n")

	y := 80
	for _, req := range arch.Requirements {
		reqType := "Requirement"
		if req.Type != nil {
			reqType = *req.Type
		}
		desc := ""
		if req.Description != nil {
			desc = *req.Description
		}
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">%s (%s): %s</text>\n", y, req.ID, reqType, html.EscapeString(desc))
		y += 30
	}

	for _, sys := range arch.Systems {
		for _, req := range sys.Requirements {
			reqType := defaultRequirementType
			if req.Type != nil {
				reqType = *req.Type
			}
			desc := ""
			if req.Description != nil {
				desc = *req.Description
			}
			fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">%s.%s (%s): %s</text>\n", y, sys.ID, req.ID, reqType, html.EscapeString(desc))
			y += 30
		}
	}

	sb.WriteString("        </g>\n")
	e.elementPaths["requirements_summary"] = dataID
}

func (e *Exporter) writeADRsSummary(sb *strings.Builder, arch *language.Architecture) {
	dataID := e.nextDataIDString()
	fmt.Fprintf(sb, "        <g id=\"%s\" style=\"display:none;\">\n", dataID)
	sb.WriteString("            <text x=\"0\" y=\"0\" class=\"section-header\">All Architectural Decision Records</text>\n")
	sb.WriteString("            <text x=\"0\" y=\"40\" class=\"subtitle\">Key architectural decisions</text>\n")

	y := 80
	for _, adr := range arch.ADRs {
		title := adr.ID
		if adr.Title != nil {
			title = *adr.Title
		}
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"section-header\">%s</text>\n", y, html.EscapeString(adr.ID))
		y += 40
		fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">Title: %s</text>\n", y, html.EscapeString(title))
		y += 30
		if adr.Body != nil {
			if adr.Body.Status != nil {
				fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">Status: %s</text>\n", y, html.EscapeString(*adr.Body.Status))
				y += 30
			}
			if adr.Body.Decision != nil {
				fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">Decision: %s</text>\n", y, html.EscapeString(*adr.Body.Decision))
				y += 30
			}
		}
		y += 20
	}

	for _, sys := range arch.Systems {
		for _, adr := range sys.ADRs {
			title := adr.ID
			if adr.Title != nil {
				title = *adr.Title
			}
			fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"section-header\">%s</text>\n", y, html.EscapeString(adr.ID))
			y += 40
			fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">Title: %s</text>\n", y, html.EscapeString(title))
			y += 30
			if adr.Body != nil {
				if adr.Body.Status != nil {
					fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">Status: %s</text>\n", y, html.EscapeString(*adr.Body.Status))
					y += 30
				}
				if adr.Body.Decision != nil {
					fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-text\">Decision: %s</text>\n", y, html.EscapeString(*adr.Body.Decision))
					y += 30
				}
			}
			y += 20
		}
	}

	sb.WriteString("        </g>\n")
	e.elementPaths["adrs_summary"] = dataID
}

func (e *Exporter) writeTechnologySummary(sb *strings.Builder, arch *language.Architecture) {
	dataID := e.nextDataIDString()
	fmt.Fprintf(sb, "        <g id=\"%s\" style=\"display:none;\">\n", dataID)
	sb.WriteString("            <text x=\"0\" y=\"0\" class=\"section-header\">Technology Stack</text>\n")
	sb.WriteString("            <text x=\"0\" y=\"40\" class=\"subtitle\">Technologies used in the architecture</text>\n")

	y := 80
	techMap := make(map[string]bool)
	for _, sys := range arch.Systems {
		for _, cont := range sys.Containers {
			for _, comp := range cont.Components {
				if comp.Technology != nil && !techMap[*comp.Technology] {
					fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-list-item\">- %s (Component: %s)</text>\n", y, html.EscapeString(*comp.Technology), html.EscapeString(comp.Label))
					techMap[*comp.Technology] = true
					y += 25
				}
			}
		}
		for _, ds := range sys.DataStores {
			if ds.Technology != nil && !techMap[*ds.Technology] {
				fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-list-item\">- %s (DataStore: %s)</text>\n", y, html.EscapeString(*ds.Technology), html.EscapeString(ds.Label))
				techMap[*ds.Technology] = true
				y += 25
			}
		}
		for _, q := range sys.Queues {
			if q.Technology != nil && !techMap[*q.Technology] {
				fmt.Fprintf(sb, "            <text x=\"0\" y=\"%d\" class=\"doc-list-item\">- %s (Queue: %s)</text>\n", y, html.EscapeString(*q.Technology), html.EscapeString(q.Label))
				techMap[*q.Technology] = true
				y += 25
			}
		}
	}

	sb.WriteString("        </g>\n")
	e.elementPaths["technology_summary"] = dataID
}

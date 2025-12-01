package svg

import (
	"encoding/xml"
	"fmt"
	"html"
	"regexp"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// postProcessSVG injects interactivity, styles, and scripts into the D2-generated SVGs and stitches them
func (e *Exporter) postProcessSVG(views map[string]string, arch *language.Architecture) string {
	var sb strings.Builder

	// SVG Header (Standard wrapper)
	sb.WriteString(`<?xml version="1.0" encoding="UTF-8" standalone="no"?>`)
	sb.WriteString("\n<svg\n")
	sb.WriteString(`    xmlns="http://www.w3.org/2000/svg"` + "\n")
	sb.WriteString(`    xmlns:xlink="http://www.w3.org/1999/xlink"` + "\n")
	sb.WriteString(`    version="1.1"` + "\n")
	sb.WriteString(`    width="100%"` + "\n")
	sb.WriteString(`    height="100%"` + "\n")
	sb.WriteString(`    viewBox="0 0 2400 1600">` + "\n") // Fixed viewBox for now, D2 might give different ones
	// Note: D2 SVGs have their own viewBox. We might need to extract it or wrap them in nested SVGs.
	// For simplicity, we'll wrap each D2 output in a <g> and let D2's SVG content sit there.
	// But D2 outputs a full <svg>. We need to strip the <svg> tags and keep the content (defs + g).

	// Inject Styles
	e.writeStyles(&sb)

	// Data Store
	sb.WriteString("    <defs>\n")
	e.writeDataStore(&sb, arch)
	sb.WriteString("    </defs>\n\n")

	// Build a map of system IDs and container IDs to determine view levels
	systemIDs := make(map[string]bool)
	containerIDs := make(map[string]bool)
	scenarioIDs := make(map[string]bool)
	flowIDs := make(map[string]bool)
	domainIDs := make(map[string]bool)
	contextIDs := make(map[string]bool)
	contractIDs := make(map[string]bool)
	deploymentIDs := make(map[string]bool)
	aggregateIDs := make(map[string]bool)
	entityIDs := make(map[string]bool)
	valueObjectIDs := make(map[string]bool)
	eventIDs := make(map[string]bool)
	viewDefIDs := make(map[string]bool)
	specialViewIDs := make(map[string]bool) // requirements, adrs, shared-artifacts, libraries, policies, constraints, conventions, imports, etc.

	for _, sys := range arch.Systems {
		systemIDs[sys.ID] = true
		for _, cont := range sys.Containers {
			containerIDs[cont.ID] = true
		}
		for _, flow := range sys.Flows {
			flowIDs[fmt.Sprintf("flow-%s-%s", sys.ID, sanitizeID(flow.Title))] = true
		}
	}
	for _, scenario := range arch.Scenarios {
		scenarioIDs[fmt.Sprintf("scenario-%s", sanitizeID(scenario.Title))] = true
	}
	for _, domain := range arch.Domains {
		domainIDs[fmt.Sprintf("domain-%s", domain.ID)] = true
		for _, ctx := range domain.Contexts {
			contextIDs[fmt.Sprintf("context-%s-%s", domain.ID, ctx.ID)] = true
			for _, agg := range ctx.Aggregates {
				aggregateIDs[fmt.Sprintf("aggregate-%s-%s-%s", domain.ID, ctx.ID, agg.ID)] = true
			}
			for _, entity := range ctx.Entities {
				// Check if entity is in an aggregate
				inAggregate := false
				for _, agg := range ctx.Aggregates {
					for _, e := range agg.Entities {
						if e.ID == entity.ID {
							inAggregate = true
							break
						}
					}
					if inAggregate {
						break
					}
				}
				if !inAggregate {
					entityIDs[fmt.Sprintf("entity-%s-%s-%s", domain.ID, ctx.ID, entity.ID)] = true
				}
			}
			for _, vo := range ctx.ValueObjects {
				// Check if value object is in an aggregate
				inAggregate := false
				for _, agg := range ctx.Aggregates {
					for _, v := range agg.ValueObjects {
						if v.ID == vo.ID {
							inAggregate = true
							break
						}
					}
					if inAggregate {
						break
					}
				}
				if !inAggregate {
					valueObjectIDs[fmt.Sprintf("valueobject-%s-%s-%s", domain.ID, ctx.ID, vo.ID)] = true
				}
			}
			for _, event := range ctx.Events {
				eventIDs[fmt.Sprintf("event-%s-%s-%s", domain.ID, ctx.ID, event.ID)] = true
			}
		}
	}
	for _, view := range arch.Views {
		viewDefIDs[fmt.Sprintf("view-%s", view.ID)] = true
	}
	for _, deployment := range arch.DeploymentNodes {
		deploymentIDs[fmt.Sprintf("deployment-%s", deployment.ID)] = true
	}
	// Mark contract views and other special views
	for id := range views {
		if strings.HasPrefix(id, "contracts-") {
			contractIDs[id] = true
		} else if id == "shared-artifacts" || id == "libraries" || id == "policies" || id == "constraints" || id == "conventions" || id == "imports" {
			specialViewIDs[id] = true
		}
	}

	// Mark special independent views
	if _, ok := views["requirements"]; ok {
		specialViewIDs["requirements"] = true
	}
	if _, ok := views["adrs"]; ok {
		specialViewIDs["adrs"] = true
	}

	// Main Diagram Area
	sb.WriteString("    <g id=\"diagramArea\">\n")

	// 1. Level 1 (Always present)
	if l1Content, ok := views["level1"]; ok {
		processedContent, viewBox := e.processViewContent(l1Content, views)
		// Wrap in nested SVG to preserve viewBox and coordinate system
		if viewBox != "" {
			fmt.Fprintf(&sb, "        <g id=\"level1\" data-level=\"1\" style=\"display:block;\">\n")
			fmt.Fprintf(&sb, "            <svg viewBox=\"%s\" width=\"100%%\" height=\"100%%\" preserveAspectRatio=\"xMidYMid meet\">\n", viewBox)
			sb.WriteString(processedContent)
			sb.WriteString("            </svg>\n")
			sb.WriteString("        </g>\n")
		} else {
			sb.WriteString("        <g id=\"level1\" data-level=\"1\" style=\"display:block;\">\n")
			sb.WriteString(processedContent)
			sb.WriteString("        </g>\n")
		}
	}

	// 2. Other Views (Hidden by default) - each independent diagram component
	for id, content := range views {
		if id == "level1" {
			continue
		}
		// Determine level and view type for each independent diagram
		level := 2
		if containerIDs[id] {
			level = 3
		} else if scenarioIDs[id] || flowIDs[id] || domainIDs[id] || contextIDs[id] || contractIDs[id] || deploymentIDs[id] || aggregateIDs[id] || entityIDs[id] || valueObjectIDs[id] || eventIDs[id] || viewDefIDs[id] || specialViewIDs[id] {
			level = 0 // Independent views (scenarios, flows, domains, contexts, contracts, deployments, aggregates, entities, valueobjects, events, views, requirements, ADRs, etc.)
		}
		processedContent, viewBox := e.processViewContent(content, views)
		// Wrap in nested SVG to preserve viewBox and coordinate system
		if viewBox != "" {
			fmt.Fprintf(&sb, "        <g id=\"view-%s\" data-level=\"%d\" data-view-type=\"%s\" style=\"display:none;\">\n", id, level, e.getViewType(id, scenarioIDs, flowIDs, domainIDs, contextIDs, contractIDs, deploymentIDs, aggregateIDs, entityIDs, valueObjectIDs, eventIDs, viewDefIDs, specialViewIDs))
			fmt.Fprintf(&sb, "            <svg viewBox=\"%s\" width=\"100%%\" height=\"100%%\" preserveAspectRatio=\"xMidYMid meet\">\n", viewBox)
			sb.WriteString(processedContent)
			sb.WriteString("            </svg>\n")
			sb.WriteString("        </g>\n")
		} else {
			fmt.Fprintf(&sb, "        <g id=\"view-%s\" data-level=\"%d\" data-view-type=\"%s\" style=\"display:none;\">\n", id, level, e.getViewType(id, scenarioIDs, flowIDs, domainIDs, contextIDs, contractIDs, deploymentIDs, aggregateIDs, entityIDs, valueObjectIDs, eventIDs, viewDefIDs, specialViewIDs))
			sb.WriteString(processedContent)
			sb.WriteString("        </g>\n")
		}
	}

	sb.WriteString("    </g>\n\n")

	// UI Controls
	e.writeHeader(&sb, arch)
	e.writeLevelButtons(&sb) // These might need updating to handle drill-down navigation
	e.writeFilterButtons(&sb)
	e.writeDocPanel(&sb, arch)

	// JavaScript
	e.writeJavaScript(&sb)

	sb.WriteString("</svg>")
	return sb.String()
}

// extractViewBox extracts the viewBox attribute from an SVG string
func (e *Exporter) extractViewBox(content string) string {
	// Look for viewBox in the SVG tag
	viewBoxRe := regexp.MustCompile(`(?i)viewBox\s*=\s*["']([^"']+)["']`)
	if matches := viewBoxRe.FindStringSubmatch(content); matches != nil {
		return matches[1]
	}
	return ""
}

// processViewContent strips <svg> tags and injects interactivity into a single view's content
// Returns the processed content and the extracted viewBox (if any)
func (e *Exporter) processViewContent(content string, views map[string]string) (string, string) {
	// Extract viewBox BEFORE processing (D2 generates viewBox in the SVG tag)
	// We need to extract it before we strip the SVG tags
	viewBox := e.extractViewBox(content)

	// 1. Strip <svg> tags using XML parser (more robust than regex)
	// D2 generates nested <svg> tags, so we need to remove all
	// First, remove any XML declarations (<?xml ...?>) as they can only appear at document start
	// Use a simple but effective approach: find and remove all XML declarations
	// Match: <?xml ... ?> where ... can be any characters including newlines
	for {
		// Find the start of an XML declaration
		start := strings.Index(strings.ToLower(content), "<?xml")
		if start == -1 {
			break // No more XML declarations
		}
		// Find the closing ?>
		end := strings.Index(content[start:], "?>")
		if end == -1 {
			break // Malformed, stop
		}
		// Remove the XML declaration (including the ?>)
		content = content[:start] + content[start+end+2:]
	}

	// Wrap content in a temporary root to handle partial XML fragments
	wrappedContent := "<root>" + content + "</root>"
	decoder := xml.NewDecoder(strings.NewReader(wrappedContent))
	decoder.Strict = false // Allow malformed XML

	var result strings.Builder
	svgDepth := 0
	parseError := false

	for {
		token, err := decoder.Token()
		if err != nil {
			// If XML parsing fails, fall back to string-based removal
			parseError = true
			break
		}

		if token == nil {
			break
		}

		switch t := token.(type) {
		case xml.StartElement:
			if strings.EqualFold(t.Name.Local, "svg") {
				svgDepth++
				// Skip this element and its children - don't write anything
				continue
			}
			if svgDepth == 0 {
				// Only write elements that are not inside an SVG tag
				result.WriteString("<")
				if t.Name.Space != "" {
					result.WriteString(t.Name.Space + ":")
				}
				result.WriteString(t.Name.Local)
				for _, attr := range t.Attr {
					result.WriteString(" ")
					if attr.Name.Space != "" {
						result.WriteString(attr.Name.Space + ":")
					}
					result.WriteString(attr.Name.Local)
					result.WriteString(`="`)
					// Escape XML attribute value (escape &, <, and ")
					escaped := html.EscapeString(attr.Value)
					// html.EscapeString escapes &, <, > but we also need to escape " for attributes
					escaped = strings.ReplaceAll(escaped, `"`, "&quot;")
					result.WriteString(escaped)
					result.WriteString(`"`)
				}
				result.WriteString(">")
			}
		case xml.EndElement:
			if strings.EqualFold(t.Name.Local, "svg") {
				svgDepth--
				continue
			}
			if svgDepth == 0 {
				result.WriteString("</")
				if t.Name.Space != "" {
					result.WriteString(t.Name.Space + ":")
				}
				result.WriteString(t.Name.Local)
				result.WriteString(">")
			}
		case xml.CharData:
			if svgDepth == 0 {
				// Check if CharData contains XML declarations and remove them
				charData := string(t)
				// Remove XML declarations from character data using string manipulation
				for {
					start := strings.Index(strings.ToLower(charData), "<?xml")
					if start == -1 {
						break
					}
					end := strings.Index(charData[start:], "?>")
					if end == -1 {
						break
					}
					charData = charData[:start] + charData[start+end+2:]
				}
				result.WriteString(charData)
			}
		case xml.Comment:
			if svgDepth == 0 {
				result.WriteString("<!--")
				result.Write(t)
				result.WriteString("-->")
			}
		case xml.ProcInst:
			// Skip XML declarations (<?xml ...?>) - they should only appear at document start
			// Only include other processing instructions if not inside an SVG tag
			if svgDepth == 0 && !strings.EqualFold(t.Target, "xml") {
				result.WriteString("<?")
				result.WriteString(t.Target)
				if len(t.Inst) > 0 {
					result.WriteString(" ")
					result.Write(t.Inst)
				}
				result.WriteString("?>")
			}
		case xml.Directive:
			if svgDepth == 0 {
				result.WriteString("<!")
				result.Write(t)
				result.WriteString(">")
			}
		}
	}

	// If XML parsing succeeded, use the result
	if !parseError && result.Len() > 0 {
		content = result.String()
		// Remove the temporary root wrapper
		content = strings.TrimPrefix(content, "<root>")
		content = strings.TrimSuffix(content, "</root>")
	} else {
		// Fallback to simple string removal if XML parsing fails
		// Remove XML declarations first (<?xml ...?>)
		for {
			start := strings.Index(strings.ToLower(content), "<?xml")
			if start == -1 {
				break
			}
			end := strings.Index(content[start:], "?>")
			if end == -1 {
				break
			}
			content = content[:start] + content[start+end+2:]
		}
		// Remove all opening <svg> tags
		for {
			lowerContent := strings.ToLower(content)
			svgStart := strings.Index(lowerContent, "<svg")
			if svgStart == -1 {
				break
			}
			tagEnd := strings.Index(content[svgStart:], ">")
			if tagEnd == -1 {
				break
			}
			content = content[:svgStart] + content[svgStart+tagEnd+1:]
		}
		// Remove all closing </svg> tags
		svgCloseRe := regexp.MustCompile(`(?i)</\s*svg\s*>`)
		content = svgCloseRe.ReplaceAllString(content, "")
	}

	// 2. Final cleanup: Remove any remaining XML declarations (shouldn't happen, but safety check)
	for {
		start := strings.Index(strings.ToLower(content), "<?xml")
		if start == -1 {
			break
		}
		end := strings.Index(content[start:], "?>")
		if end == -1 {
			break
		}
		content = content[:start] + content[start+end+2:]
	}

	// 3. Identify Elements and Make Interactive (Same regex logic as before)
	re := regexp.MustCompile(`<g([^>]*)>(\s*<title>SRUJA_ID:([^<|]+)(?:\|([^<]*))?</title>)`)
	content = re.ReplaceAllStringFunc(content, func(match string) string {
		submatches := re.FindStringSubmatch(match)
		originalAttrs := submatches[1]
		_ = submatches[2] // fullTitleTag unused
		id := submatches[3]
		desc := submatches[4]

		newTitle := ""
		if desc != "" {
			newTitle = fmt.Sprintf("\n<title>%s</title>", desc)
		}

		// Parse existing attributes to merge class if it exists
		classRe := regexp.MustCompile(`class="([^"]*)"`)
		existingClass := ""
		if matches := classRe.FindStringSubmatch(originalAttrs); matches != nil {
			existingClass = matches[1]
			// Remove the existing class attribute from originalAttrs
			originalAttrs = classRe.ReplaceAllString(originalAttrs, "")
		}

		// Merge classes: combine existing class with "interactive"
		classes := "interactive"
		if existingClass != "" {
			classes = existingClass + " interactive"
		}

		// Add id, class, and data-content-id
		// Check if there's a drill-down view available for this element
		drillView := ""
		if _, hasView := views[id]; hasView {
			drillView = id
		}

		newAttrs := fmt.Sprintf(` id="%s" class="%s" data-content-id="%s_data"`, id, classes, id)
		if drillView != "" {
			newAttrs += fmt.Sprintf(` data-drill-view="%s"`, drillView)
		}
		// Add remaining original attributes (with class removed)
		newAttrs += originalAttrs

		return fmt.Sprintf("<g%s>%s", newAttrs, newTitle)
	})

	return content, viewBox
}

// getViewType returns the type of view for navigation purposes
func (e *Exporter) getViewType(id string, scenarioIDs, flowIDs, domainIDs, contextIDs, contractIDs, deploymentIDs, aggregateIDs, entityIDs, valueObjectIDs, eventIDs, viewDefIDs, specialViewIDs map[string]bool) string {
	if scenarioIDs[id] {
		return "scenario"
	}
	if flowIDs[id] {
		return "flow"
	}
	if domainIDs[id] {
		return "domain"
	}
	if contextIDs[id] {
		return "context"
	}
	if contractIDs[id] {
		return "contracts"
	}
	if deploymentIDs[id] {
		return "deployment"
	}
	if aggregateIDs[id] {
		return "aggregate"
	}
	if entityIDs[id] {
		return "entity"
	}
	if valueObjectIDs[id] {
		return "valueobject"
	}
	if eventIDs[id] {
		return "event"
	}
	if viewDefIDs[id] {
		return "view"
	}
	if specialViewIDs[id] {
		return id // "requirements", "adrs", "shared-artifacts", "libraries", "policies", "constraints", "conventions", "imports"
	}
	return "c4"
}

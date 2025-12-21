package dx

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
	"golang.org/x/text/cases"
	lang "golang.org/x/text/language"
)

// buildDescription creates a natural-language description of an element.
func (e *Explainer) buildDescription(elem interface{}) string {
	var sb strings.Builder

	// Handle LikeC4ElementDef
	if likeC4Elem, ok := elem.(*language.LikeC4ElementDef); ok {
		id := likeC4Elem.GetID()
		kind := likeC4Elem.GetKind()
		title := id
		t := likeC4Elem.GetTitle()
		if t != nil {
			title = *t
		}

		sb.WriteString(fmt.Sprintf("**%s: %s**\n\n", cases.Title(lang.Und).String(kind), title))

		// Extract description from body
		description := ""
		technology := ""
		nestedCounts := make(map[string]int)
		body := likeC4Elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Description != nil {
					description = *item.Description
				}
				if item.Technology != nil {
					technology = *item.Technology
				}
				if item.Element != nil {
					nestedCounts[item.Element.GetKind()]++
				}
			}
		}

		if description != "" {
			sb.WriteString(fmt.Sprintf("%s\n\n", description))
		}

		// Specific descriptions based on kind
		switch kind {
		case "system":
			sb.WriteString("This is a system that provides functionality. ")
			if count, ok := nestedCounts["container"]; ok && count > 0 {
				sb.WriteString(fmt.Sprintf("It consists of %d container(s). ", count))
			}
			if count, ok := nestedCounts["database"]; ok && count > 0 {
				sb.WriteString(fmt.Sprintf("It uses %d data store(s). ", count))
			}
		case "container":
			sb.WriteString("This is a container application within a system. ")
			if count, ok := nestedCounts["component"]; ok && count > 0 {
				sb.WriteString(fmt.Sprintf("It contains %d component(s). ", count))
			}
		case "component":
			sb.WriteString("This is a component that provides specific functionality. ")
		case "person":
			sb.WriteString("This represents a person who interacts with the architecture. ")
		default:
			sb.WriteString("This is an architecture element. ")
		}

		if technology != "" {
			sb.WriteString(fmt.Sprintf("It uses %s technology. ", technology))
		}

		return sb.String()
	}

	// Legacy support for old Architecture types (if any remain)
	switch v := elem.(type) {
	case *language.System:
		sb.WriteString(fmt.Sprintf("**System: %s**\n\n", v.Label))
		if v.Description != nil {
			sb.WriteString(fmt.Sprintf("%s\n\n", *v.Description))
		}
		sb.WriteString(fmt.Sprintf("This is a system that provides %s functionality. ", strings.ToLower(v.Label)))
		if len(v.Containers) > 0 {
			sb.WriteString(fmt.Sprintf("It consists of %d container(s). ", len(v.Containers)))
		}
		if len(v.DataStores) > 0 {
			sb.WriteString(fmt.Sprintf("It uses %d data store(s). ", len(v.DataStores)))
		}

	case *language.Container:
		sb.WriteString(fmt.Sprintf("**Container: %s**\n\n", v.Label))
		if v.Description != nil {
			sb.WriteString(fmt.Sprintf("%s\n\n", *v.Description))
		}
		sb.WriteString("This is a container application within a system. ")
		if len(v.Components) > 0 {
			sb.WriteString(fmt.Sprintf("It contains %d component(s). ", len(v.Components)))
		}

	case *language.Component:
		sb.WriteString(fmt.Sprintf("**Component: %s**\n\n", v.Label))
		if v.Description != nil {
			sb.WriteString(fmt.Sprintf("%s\n\n", *v.Description))
		}
		sb.WriteString("This is a component that provides specific functionality. ")
		if v.Technology != nil {
			sb.WriteString(fmt.Sprintf("It uses %s technology. ", *v.Technology))
		}

	case *language.Person:
		sb.WriteString(fmt.Sprintf("**Person: %s**\n\n", v.Label))
		sb.WriteString("This represents a person who interacts with the architecture. ")

	default:
		sb.WriteString("This is an architecture element. ")
	}

	return sb.String()
}

// Format formats an element explanation as a human-readable string.
func (exp *ElementExplanation) Format() string {
	var sb strings.Builder

	// Description
	sb.WriteString(exp.Description)
	sb.WriteString("\n")

	exp.formatMetadata(&sb)
	exp.formatRelations(&sb)
	exp.formatDependencies(&sb)
	exp.formatADRs(&sb)
	exp.formatScenarios(&sb)

	return sb.String()
}

func (exp *ElementExplanation) formatMetadata(sb *strings.Builder) {
	if len(exp.Metadata) == 0 {
		return
	}
	sb.WriteString("\n## Metadata\n\n")
	for key, value := range exp.Metadata {
		fmt.Fprintf(sb, "- **%s**: %s\n", key, value)
	}
	sb.WriteString("\n")
}

func (exp *ElementExplanation) formatRelations(sb *strings.Builder) {
	if len(exp.Relations.Incoming) == 0 && len(exp.Relations.Outgoing) == 0 {
		return
	}
	sb.WriteString("## Relations\n\n")
	if len(exp.Relations.Incoming) > 0 {
		sb.WriteString("### Incoming\n\n")
		for _, rel := range exp.Relations.Incoming {
			fmt.Fprintf(sb, "- **%s** → %s", rel.From, exp.ID)
			if rel.Label != "" {
				fmt.Fprintf(sb, " (%s)", rel.Label)
			}
			sb.WriteString("\n")
		}
		sb.WriteString("\n")
	}
	if len(exp.Relations.Outgoing) > 0 {
		sb.WriteString("### Outgoing\n\n")
		for _, rel := range exp.Relations.Outgoing {
			fmt.Fprintf(sb, "- %s → **%s**", exp.ID, rel.To)
			if rel.Label != "" {
				fmt.Fprintf(sb, " (%s)", rel.Label)
			}
			sb.WriteString("\n")
		}
		sb.WriteString("\n")
	}
}

func (exp *ElementExplanation) formatDependencies(sb *strings.Builder) {
	if len(exp.Dependencies) == 0 {
		return
	}
	sb.WriteString("## Dependencies\n\n")
	for _, dep := range exp.Dependencies {
		fmt.Fprintf(sb, "- %s\n", dep)
	}
	sb.WriteString("\n")
}

func (exp *ElementExplanation) formatADRs(sb *strings.Builder) {
	if len(exp.ADRs) == 0 {
		return
	}
	sb.WriteString("## Related ADRs\n\n")
	for _, adr := range exp.ADRs {
		title := ""
		if adr.Title != nil {
			title = *adr.Title
		}
		fmt.Fprintf(sb, "- **%s**: %s\n", adr.ID, title)
	}
	sb.WriteString("\n")
}

func (exp *ElementExplanation) formatScenarios(sb *strings.Builder) {
	if len(exp.Scenarios) == 0 {
		return
	}
	sb.WriteString("## Related Scenarios\n\n")
	for _, scenario := range exp.Scenarios {
		fmt.Fprintf(sb, "- **%s**: %s\n", scenario.ID, scenario.Label)
	}
	sb.WriteString("\n")
}

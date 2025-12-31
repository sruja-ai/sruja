package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

func (e *Exporter) writeRequirements(sb *strings.Builder, arch interface{}) {
	archStruct := arch.(*struct {
		Name         string
		Description  *string
		Systems      []*language.System
		Persons      []*language.Person
		Requirements []RequirementInfo
		ADRs         []ADRInfo
	})
	if len(archStruct.Requirements) == 0 {
		return
	}

	sb.WriteString("## Functional Requirements\n\n")
	sb.WriteString("This section documents the functional requirements that the system must satisfy.\n\n")

	// Use table format for better readability
	sb.WriteString("| ID | Type | Title |\n")
	sb.WriteString("|----|------|-------|\n")

	for _, req := range archStruct.Requirements {
		reqType := "functional"
		if req.Type != "" {
			reqType = req.Type
		}
		title := req.Title
		if title == "" {
			title = "Requirement specification"
		}
		fmt.Fprintf(sb, "| **%s** | %s | %s |\n", req.ID, reqType, title)
	}
	sb.WriteString("\n")
}

func (e *Exporter) writeADRs(sb *strings.Builder, arch interface{}) {
	archStruct := arch.(*struct {
		Name         string
		Description  *string
		Systems      []*language.System
		Persons      []*language.Person
		Requirements []RequirementInfo
		ADRs         []ADRInfo
	})
	if len(archStruct.ADRs) == 0 {
		return
	}

	sb.WriteString("## Architecture Decision Records\n\n")
	sb.WriteString("This section documents significant architectural decisions, their context, and rationale.\n\n")

	// Summary table
	sb.WriteString("### ADR Summary\n\n")
	sb.WriteString("| ID | Title |\n")
	sb.WriteString("|----|-------|\n")
	for _, adr := range archStruct.ADRs {
		title := adr.Title
		if title == "" {
			title = adr.ID
		}
		fmt.Fprintf(sb, "| **%s** | %s |\n", adr.ID, title)
	}
	sb.WriteString("\n")
}

// writeScenariosAndFlows writes scenarios and flows section
func (e *Exporter) writeScenariosAndFlows(sb *strings.Builder, prog *language.Program) {
	scenarios, flows := extractScenariosAndFlowsFromModel(prog)
	if len(scenarios) == 0 && len(flows) == 0 {
		return
	}

	sb.WriteString("## User Scenarios and Data Flows\n\n")
	sb.WriteString("This section documents user scenarios and data flows within the system architecture.\n\n")

	// Write scenarios
	if len(scenarios) > 0 {
		sb.WriteString("### User Scenarios\n\n")
		for _, scenario := range scenarios {
			title := scenario.Title
			if title == "" {
				title = scenario.ID
			}
			fmt.Fprintf(sb, "#### %s\n\n", title)
			if scenario.Description != "" {
				fmt.Fprintf(sb, "%s\n\n", scenario.Description)
			}

			// Generate Sequence Diagram
			if len(scenario.Steps) > 0 {
				sb.WriteString("```mermaid\n")
				sb.WriteString("sequenceDiagram\n")
				// Collect participants to verify existence or style them? (Optional)
				// Just render steps
				for _, step := range scenario.Steps {
					// Prepare participants (From/To might be FQN, mermaid handles dots usually or simple names)
					from := sanitizeIDForMermaid(step.From)
					to := sanitizeIDForMermaid(step.To)
					desc := escapeQuotesForMermaid(step.Description)
					if desc == "" {
						desc = " "
					}
					fmt.Fprintf(sb, "    participant %s\n", from)
					fmt.Fprintf(sb, "    participant %s\n", to)
					fmt.Fprintf(sb, "    %s->>%s: %s\n", from, to, desc)
				}
				sb.WriteString("```\n\n")

				// Steps Table
				sb.WriteString("**Steps:**\n\n")
				for i, step := range scenario.Steps {
					tags := ""
					if len(step.Tags) > 0 {
						tags = fmt.Sprintf(" Tags: %s", strings.Join(step.Tags, ", "))
					}
					desc := step.Description
					if desc == "" {
						desc = "Interaction"
					}
					fmt.Fprintf(sb, "%d. **%s** to **%s**: %s%s\n", i+1, step.From, step.To, desc, tags)
				}
				sb.WriteString("\n")
			}
		}
	}

	// Write flows
	if len(flows) > 0 {
		sb.WriteString("### Data Flows\n\n")
		for _, flow := range flows {
			title := flow.Title
			if title == "" {
				title = flow.ID
			}
			fmt.Fprintf(sb, "#### %s\n\n", title)
			if flow.Description != "" {
				fmt.Fprintf(sb, "%s\n\n", flow.Description)
			}

			// Generate Sequence Diagram
			if len(flow.Steps) > 0 {
				sb.WriteString("```mermaid\n")
				sb.WriteString("sequenceDiagram\n")
				for _, step := range flow.Steps {
					from := sanitizeIDForMermaid(step.From)
					to := sanitizeIDForMermaid(step.To)
					desc := escapeQuotesForMermaid(step.Description)
					if desc == "" {
						desc = " "
					}
					fmt.Fprintf(sb, "    participant %s\n", from)
					fmt.Fprintf(sb, "    participant %s\n", to)
					fmt.Fprintf(sb, "    %s->>%s: %s\n", from, to, desc)
				}
				sb.WriteString("```\n\n")
			}
		}
	}
}

// generateSequenceDiagram generates a Mermaid sequence diagram from scenario steps
func (e *Exporter) generateSequenceDiagram(steps []*language.ScenarioStep, _ string) string {
	if len(steps) == 0 {
		return ""
	}

	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	sb.WriteString("sequenceDiagram\n")

	// Collect unique participants
	participants := make(map[string]bool)
	for _, step := range steps {
		participants[step.From.String()] = true
		participants[step.To.String()] = true
	}

	// Add participants
	participantList := make([]string, 0, len(participants))
	for p := range participants {
		participantList = append(participantList, p)
	}
	// Simple sort for consistency
	for i := 0; i < len(participantList)-1; i++ {
		for j := i + 1; j < len(participantList); j++ {
			if participantList[i] > participantList[j] {
				participantList[i], participantList[j] = participantList[j], participantList[i]
			}
		}
	}

	for _, p := range participantList {
		fmt.Fprintf(sb, "    participant %s\n", sanitizeIDForMermaid(p))
	}
	sb.WriteString("\n")

	// Add interactions
	for _, step := range steps {
		from := sanitizeIDForMermaid(step.From.String())
		to := sanitizeIDForMermaid(step.To.String())
		desc := ""
		if step.Description != nil {
			desc = escapeQuotesForMermaid(*step.Description)
		}
		if desc != "" {
			fmt.Fprintf(sb, "    %s->>%s: %s\n", from, to, desc)
		} else {
			fmt.Fprintf(sb, "    %s->>%s\n", from, to)
		}
	}

	return sb.String()
}

// Recommendation represents a best practice recommendation
type Recommendation struct {
	Severity       string
	Category       string
	Element        string
	Issue          string
	Recommendation string
}

// writeRecommendations analyzes the architecture and generates best practice recommendations
func (e *Exporter) writeRecommendations(sb *strings.Builder, prog *language.Program) {
	if prog == nil || prog.Model == nil {
		return
	}

	recommendations := e.analyzeBestPractices(prog)
	if len(recommendations) == 0 {
		return
	}

	sb.WriteString("## Recommendations\n\n")
	sb.WriteString("This section identifies high-level architectural issues that should be addressed.\n\n")

	// Only show high-priority architectural flaws
	high := []Recommendation{}
	for _, rec := range recommendations {
		if rec.Severity == "high" {
			high = append(high, rec)
		}
	}

	if len(high) == 0 {
		sb.WriteString("*No critical architectural issues identified.*\n\n")
		return
	}

	for _, rec := range high {
		fmt.Fprintf(sb, "- **%s**: %s — %s\n", rec.Element, rec.Issue, rec.Recommendation)
	}
	sb.WriteString("\n")
}

// analyzeBestPractices analyzes the architecture and returns recommendations
// Focuses only on high-level architectural flaws, not informational/documentation gaps
func (e *Exporter) analyzeBestPractices(prog *language.Program) []Recommendation {
	var recommendations []Recommendation

	if prog == nil || prog.Model == nil {
		return recommendations
	}

	systems := extractSystemsFromModel(prog)
	relations := extractRelationsFromModel(prog)

	// High-level architectural flaws only

	// System without any internal structure (architectural flaw)
	for _, sys := range systems {
		if len(sys.Containers) == 0 && len(sys.DataStores) == 0 && len(sys.Queues) == 0 {
			recommendations = append(recommendations, Recommendation{
				Severity:       "high",
				Category:       "architecture",
				Element:        sys.Label,
				Issue:          "System has no internal structure",
				Recommendation: fmt.Sprintf("Define containers, data stores, or queues for %s", sys.Label),
			})
		}
	}

	// Container without technology (critical for implementation)
	for _, sys := range systems {
		for _, cont := range sys.Containers {
			hasTech := false
			if tech, ok := cont.Properties["technology"]; ok && tech != "" {
				hasTech = true
			}
			if !hasTech {
				recommendations = append(recommendations, Recommendation{
					Severity:       "high",
					Category:       "architecture",
					Element:        fmt.Sprintf("%s.%s", sys.Label, cont.Label),
					Issue:          "Container missing technology specification",
					Recommendation: fmt.Sprintf("Specify technology for %s.%s", sys.Label, cont.Label),
				})
			}
		}
	}

	// Unlabeled critical relationships (architectural clarity issue)
	unlabeledRelations := 0
	for _, rel := range relations {
		label := getString(rel.Label)
		verb := getString(rel.Verb)
		if label == "" && verb == "" {
			unlabeledRelations++
		}
	}
	if unlabeledRelations > len(relations)/2 {
		recommendations = append(recommendations, Recommendation{
			Severity:       "high",
			Category:       "architecture",
			Element:        "Relationships",
			Issue:          fmt.Sprintf("%d%% of relationships lack labels", (unlabeledRelations*100)/len(relations)),
			Recommendation: "Label relationships to clarify system interactions",
		})
	}

	return recommendations
}

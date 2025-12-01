// pkg/export/svg/svg.go
// Package svg provides SVG export functionality for Sruja architectures.
// Generates interactive, self-contained SVG files with C4 model visualization,
// requirements, ADRs, and technology documentation.
package svg

import (
	"context"
	"fmt"
	"log/slog"
	"strings"

	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
	"oss.terrastruct.com/d2/d2graph"
	"oss.terrastruct.com/d2/d2layouts/d2dagrelayout"
	"oss.terrastruct.com/d2/d2lib"
	"oss.terrastruct.com/d2/d2renderers/d2svg"
	"oss.terrastruct.com/d2/d2themes/d2themescatalog"
	d2log "oss.terrastruct.com/d2/lib/log"
	"oss.terrastruct.com/d2/lib/textmeasure"
)

// Exporter handles exporting Sruja AST to interactive SVG format.
type Exporter struct {
	elementPaths map[string]string
	nextDataID   int
	layout       *LayoutConfig
}

// NewExporter creates a new SVG exporter.
func NewExporter() *Exporter {
	return &Exporter{
		elementPaths: make(map[string]string),
		layout:       DefaultLayoutConfig(),
	}
}

// Export converts a Sruja architecture to interactive SVG string.
func (e *Exporter) Export(arch *language.Architecture) (string, error) {
	e.buildElementPaths(arch)
	e.nextDataID = 1

	// Map to store generated SVGs for each view
	// Key: view ID (e.g., "level1", "Sys1", "Cont1")
	// Value: SVG content
	views := make(map[string]string)

	// 1. Level 1: System Context
	l1SVG, err := e.renderD2(arch, 1, "")
	if err != nil {
		return "", fmt.Errorf("failed to render Level 1: %w", err)
	}
	views["level1"] = l1SVG

	// 2. Level 2: Container Views (per System)
	for _, sys := range arch.Systems {
		// Only generate if system has containers
		if len(sys.Containers) > 0 || len(sys.DataStores) > 0 || len(sys.Queues) > 0 {
			l2SVG, err := e.renderD2(arch, 2, sys.ID)
			if err != nil {
				return "", fmt.Errorf("failed to render Level 2 for %s: %w", sys.ID, err)
			}
			views[sys.ID] = l2SVG

			// 3. Level 3: Component Views (per Container)
			for _, cont := range sys.Containers {
				if len(cont.Components) > 0 {
					l3SVG, err := e.renderD2(arch, 3, cont.ID)
					if err != nil {
						return "", fmt.Errorf("failed to render Level 3 for %s: %w", cont.ID, err)
					}
					views[cont.ID] = l3SVG
				}
			}
		}
	}

	// 4. Scenario Views (independent - each scenario is a separate diagram)
	for _, scenario := range arch.Scenarios {
		scenarioSVG, err := e.renderScenario(arch, scenario)
		if err != nil {
			return "", fmt.Errorf("failed to render scenario %s: %w", scenario.Title, err)
		}
		viewID := fmt.Sprintf("scenario-%s", sanitizeID(scenario.Title))
		views[viewID] = scenarioSVG
	}

	// 5. Flow Views (independent - each flow is a separate diagram)
	for _, sys := range arch.Systems {
		for _, flow := range sys.Flows {
			flowSVG, err := e.renderFlow(arch, sys, flow)
			if err != nil {
				return "", fmt.Errorf("failed to render flow %s: %w", flow.Title, err)
			}
			viewID := fmt.Sprintf("flow-%s-%s", sys.ID, sanitizeID(flow.Title))
			views[viewID] = flowSVG
		}
	}

	// 6. Requirements Layer View (independent diagram)
	hasRequirements := len(arch.Requirements) > 0
	if !hasRequirements {
		for _, sys := range arch.Systems {
			if len(sys.Requirements) > 0 {
				hasRequirements = true
				break
			}
		}
	}
	if hasRequirements {
		reqSVG, err := e.renderRequirements(arch)
		if err != nil {
			return "", fmt.Errorf("failed to render requirements: %w", err)
		}
		views["requirements"] = reqSVG
	}

	// 7. ADRs Layer View (independent diagram)
	hasADRs := len(arch.ADRs) > 0
	if !hasADRs {
		for _, sys := range arch.Systems {
			if len(sys.ADRs) > 0 {
				hasADRs = true
				break
			}
		}
	}
	if hasADRs {
		adrSVG, err := e.renderADRs(arch)
		if err != nil {
			return "", fmt.Errorf("failed to render ADRs: %w", err)
		}
		views["adrs"] = adrSVG
	}

	// 8. DDD Domain Views (independent - each domain is a separate diagram)
	for _, domain := range arch.Domains {
		domainSVG, err := e.renderDomain(arch, domain)
		if err != nil {
			return "", fmt.Errorf("failed to render domain %s: %w", domain.ID, err)
		}
		viewID := fmt.Sprintf("domain-%s", domain.ID)
		views[viewID] = domainSVG
	}

	// 9. DDD Context Views (independent - each context is a separate diagram)
	for _, domain := range arch.Domains {
		for _, ctx := range domain.Contexts {
			ctxSVG, err := e.renderContext(arch, domain, ctx)
			if err != nil {
				return "", fmt.Errorf("failed to render context %s: %w", ctx.ID, err)
			}
			viewID := fmt.Sprintf("context-%s-%s", domain.ID, ctx.ID)
			views[viewID] = ctxSVG
		}
	}

	// 10. Contracts Views (independent - contracts can be visualized per system/container/component)
	// Group contracts by their parent (system/container/component) for better organization
	contractGroups := make(map[string][]*language.Contract)

	// Architecture-level contracts
	if len(arch.Contracts) > 0 {
		contractGroups["arch"] = arch.Contracts
	}

	// System-level contracts
	for _, sys := range arch.Systems {
		if len(sys.Contracts) > 0 {
			contractGroups[fmt.Sprintf("system-%s", sys.ID)] = sys.Contracts
		}
		// Container-level contracts
		for _, cont := range sys.Containers {
			if len(cont.Contracts) > 0 {
				contractGroups[fmt.Sprintf("container-%s-%s", sys.ID, cont.ID)] = cont.Contracts
			}
		}
	}

	// Render each contract group as a separate view
	for groupID, contracts := range contractGroups {
		contractSVG, err := e.renderContracts(arch, groupID, contracts)
		if err != nil {
			return "", fmt.Errorf("failed to render contracts for %s: %w", groupID, err)
		}
		viewID := fmt.Sprintf("contracts-%s", groupID)
		views[viewID] = contractSVG
	}

	// 11. Deployment Views (independent - each deployment node is a separate diagram)
	for _, deployment := range arch.DeploymentNodes {
		deploymentSVG, err := e.renderDeployment(arch, deployment)
		if err != nil {
			return "", fmt.Errorf("failed to render deployment %s: %w", deployment.ID, err)
		}
		viewID := fmt.Sprintf("deployment-%s", deployment.ID)
		views[viewID] = deploymentSVG
	}

	// 12. DDD Entity/Aggregate/ValueObject Views (independent - show domain model details)
	for _, domain := range arch.Domains {
		for _, ctx := range domain.Contexts {
			// Aggregate views
			for _, agg := range ctx.Aggregates {
				aggSVG, err := e.renderAggregate(arch, domain, ctx, agg)
				if err != nil {
					return "", fmt.Errorf("failed to render aggregate %s: %w", agg.ID, err)
				}
				viewID := fmt.Sprintf("aggregate-%s-%s-%s", domain.ID, ctx.ID, agg.ID)
				views[viewID] = aggSVG
			}
			// Entity views (standalone entities not in aggregates)
			for _, entity := range ctx.Entities {
				// Check if entity is already in an aggregate
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
					entitySVG, err := e.renderEntity(arch, domain, ctx, entity)
					if err != nil {
						return "", fmt.Errorf("failed to render entity %s: %w", entity.ID, err)
					}
					viewID := fmt.Sprintf("entity-%s-%s-%s", domain.ID, ctx.ID, entity.ID)
					views[viewID] = entitySVG
				}
			}
			// ValueObject views (standalone value objects not in aggregates/entities)
			for _, vo := range ctx.ValueObjects {
				// Check if value object is in an aggregate or entity
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
					voSVG, err := e.renderValueObject(arch, domain, ctx, vo)
					if err != nil {
						return "", fmt.Errorf("failed to render value object %s: %w", vo.ID, err)
					}
					viewID := fmt.Sprintf("valueobject-%s-%s-%s", domain.ID, ctx.ID, vo.ID)
					views[viewID] = voSVG
				}
			}
			// DomainEvent views
			for _, event := range ctx.Events {
				eventSVG, err := e.renderDomainEvent(arch, domain, ctx, event)
				if err != nil {
					return "", fmt.Errorf("failed to render domain event %s: %w", event.ID, err)
				}
				viewID := fmt.Sprintf("event-%s-%s-%s", domain.ID, ctx.ID, event.ID)
				views[viewID] = eventSVG
			}
		}
	}

	// 13. SharedArtifacts & Libraries Views
	if len(arch.SharedArtifacts) > 0 {
		sharedSVG, err := e.renderSharedArtifacts(arch)
		if err != nil {
			return "", fmt.Errorf("failed to render shared artifacts: %w", err)
		}
		views["shared-artifacts"] = sharedSVG
	}

	if len(arch.Libraries) > 0 {
		libSVG, err := e.renderLibraries(arch)
		if err != nil {
			return "", fmt.Errorf("failed to render libraries: %w", err)
		}
		views["libraries"] = libSVG
	}

	// 16. Policy Views (independent - show governance policies)
	allPolicies := e.collectPolicies(arch)
	if len(allPolicies) > 0 {
		policySVG, err := e.renderPolicies(arch, allPolicies)
		if err != nil {
			return "", fmt.Errorf("failed to render policies: %w", err)
		}
		views["policies"] = policySVG
	}

	// 17. Constraints/Conventions Views (independent - show governance rules)
	if len(arch.Constraints) > 0 || e.hasSystemConstraints(arch) {
		constraintSVG, err := e.renderConstraints(arch)
		if err != nil {
			return "", fmt.Errorf("failed to render constraints: %w", err)
		}
		views["constraints"] = constraintSVG
	}

	if len(arch.Conventions) > 0 || e.hasSystemConventions(arch) {
		conventionSVG, err := e.renderConventions(arch)
		if err != nil {
			return "", fmt.Errorf("failed to render conventions: %w", err)
		}
		views["conventions"] = conventionSVG
	}

	// 18. View Definitions (independent - custom filtered views)
	for _, view := range arch.Views {
		viewSVG, err := e.renderView(arch, view)
		if err != nil {
			return "", fmt.Errorf("failed to render view %s: %w", view.ID, err)
		}
		viewID := fmt.Sprintf("view-%s", view.ID)
		views[viewID] = viewSVG
	}

	// 19. Import Views (independent - multi-architecture composition)
	if len(arch.ResolvedImports) > 0 {
		importSVG, err := e.renderImports(arch)
		if err != nil {
			return "", fmt.Errorf("failed to render imports: %w", err)
		}
		views["imports"] = importSVG
	}

	// 20. Post-Process and Stitch all independent diagrams
	finalSVG := e.postProcessSVG(views, arch)

	return finalSVG, nil
}

func (e *Exporter) renderD2(arch *language.Architecture, level int, elementID string) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = level
	d2Exporter.ScopeElementID = elementID

	d2Script, err := d2Exporter.Export(arch)
	if err != nil {
		return "", err
	}

	return e.compileAndRenderD2(d2Script)
}

// compileAndRenderD2 compiles D2 script and renders to SVG
func (e *Exporter) compileAndRenderD2(d2Script string) (string, error) {
	// Validate D2 script is not empty
	if strings.TrimSpace(d2Script) == "" {
		return "", fmt.Errorf("empty D2 script provided")
	}

	ruler, err := textmeasure.NewRuler()
	if err != nil {
		return "", fmt.Errorf("failed to create ruler: %w", err)
	}

	renderOpts := &d2svg.RenderOpts{
		ThemeID: &d2themescatalog.GrapeSoda.ID,
	}

	// Create a logger context to suppress D2 warnings
	logger := slog.New(slog.NewTextHandler(nil, &slog.HandlerOptions{Level: slog.LevelError}))
	ctx := d2log.With(context.Background(), logger)

	diagram, _, err := d2lib.Compile(ctx, d2Script, &d2lib.CompileOptions{
		Ruler: ruler,
		LayoutResolver: func(engine string) (d2graph.LayoutGraph, error) {
			return func(ctx context.Context, g *d2graph.Graph) error {
				return d2dagrelayout.Layout(ctx, g, nil)
			}, nil
		},
	}, renderOpts)
	if err != nil {
		return "", fmt.Errorf("D2 compilation failed: %w (script preview: %s)", err, truncateString(d2Script, 200))
	}

	svgBytes, err := d2svg.Render(diagram, renderOpts)
	if err != nil {
		return "", fmt.Errorf("D2 rendering failed: %w", err)
	}

	return string(svgBytes), nil
}

// truncateString truncates a string to max length
func truncateString(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}

// extractScenarioElements extracts all architecture element IDs referenced in a scenario
func (e *Exporter) extractScenarioElements(arch *language.Architecture, scenario *language.Scenario) map[string]bool {
	elements := make(map[string]bool)

	for _, step := range scenario.Steps {
		// Add From element
		fromID := step.From
		if e.isArchitectureElement(arch, fromID) {
			elements[fromID] = true
		}

		// Add To element if present
		if step.To != nil {
			toID := *step.To
			if e.isArchitectureElement(arch, toID) {
				elements[toID] = true
			}
		}
	}

	return elements
}

// extractFlowElements extracts all architecture element IDs referenced in a flow
func (e *Exporter) extractFlowElements(arch *language.Architecture, flow *language.Flow) map[string]bool {
	elements := make(map[string]bool)

	for _, step := range flow.Steps {
		// Add From element
		if e.isArchitectureElement(arch, step.From) {
			elements[step.From] = true
		}

		// Add To element
		if e.isArchitectureElement(arch, step.To) {
			elements[step.To] = true
		}
	}

	return elements
}

// isArchitectureElement checks if an ID refers to an architecture element
func (e *Exporter) isArchitectureElement(arch *language.Architecture, id string) bool {
	// Check persons
	for _, p := range arch.Persons {
		if p.ID == id {
			return true
		}
	}

	// Check systems and their nested elements
	for _, sys := range arch.Systems {
		if sys.ID == id {
			return true
		}
		for _, cont := range sys.Containers {
			if cont.ID == id {
				return true
			}
			for _, comp := range cont.Components {
				if comp.ID == id {
					return true
				}
			}
		}
		for _, comp := range sys.Components {
			if comp.ID == id {
				return true
			}
		}
		for _, ds := range sys.DataStores {
			if ds.ID == id {
				return true
			}
		}
		for _, q := range sys.Queues {
			if q.ID == id {
				return true
			}
		}
	}

	return false
}

// filterArchitectureForElements creates a filtered architecture containing only the specified elements
func (e *Exporter) filterArchitectureForElements(arch *language.Architecture, elementIDs map[string]bool) *language.Architecture {
	filtered := &language.Architecture{
		Name: arch.Name,
	}

	// Filter persons
	for _, p := range arch.Persons {
		if elementIDs[p.ID] {
			filtered.Persons = append(filtered.Persons, p)
		}
	}

	// Filter systems and their nested elements
	for _, sys := range arch.Systems {
		// Check if system itself is involved or any of its children
		sysInvolved := elementIDs[sys.ID]
		hasInvolvedChildren := false

		// Create filtered system
		filteredSys := &language.System{
			ID:          sys.ID,
			Label:       sys.Label,
			Description: sys.Description,
		}

		// Filter containers
		for _, cont := range sys.Containers {
			if elementIDs[cont.ID] {
				hasInvolvedChildren = true
				filteredCont := &language.Container{
					ID:          cont.ID,
					Label:       cont.Label,
					Description: cont.Description,
				}
				// Include all components if container is involved
				filteredCont.Components = cont.Components
				filteredSys.Containers = append(filteredSys.Containers, filteredCont)
			} else {
				// Check if any component in this container is involved
				for _, comp := range cont.Components {
					if elementIDs[comp.ID] {
						hasInvolvedChildren = true
						filteredCont := &language.Container{
							ID:          cont.ID,
							Label:       cont.Label,
							Description: cont.Description,
						}
						// Include only involved components
						for _, c := range cont.Components {
							if elementIDs[c.ID] {
								filteredCont.Components = append(filteredCont.Components, c)
							}
						}
						filteredSys.Containers = append(filteredSys.Containers, filteredCont)
						break
					}
				}
			}
		}

		// Filter components (direct in system)
		for _, comp := range sys.Components {
			if elementIDs[comp.ID] {
				hasInvolvedChildren = true
				filteredSys.Components = append(filteredSys.Components, comp)
			}
		}

		// Filter datastores
		for _, ds := range sys.DataStores {
			if elementIDs[ds.ID] {
				hasInvolvedChildren = true
				filteredSys.DataStores = append(filteredSys.DataStores, ds)
			}
		}

		// Filter queues
		for _, q := range sys.Queues {
			if elementIDs[q.ID] {
				hasInvolvedChildren = true
				filteredSys.Queues = append(filteredSys.Queues, q)
			}
		}

		// Include system if it or its children are involved
		if sysInvolved || hasInvolvedChildren {
			// Include all relations that involve this system or its elements
			for _, rel := range sys.Relations {
				if elementIDs[rel.From] || elementIDs[rel.To] {
					filteredSys.Relations = append(filteredSys.Relations, rel)
				}
			}
			filtered.Systems = append(filtered.Systems, filteredSys)
		}
	}

	// Filter top-level relations
	for _, rel := range arch.Relations {
		if elementIDs[rel.From] || elementIDs[rel.To] {
			filtered.Relations = append(filtered.Relations, rel)
		}
	}

	return filtered
}

// collectPolicies collects all policies from architecture and all nested levels
func (e *Exporter) collectPolicies(arch *language.Architecture) []*language.Policy {
	var policies []*language.Policy

	// Note: Policies are defined in ArchitectureItem/SystemItem/ContainerItem/ComponentItem
	// but not as post-processed fields. For now, return empty list.
	// Full implementation would require parsing the Items arrays or adding post-processing.
	// This is a placeholder for future implementation.

	return policies
}

// Helper methods to check for constraints/conventions
func (e *Exporter) hasSystemConstraints(arch *language.Architecture) bool {
	for _, sys := range arch.Systems {
		if len(sys.Constraints) > 0 {
			return true
		}
	}
	return false
}

func (e *Exporter) hasSystemConventions(arch *language.Architecture) bool {
	for _, sys := range arch.Systems {
		if len(sys.Conventions) > 0 {
			return true
		}
	}
	return false
}

// sanitizeID creates a safe ID from a title string
func sanitizeID(title string) string {
	// Simple sanitization: replace spaces and special chars with hyphens
	result := ""
	for _, r := range title {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') {
			result += string(r)
		} else {
			result += "-"
		}
	}
	return result
}

// Helper methods for UI generation (writeStyles, writeJavaScript, etc.) are in other files.
// We need to ensure they write to a strings.Builder passed to them.
// The existing methods signature is (sb *strings.Builder, ...).

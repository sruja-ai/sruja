package svg

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/export/svg/icons"
	"github.com/sruja-ai/sruja/pkg/language"
)

// ExportAll renders a flat graph with all elements and relations
//
//nolint:funlen,gocyclo // Export logic is comprehensive
func (e *Exporter) ExportAll(arch *language.Architecture) (string, error) {
	if arch == nil {
		return "", fmt.Errorf("architecture is nil")
	}
	theme := e.Theme
	if theme == nil {
		theme = ProfessionalTheme()
	}

	// 1. Collect metadata
	kind, label, desc := collectMetadata(arch)

	// 2. Layout systems
	sysInternalPos := make(map[string]map[string]point)
	sysDims := make(map[string]layoutNode)
	for _, s := range arch.Systems {
		dim, positions := e.layoutSystemInternals(s, arch)
		sysDims[s.ID] = dim
		sysInternalPos[s.ID] = positions
	}

	// 3. Layout top level
	topNodes, topKind, topLabel := e.determineTopLevelNodes(arch, sysDims, kind)
	// Update metadata maps with externals found
	for k, v := range topKind {
		kind[k] = v
	}
	for k, v := range topLabel {
		label[k] = v
	}

	topEdges := collectTopLevelEdges(arch, topNodes)

	topRes := layoutCluster(topNodes, topEdges, layoutConfig{
		Direction:         e.Direction,
		HorizontalSpacing: e.HorizontalSpacing * 2, // More spacing for top level
		VerticalSpacing:   e.VerticalSpacing * 2,
		Padding:           e.Padding,
	})

	// 5. Calculate final positions
	positions := make(map[string]point)

	// Top level positions
	for id, p := range topRes.Positions {
		positions[id] = p
	}

	// Internal positions (relative -> absolute)
	for sID, internals := range sysInternalPos {
		sysPos := positions[sID]
		// Offset for system padding and title
		offsetX := sysPos.X + 16
		offsetY := sysPos.Y + 16 + e.TitleFontSize + 20

		for id, p := range internals {
			positions[id] = point{X: p.X + offsetX, Y: p.Y + offsetY}
		}
	}

	// Calculate total dimensions
	width := topRes.Width
	height := topRes.Height

	// Title adjustment
	titleOffset := 0
	if e.ShowTitle && arch.Name != "" {
		titleOffset = e.TitleFontSize + 10
		titleWidth := len(arch.Name)*12 + e.Padding*2
		if titleWidth > width {
			width = titleWidth
		}
	}
	height += titleOffset

	// Shift all positions down if title is shown
	if titleOffset > 0 {
		for id, p := range positions {
			positions[id] = point{X: p.X, Y: p.Y + titleOffset}
		}
	}

	// defs
	iconDefs := []IconDef{}
	for _, n := range []string{"user", "home", "server", "database", "inbox", "box"} {
		if c := icons.Get(n); c != "" {
			iconDefs = append(iconDefs, IconDef{Name: n, Content: c})
		}
	}
	defs, err := renderDefs(theme, iconDefs)
	if err != nil {
		return "", fmt.Errorf("failed to render defs: %w", err)
	}

	// edges
	svgEdges := e.collectSVGEdges(arch, positions, sysDims, label, theme)

	// nodes
	svgNodes := e.collectSVGNodes(positions, sysDims, kind, label, desc, theme)

	// Build groups (system boundaries)
	groups := []Group{}
	for _, s := range arch.Systems {
		if dim, ok := sysDims[s.ID]; ok {
			p := positions[s.ID]
			groups = append(groups, Group{
				X:          p.X,
				Y:          p.Y,
				Width:      dim.Width,
				Height:     dim.Height,
				Title:      firstNonEmpty(s.Label, s.ID),
				Stroke:     theme.SystemStroke,
				Fill:       "none",
				TitleColor: theme.TitleColor,
			})
		}
	}

	// Legend for all view
	legendItems := []LegendItem{}
	if e.ShowLegend {
		legendItems = append(legendItems,
			LegendItem{Icon: IconUser, Label: "Person", Stroke: theme.PersonStroke, Fill: theme.PersonFill},
			LegendItem{Icon: IconHome, Label: "System", Stroke: theme.SystemStroke, Fill: theme.SystemFill},
			LegendItem{Icon: IconServer, Label: "Container", Stroke: theme.ContainerStroke, Fill: theme.ContainerFill},
			LegendItem{Icon: IconDatabase, Label: "Database", Stroke: theme.DatabaseStroke, Fill: theme.DatabaseFill},
			LegendItem{Icon: IconInbox, Label: "Queue", Stroke: theme.QueueStroke, Fill: theme.QueueFill},
			LegendItem{Icon: IconBox, Label: "Component", Stroke: theme.ContainerStroke, Fill: theme.ContainerFill},
		)
	}

	data := &TemplateData{
		Width:      width,
		Height:     height,
		Background: theme.Background,
		Theme:      theme,
		ShowTitle:  e.ShowTitle && arch.Name != "",
		Title:      arch.Name,
		TitleX:     e.Padding,
		TitleY:     e.Padding + theme.TitleFontSize,
		Defs:       defs,
		Edges:      svgEdges,
		Nodes:      svgNodes,
		Groups:     groups,
		Legend:     Legend{Show: e.ShowLegend, X: width - e.Padding - 210, Y: e.Padding, Items: legendItems, Title: "Legend"},
		Metadata:   e.Metadata,
		EmbedFonts: e.EmbedFonts,
	}
	return renderSVG(data)
}

func firstNonEmpty(a, b string) string {
	if a != "" {
		return a
	}
	return b
}

func deref(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

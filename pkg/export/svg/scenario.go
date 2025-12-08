// pkg/export/svg/scenario.go
package svg

import (
	"fmt"
	"sort"
	"strings"

	"gonum.org/v1/gonum/graph/simple"

	"github.com/sruja-ai/sruja/pkg/export/svg/icons"
	"github.com/sruja-ai/sruja/pkg/language"
)

// ExportScenario exports a scenario/story as an SVG diagram showing the flow of interactions
//
//nolint:funlen,gocyclo // Scenario export logic is detailed
func (e *Exporter) ExportScenario(arch *language.Architecture, scenario *language.Scenario) (string, error) {
	if arch == nil || scenario == nil {
		return "", fmt.Errorf("invalid input")
	}

	theme := e.Theme
	if theme == nil {
		theme = ProfessionalTheme()
	}

	// Collect unique nodes from scenario steps
	nodeSet := make(map[string]bool)
	for _, step := range scenario.Steps {
		nodeSet[step.From.String()] = true
		nodeSet[step.To.String()] = true
	}

	// Build node information map
	nodes := make(map[string]node)
	for id := range nodeSet {
		label := id
		kind := KindExternal

		// Persons (top-level)
		for _, p := range arch.Persons {
			if p.ID == id {
				label = firstNonEmpty(p.Label, p.ID)
				kind = KindPerson
				break
			}
		}
		if kind != KindExternal {
			nodes[id] = node{ID: id, Label: label, Kind: kind}
			continue
		}

		// Systems (qualified: sys.ID)
		for _, s := range arch.Systems {
			if s.ID == id {
				label = firstNonEmpty(s.Label, s.ID)
				kind = KindSystem
				break
			}
		}
		if kind != KindExternal {
			nodes[id] = node{ID: id, Label: label, Kind: kind}
			continue
		}

		// Top-level containers/components/datastores/queues
		for _, c := range arch.Containers {
			if c.ID == id {
				label = firstNonEmpty(c.Label, c.ID)
				kind = KindContainer
				break
			}
		}
		if kind == KindExternal {
			for _, comp := range arch.Components {
				if comp.ID == id {
					label = firstNonEmpty(comp.Label, comp.ID)
					kind = KindComponent
					break
				}
			}
		}
		if kind == KindExternal {
			for _, ds := range arch.DataStores {
				if ds.ID == id {
					label = firstNonEmpty(ds.Label, ds.ID)
					kind = KindContainer // render like container
					break
				}
			}
		}
		if kind == KindExternal {
			for _, q := range arch.Queues {
				if q.ID == id {
					label = firstNonEmpty(q.Label, q.ID)
					kind = KindContainer // render like container
					break
				}
			}
		}
		if kind != KindExternal {
			nodes[id] = node{ID: id, Label: label, Kind: kind}
			continue
		}

		// Containers/components within systems (qualified match)
		for _, s := range arch.Systems {
			for _, c := range s.Containers {
				contQ := s.ID + "." + c.ID
				if contQ == id {
					label = firstNonEmpty(c.Label, c.ID)
					kind = KindContainer
					break
				}
			}
			if kind != KindExternal {
				break
			}
			for _, c := range s.Containers {
				for _, comp := range c.Components {
					compQ := s.ID + "." + c.ID + "." + comp.ID
					if compQ == id {
						label = firstNonEmpty(comp.Label, comp.ID)
						kind = "component"
						break
					}
				}
				if kind != KindExternal {
					break
				}
			}
			if kind != KindExternal {
				break
			}
			for _, comp := range s.Components {
				compQ := s.ID + "." + comp.ID
				if compQ == id {
					label = firstNonEmpty(comp.Label, comp.ID)
					kind = "component"
					break
				}
			}
			if kind != KindExternal {
				break
			}
			for _, ds := range s.DataStores {
				dsQ := s.ID + "." + ds.ID
				if dsQ == id {
					label = firstNonEmpty(ds.Label, ds.ID)
					kind = "container"
					break
				}
			}
			if kind != KindExternal {
				break
			}
			for _, q := range s.Queues {
				qQ := s.ID + "." + q.ID
				if qQ == id {
					label = firstNonEmpty(q.Label, q.ID)
					kind = "container"
					break
				}
			}
			if kind != "external" {
				break
			}
		}

		nodes[id] = node{ID: id, Label: label, Kind: kind}
	}

	// Build graph for layout
	g := simple.NewDirectedGraph()
	idMap := make(map[string]int64)
	nextID := int64(1)
	addNode := func(name string) int64 {
		if id, ok := idMap[name]; ok {
			return id
		}
		id := nextID
		nextID++
		idMap[name] = id
		g.AddNode(simple.Node(id))
		return id
	}

	for id := range nodes {
		addNode(id)
	}

	// Add edges from scenario steps
	for _, step := range scenario.Steps {
		fromID := addNode(step.From.String())
		toID := addNode(step.To.String())
		if !g.HasEdgeFromTo(fromID, toID) {
			g.SetEdge(simple.Edge{F: simple.Node(fromID), T: simple.Node(toID)})
		}
	}

	// Layout using longest path layers
	layers := longestPathLayers(g)
	layerNodes := make(map[int][]int64)
	for name, id := range idMap {
		_ = name
		l := layers[id]
		layerNodes[l] = append(layerNodes[l], id)
	}

	// Sort nodes within each layer
	for l := range layerNodes {
		sort.Slice(layerNodes[l], func(i, j int) bool {
			a := layerNodes[l][i]
			b := layerNodes[l][j]
			return nodes[nameByID(idMap, a)].Label < nodes[nameByID(idMap, b)].Label
		})
	}

	maxLayer := 0
	for l := range layerNodes {
		if l > maxLayer {
			maxLayer = l
		}
	}
	cols := maxLayer + 1
	maxRows := 0
	for _, ids := range layerNodes {
		if len(ids) > maxRows {
			maxRows = len(ids)
		}
	}

	titleOffset := 0
	if e.ShowTitle {
		titleOffset = e.TitleFontSize + 10
	}
	contentTopMargin := e.TitleFontSize + 40
	width := e.Padding*2 + cols*e.NodeWidth + (cols-1)*e.HorizontalSpacing
	height := e.Padding*2 + maxRows*e.NodeHeight + (maxRows-1)*e.VerticalSpacing + titleOffset + contentTopMargin

	// Calculate positions
	positions := make(map[int64]point)
	for l := 0; l <= maxLayer; l++ {
		ids := layerNodes[l]
		for i, id := range ids {
			var x, y int
			switch strings.ToUpper(e.Direction) {
			case "LR":
				x = e.Padding + l*(e.NodeWidth+e.HorizontalSpacing)
				y = e.Padding + titleOffset + i*(e.NodeHeight+e.VerticalSpacing) + contentTopMargin
			case "RL":
				x = e.Padding + (cols-1-l)*(e.NodeWidth+e.HorizontalSpacing)
				y = e.Padding + titleOffset + i*(e.NodeHeight+e.VerticalSpacing) + contentTopMargin
			case "TB":
				x = e.Padding + i*(e.NodeWidth+e.HorizontalSpacing)
				y = e.Padding + titleOffset + l*(e.NodeHeight+e.VerticalSpacing) + contentTopMargin
			case "BT":
				x = e.Padding + i*(e.NodeWidth+e.HorizontalSpacing)
				y = e.Padding + titleOffset + (cols-1-l)*(e.NodeHeight+e.VerticalSpacing) + contentTopMargin
			default:
				x = e.Padding + l*(e.NodeWidth+e.HorizontalSpacing)
				y = e.Padding + titleOffset + i*(e.NodeHeight+e.VerticalSpacing) + contentTopMargin
			}
			positions[id] = point{X: x, Y: y}
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
	svgEdges := []Edge{}
	edgeSet := make(map[string]bool)
	for _, step := range scenario.Steps {
		fromID := idMap[step.From.String()]
		toID := idMap[step.To.String()]
		key := fmt.Sprintf("%d->%d", fromID, toID)
		if !edgeSet[key] {
			edgeSet[key] = true
			fromPos := positions[fromID]
			toPos := positions[toID]

			var x1, y1, x2, y2 int
			switch strings.ToUpper(e.Direction) {
			case "LR":
				x1 = fromPos.X + e.NodeWidth
				y1 = fromPos.Y + e.NodeHeight/2
				x2 = toPos.X
				y2 = toPos.Y + e.NodeHeight/2
			case "RL":
				x1 = fromPos.X
				y1 = fromPos.Y + e.NodeHeight/2
				x2 = toPos.X + e.NodeWidth
				y2 = toPos.Y + e.NodeHeight/2
			case "TB":
				x1 = fromPos.X + e.NodeWidth/2
				y1 = fromPos.Y + e.NodeHeight
				x2 = toPos.X + e.NodeWidth/2
				y2 = toPos.Y
			case "BT":
				x1 = fromPos.X + e.NodeWidth/2
				y1 = fromPos.Y
				x2 = toPos.X + e.NodeWidth/2
				y2 = toPos.Y + e.NodeHeight
			default:
				x1 = fromPos.X + e.NodeWidth
				y1 = fromPos.Y + e.NodeHeight/2
				x2 = toPos.X
				y2 = toPos.Y + e.NodeHeight/2
			}

			edge := Edge{
				X1: x1, Y1: y1, X2: x2, Y2: y2,
				Color: theme.EdgeColor,
				Width: theme.EdgeWidth,
			}
			if step.Description != nil && *step.Description != "" {
				edge.Label = *step.Description
				edge.LabelX = (x1 + x2) / 2
				edge.LabelY = (y1 + y2) / 2
				edge.LabelBg = true
			}
			edge.Title = fmt.Sprintf("%s to %s", nodes[step.From.String()].Label, nodes[step.To.String()].Label)
			svgEdges = append(svgEdges, edge)
		}
	}

	// nodes
	svgNodes := []Node{}
	for name, id := range idMap {
		el := nodes[name]
		p := positions[id]
		node := Node{
			X:           p.X,
			Y:           p.Y,
			Width:       e.NodeWidth,
			Height:      e.NodeHeight,
			StrokeWidth: 2,
			LabelColor:  theme.LabelColor,
			LabelSize:   13,
		}

		switch el.Kind {
		case KindPerson:
			node.Fill = theme.PersonFill
			node.Stroke = theme.PersonStroke
			node.Icon = "icon-user"
		case KindSystem:
			node.Fill = theme.SystemFill
			node.Stroke = theme.SystemStroke
			node.Icon = "icon-home"
		case KindContainer:
			node.Fill = theme.ContainerFill
			node.Stroke = theme.ContainerStroke
			node.Icon = "icon-server"
		case KindComponent:
			node.Fill = theme.ContainerFill
			node.Stroke = theme.ContainerStroke
			node.Icon = "icon-box"
		default:
			node.Fill = "#eeeeee"
			node.Stroke = "#666"
		}

		node.Label = el.Label
		node.LabelX = p.X + e.NodeWidth/2
		node.LabelY = p.Y + e.NodeHeight/2

		if node.Icon != "" {
			iconTarget := int(0.28 * float64(min(e.NodeWidth, e.NodeHeight)))
			if iconTarget < 16 {
				iconTarget = 16
			}
			if iconTarget > 24 {
				iconTarget = 24
			}
			node.IconW = iconTarget
			node.IconH = iconTarget
			node.IconStrokeWidth = 1.8 * 24.0 / float64(iconTarget)
			iconPadding := 10
			node.IconX = p.X + iconPadding
			node.IconY = p.Y + e.NodeHeight/2 - iconTarget/2
			node.LabelX = p.X + iconPadding + iconTarget + 8
		}

		rightPad := 10
		var leftUsed int
		if node.Icon != "" {
			leftUsed = node.LabelX - p.X
		} else {
			leftUsed = 10
		}
		avail := e.NodeWidth - leftUsed - rightPad
		approxChar := int(float64(node.LabelSize) * 0.6)
		if approxChar < 6 {
			approxChar = 6
		}
		lines := wrapLabel(node.Label, avail/approxChar)
		if len(lines) > 1 {
			node.LineHeight = node.LabelSize + 10
		} else {
			node.LineHeight = node.LabelSize + 2
		}
		node.WrappedLines = lines
		node.Title = node.Label
		svgNodes = append(svgNodes, node)
	}

	data := &TemplateData{
		Width:      width,
		Height:     height,
		Background: theme.Background,
		Theme:      theme,
		ShowTitle:  e.ShowTitle,
		Title:      firstNonEmpty(scenario.Title, scenario.ID),
		TitleX:     e.Padding,
		TitleY:     e.Padding + theme.TitleFontSize,
		Defs:       defs,
		Edges:      svgEdges,
		Nodes:      svgNodes,
		EmbedFonts: e.EmbedFonts,
	}
	return renderSVG(data)
}

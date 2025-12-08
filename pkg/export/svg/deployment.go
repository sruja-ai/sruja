package svg

import (
	"fmt"
	"sort"
	"strings"

	"gonum.org/v1/gonum/graph/simple"

	"github.com/sruja-ai/sruja/pkg/export/svg/icons"
	"github.com/sruja-ai/sruja/pkg/language"
)

//nolint:funlen,gocyclo // Deployment export logic is detailed
func (e *Exporter) ExportDeployment(arch *language.Architecture, dep *language.DeploymentNode) (string, error) {
	if arch == nil || dep == nil {
		return "", fmt.Errorf("invalid input")
	}

	// Flatten deployment contents to nodes
	type elem struct{ ID, Label, Kind string }
	elems := make(map[string]elem)

	addInstance := func(ci *language.ContainerInstance) {
		label := ci.Label
		// Try to find container label in arch
		containerLabel := ""
		for _, s := range arch.Systems {
			for _, c := range s.Containers {
				if c.ID == ci.ContainerID {
					containerLabel = c.Label
				}
			}
		}
		if label == "" {
			if containerLabel != "" {
				label = containerLabel
			} else {
				label = ci.ContainerID
			}
		}
		elems[ci.ContainerID] = elem{ID: ci.ContainerID, Label: label, Kind: KindContainer}
	}

	var walk func(n *language.DeploymentNode)
	walk = func(n *language.DeploymentNode) {
		for _, ci := range n.ContainerInstances {
			addInstance(ci)
		}
		for _, infra := range n.Infrastructure {
			elems[infra.ID] = elem{ID: infra.ID, Label: infra.Label, Kind: "infra"}
		}
		for _, ch := range n.Children {
			walk(ch)
		}
	}
	walk(dep)

	// Build simple graph ordering for layout
	g := simple.NewDirectedGraph()
	idMap := make(map[string]int64)
	nextID := int64(1)
	add := func(name string) int64 {
		if id, ok := idMap[name]; ok {
			return id
		}
		id := nextID
		nextID++
		idMap[name] = id
		g.AddNode(simple.Node(id))
		return id
	}

	for id := range elems {
		add(id)
	}

	layers := longestPathLayers(g)
	layerNodes := make(map[int][]int64)
	for name, id := range idMap {
		_ = name
		l := layers[id]
		layerNodes[l] = append(layerNodes[l], id)
	}
	for l := range layerNodes {
		sort.Slice(layerNodes[l], func(i, j int) bool {
			a := layerNodes[l][i]
			b := layerNodes[l][j]
			return elems[nameByID(idMap, a)].Label < elems[nameByID(idMap, b)].Label
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
	if e.ShowTitle && arch.Name != "" {
		titleOffset = e.TitleFontSize + 10
	}
	contentTopMargin := e.TitleFontSize + 40
	width := e.Padding*2 + cols*e.NodeWidth + (cols-1)*e.HorizontalSpacing
	height := e.Padding*2 + maxRows*e.NodeHeight + (maxRows-1)*e.VerticalSpacing + titleOffset + contentTopMargin

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

	theme := e.Theme
	if theme == nil {
		theme = ProfessionalTheme()
	}

	// defs
	iconDefs := []IconDef{}
	for _, n := range []string{"server", "database", "inbox", "box"} {
		if c := icons.Get(n); c != "" {
			iconDefs = append(iconDefs, IconDef{Name: n, Content: c})
		}
	}
	defs, err := renderDefs(theme, iconDefs)
	if err != nil {
		return "", fmt.Errorf("failed to render defs: %w", err)
	}

	// nodes
	svgNodes := []Node{}
	for name, id := range idMap {
		el := elems[name]
		p := positions[id]
		node := Node{X: p.X, Y: p.Y, Width: e.NodeWidth, Height: e.NodeHeight, StrokeWidth: 2, LabelColor: theme.LabelColor, LabelSize: 13}
		switch el.Kind {
		case KindContainer:
			node.Fill = theme.ContainerFill
			node.Stroke = theme.ContainerStroke
			if shouldShowIcon(e) {
				node.Icon = "icon-server"
			}
		case "infra":
			node.Fill = theme.QueueFill
			node.Stroke = theme.QueueStroke
			if shouldShowIcon(e) {
				node.Icon = "icon-box"
			}
		default:
			node.Fill = theme.SystemFill
			node.Stroke = theme.SystemStroke
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
		svgNodes = append(svgNodes, node)
	}

	// group boundary for deployment over present elements; skip if only one
	groups := []Group{}
	present := 0
	minX, minY := 1<<30, 1<<30
	maxX, maxY := -1, -1
	for name := range elems {
		id := idMap[name]
		p := positions[id]
		present++
		if p.X < minX {
			minX = p.X
		}
		if p.Y < minY {
			minY = p.Y
		}
		if p.X+e.NodeWidth > maxX {
			maxX = p.X + e.NodeWidth
		}
		if p.Y+e.NodeHeight > maxY {
			maxY = p.Y + e.NodeHeight
		}
	}
	if present > 1 && minX < 1<<30 {
		pad := 16
		titleSpace := e.TitleFontSize + 20
		y0 := minY - pad - titleSpace
		minCanvasY := e.Padding + titleOffset + 8
		if y0 < minCanvasY {
			delta := minCanvasY - y0
			y0 = minCanvasY
			maxY += delta
		}
		groups = append(groups, Group{X: minX - pad, Y: y0, Width: (maxX - minX) + pad*2, Height: (maxY - minY) + pad*2 + titleSpace, Title: dep.Label, Stroke: theme.SystemStroke, Fill: "none", TitleColor: theme.TitleColor})
	}

	data := &TemplateData{
		Width:      width,
		Height:     height,
		Background: theme.Background,
		Theme:      theme,
		ShowTitle:  false,
		Title:      "",
		TitleX:     e.Padding,
		TitleY:     e.Padding + theme.TitleFontSize,
		Defs:       defs,
		Nodes:      svgNodes,
		Groups:     groups,
	}

	return renderSVG(data)
}

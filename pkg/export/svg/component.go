package svg

import (
	"fmt"
	"sort"
	"strings"

	"gonum.org/v1/gonum/graph/simple"

	"github.com/sruja-ai/sruja/pkg/export/svg/icons"
	"github.com/sruja-ai/sruja/pkg/language"
)

//nolint:funlen,gocyclo // Component export logic is detailed
func (e *Exporter) ExportContainerComponent(arch *language.Architecture, sys *language.System, cont *language.Container) (string, error) {
	if arch == nil || sys == nil || cont == nil {
		return "", fmt.Errorf("invalid input")
	}

	type elem struct{ ID, Label, Desc string }
	elems := make(map[string]elem)
	for _, c := range cont.Components {
		label := c.Label
		if label == "" {
			label = c.ID
		}
		desc := ""
		if c.Description != nil {
			desc = *c.Description
		}
		elems[c.ID] = elem{ID: c.ID, Label: label, Desc: desc}
	}

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

	edgeSet := make(map[string]bool)
	edgeLabel := make(map[string]string)
	edgeRel := make(map[string]*language.Relation)

	for _, rel := range sys.Relations {
		from := lastSegment(rel.From.String())
		to := lastSegment(rel.To.String())
		if _, ok := elems[from]; !ok {
			continue
		}
		if _, ok := elems[to]; !ok {
			continue
		}
		if from == to {
			continue
		}
		key := from + "=>" + to
		if !edgeSet[key] {
			edgeSet[key] = true
			edgeLabel[key] = relationLabel(rel)
			edgeRel[key] = rel
		}
		g.SetEdge(g.NewEdge(simple.Node(add(from)), simple.Node(add(to))))
	}

	for _, rel := range arch.Relations {
		from := lastSegment(rel.From.String())
		to := lastSegment(rel.To.String())
		if _, ok := elems[from]; !ok {
			continue
		}
		if _, ok := elems[to]; !ok {
			continue
		}
		if from == to {
			continue
		}
		key := from + "=>" + to
		if !edgeSet[key] {
			edgeSet[key] = true
			edgeLabel[key] = relationLabel(rel)
			edgeRel[key] = rel
		}
		g.SetEdge(g.NewEdge(simple.Node(add(from)), simple.Node(add(to))))
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
	width := e.Padding*2 + cols*e.NodeWidth + (cols-1)*e.HorizontalSpacing
	if e.ShowTitle && arch.Name != "" {
		titleOffset = e.TitleFontSize + 10
		// Estimate title width
		titleWidth := len(arch.Name)*12 + e.Padding*2
		if titleWidth > width {
			width = titleWidth
		}
	}
	contentTopMargin := e.TitleFontSize + 40
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
	iconDefs := []IconDef{}
	if c := icons.Get("box"); c != "" {
		iconDefs = append(iconDefs, IconDef{Name: "box", Content: c})
	}
	defs, err := renderDefs(theme, iconDefs)
	if err != nil {
		return "", fmt.Errorf("failed to render defs: %w", err)
	}

	svgEdges := []Edge{}
	for key := range edgeSet {
		parts := strings.Split(key, "=>")
		u := idMap[parts[0]]
		v := idMap[parts[1]]
		pu := positions[u]
		pv := positions[v]
		var x1, y1, x2, y2 int
		switch strings.ToUpper(e.Direction) {
		case "LR":
			x1 = pu.X + e.NodeWidth
			y1 = pu.Y + e.NodeHeight/2
			x2 = pv.X
			y2 = pv.Y + e.NodeHeight/2
		case "RL":
			x1 = pu.X
			y1 = pu.Y + e.NodeHeight/2
			x2 = pv.X + e.NodeWidth
			y2 = pv.Y + e.NodeHeight/2
		case "TB":
			x1 = pu.X + e.NodeWidth/2
			y1 = pu.Y + e.NodeHeight
			x2 = pv.X + e.NodeWidth/2
			y2 = pv.Y
		case "BT":
			x1 = pu.X + e.NodeWidth/2
			y1 = pu.Y
			x2 = pv.X + e.NodeWidth/2
			y2 = pv.Y + e.NodeHeight
		default:
			x1 = pu.X + e.NodeWidth
			y1 = pu.Y + e.NodeHeight/2
			x2 = pv.X
			y2 = pv.Y + e.NodeHeight/2
		}
		edge := Edge{X1: x1, Y1: y1, X2: x2, Y2: y2, Color: theme.EdgeColor, Width: theme.EdgeWidth}
		if theme.UseCurvedEdges && strings.EqualFold(e.Direction, "LR") {
			edge.Curved = true
			controlOffset := 40
			edge.CX1 = x1 + controlOffset
			edge.CY1 = y1
			edge.CX2 = x2 - controlOffset
			edge.CY2 = y2
		}
		lbl := strings.TrimSpace(edgeLabel[key])
		if lbl != "" {
			edge.Label = lbl
			edge.LabelX = (x1 + x2) / 2
			edge.LabelY = (y1 + y2) / 2
			edge.LabelBg = true
		}
		if rel, ok := edgeRel[key]; ok && rel.Label != nil {
			edge.Description = *rel.Label
		}
		edge.Title = fmt.Sprintf("%s to %s", elems[parts[0]].Label, elems[parts[1]].Label)
		svgEdges = append(svgEdges, edge)
	}

	svgNodes := []Node{}
	for name, id := range idMap {
		el := elems[name]
		p := positions[id]
		node := Node{X: p.X, Y: p.Y, Width: e.NodeWidth, Height: e.NodeHeight, Fill: theme.ContainerFill, Stroke: theme.ContainerStroke, StrokeWidth: 2, LabelColor: theme.LabelColor, LabelSize: 13}
		node.Label = el.Label
		node.LabelX = p.X + e.NodeWidth/2
		node.LabelY = p.Y + e.NodeHeight/2
		if shouldShowIcon(e) {
			node.Icon = IconBox
		}
		// icon placement
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
			labelOffsetX := iconPadding + iconTarget + 8
			node.LabelX = p.X + labelOffsetX
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
		node.Title = el.Label
		node.Description = el.Desc
		svgNodes = append(svgNodes, node)
	}

	bx := e.Padding / 2
	by := e.Padding + titleOffset
	bw := width - e.Padding
	bh := height - e.Padding*1 - titleOffset
	groups := []Group{{X: bx, Y: by, Width: bw, Height: bh, Title: cont.Label, Stroke: theme.ContainerStroke, Fill: "none", TitleColor: theme.TitleColor}}

	legendItems := []LegendItem{}
	if e.ShowLegend {
		legendItems = append(legendItems, LegendItem{Icon: "", Label: "Component", Stroke: theme.ContainerStroke, Fill: theme.ContainerFill})
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
		Edges:      svgEdges,
		Nodes:      svgNodes,
		Groups:     groups,
		Legend:     Legend{Show: e.ShowLegend, X: width - e.Padding - 210, Y: e.Padding, Items: legendItems, Title: "Legend"},
		Metadata:   e.Metadata,
		EmbedFonts: e.EmbedFonts,
	}

	return renderSVG(data)
}

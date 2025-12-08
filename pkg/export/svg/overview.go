package svg

import (
	"fmt"
	"sort"
	"strings"

	"gonum.org/v1/gonum/graph/simple"
	"gonum.org/v1/gonum/graph/topo"

	"github.com/sruja-ai/sruja/pkg/export/svg/icons"
	"github.com/sruja-ai/sruja/pkg/language"
)

//nolint:funlen,gocyclo // Overview export logic is extensive
func (e *Exporter) Export(arch *language.Architecture) (string, error) {
	if arch == nil {
		return "", fmt.Errorf("architecture is nil")
	}
	dir := e.Direction
	if arch.Style != nil {
		if d, ok := arch.Style["svg_direction"]; ok && d != "" {
			dir = strings.ToUpper(d)
		}
	}
	for _, m := range arch.Metadata {
		if strings.EqualFold(m.Key, "svg_direction") && m.Value != nil && *m.Value != "" {
			dir = strings.ToUpper(*m.Value)
		}
	}
	e.Direction = dir

	nodes := make(map[string]*node)
	for _, p := range arch.Persons {
		label := p.Label
		if label == "" {
			label = p.ID
		}
		nodes[p.ID] = &node{ID: p.ID, Label: label, Kind: "person"}
	}
	for _, s := range arch.Systems {
		label := s.Label
		if label == "" {
			label = s.ID
		}
		nodes[s.ID] = &node{ID: s.ID, Label: label, Kind: "system"}
	}
	desc := map[string]string{}
	for _, p := range arch.Persons {
		if p.Description != nil {
			desc[p.ID] = *p.Description
		}
	}
	for _, s := range arch.Systems {
		if s.Description != nil {
			desc[s.ID] = *s.Description
		}
	}

	contToSys := buildContainerSystemMap(arch)
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

	edgeSet := make(map[string]bool)
	edgeLabel := make(map[string]string)
	edgeRel := make(map[string]*language.Relation)
	for _, rel := range arch.Relations {
		fromTop, okFrom := resolveOverviewNode(arch, contToSys, rel.From.String())
		toTop, okTo := resolveOverviewNode(arch, contToSys, rel.To.String())
		if !okFrom || !okTo {
			continue
		}
		if fromTop == toTop {
			continue
		}
		key := fromTop + "=>" + toTop
		if !edgeSet[key] {
			edgeSet[key] = true
			lbl := ""
			if rel.Verb != nil {
				lbl = *rel.Verb
			} else if rel.Label != nil {
				lbl = *rel.Label
			}
			edgeLabel[key] = lbl
			edgeRel[key] = rel
		}
		u := addNode(fromTop)
		v := addNode(toTop)
		g.SetEdge(g.NewEdge(simple.Node(u), simple.Node(v)))
		if _, ok := nodes[fromTop]; !ok {
			nodes[fromTop] = &node{ID: fromTop, Label: fromTop, Kind: kindOfTop(arch, fromTop)}
		}
		if _, ok := nodes[toTop]; !ok {
			nodes[toTop] = &node{ID: toTop, Label: toTop, Kind: kindOfTop(arch, toTop)}
		}
	}
	for id := range nodes {
		addNode(id)
	}

	order, err := topo.Sort(g)
	layers := make(map[int64]int)
	if err == nil {
		for _, n := range order {
			maxPred := 0
			it := g.To(n.ID())
			for it.Next() {
				p := it.Node()
				if lp := layers[p.ID()] + 1; lp > maxPred {
					maxPred = lp
				}
			}
			layers[n.ID()] = maxPred
		}
	} else {
		zeroIn := make([]int64, 0)
		for _, id := range idMap {
			if g.To(id).Len() == 0 {
				zeroIn = append(zeroIn, id)
			}
		}
		if len(zeroIn) == 0 {
			for _, id := range idMap {
				zeroIn = append(zeroIn, id)
				break
			}
		}
		visited := make(map[int64]bool)
		queue := make([]int64, 0)
		queue = append(queue, zeroIn...)
		for _, id := range zeroIn {
			layers[id] = 0
			visited[id] = true
		}
		for len(queue) > 0 {
			u := queue[0]
			queue = queue[1:]
			it := g.From(u)
			for it.Next() {
				v := it.Node()
				if lv := layers[u] + 1; lv > layers[v.ID()] {
					layers[v.ID()] = lv
				}
				if !visited[v.ID()] {
					visited[v.ID()] = true
					queue = append(queue, v.ID())
				}
			}
		}
	}

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
	width := e.Padding*2 + cols*e.NodeWidth + (cols-1)*e.HorizontalSpacing
	titleOffset := 0
	if e.ShowTitle && arch.Name != "" {
		titleOffset = e.TitleFontSize + 10
		// Estimate title width
		titleWidth := len(arch.Name)*12 + e.Padding*2
		if titleWidth > width {
			width = titleWidth
		}
	}
	height := e.Padding*2 + maxRows*e.NodeHeight + (maxRows-1)*e.VerticalSpacing + titleOffset

	positions := make(map[int64]point)
	for l := 0; l <= maxLayer; l++ {
		ids := layerNodes[l]
		for i, id := range ids {
			var x, y int
			switch strings.ToUpper(e.Direction) {
			case "LR":
				x = e.Padding + l*(e.NodeWidth+e.HorizontalSpacing)
				y = e.Padding + titleOffset + i*(e.NodeHeight+e.VerticalSpacing)
			case "RL":
				x = e.Padding + (cols-1-l)*(e.NodeWidth+e.HorizontalSpacing)
				y = e.Padding + titleOffset + i*(e.NodeHeight+e.VerticalSpacing)
			case "TB":
				x = e.Padding + i*(e.NodeWidth+e.HorizontalSpacing)
				y = e.Padding + titleOffset + l*(e.NodeHeight+e.VerticalSpacing)
			case "BT":
				x = e.Padding + i*(e.NodeWidth+e.HorizontalSpacing)
				y = e.Padding + titleOffset + (cols-1-l)*(e.NodeHeight+e.VerticalSpacing)
			default:
				x = e.Padding + l*(e.NodeWidth+e.HorizontalSpacing)
				y = e.Padding + titleOffset + i*(e.NodeHeight+e.VerticalSpacing)
			}
			positions[id] = point{X: x, Y: y}
		}
	}

	theme := e.Theme
	if theme == nil {
		theme = ProfessionalTheme()
	}

	iconDefs := []IconDef{}
	need := map[string]bool{"user": false, "home": false, "server": false, "database": false, "inbox": false}
	for name := range idMap {
		switch nodes[name].Kind {
		case KindPerson:
			need["user"] = true
		case KindSystem:
			need["home"] = true
		}
	}
	for iconName, ok := range need {
		if ok {
			if content := icons.Get(iconName); content != "" {
				iconDefs = append(iconDefs, IconDef{Name: iconName, Content: content})
			}
		}
	}
	defs, err := renderDefs(theme, iconDefs)
	if err != nil {
		return "", fmt.Errorf("failed to render defs: %w", err)
	}

	gridLines := []GridLine{}
	if e.ShowGrid {
		gridSize := 20
		for x := 0; x < width; x += gridSize {
			gridLines = append(gridLines, GridLine{X1: x, Y1: 0, X2: x, Y2: height, Color: theme.GridColor, Opacity: 0.3})
		}
		for y := 0; y < height; y += gridSize {
			gridLines = append(gridLines, GridLine{X1: 0, Y1: y, X2: width, Y2: y, Color: theme.GridColor, Opacity: 0.3})
		}
	}

	svgEdges := []Edge{}
	for key := range edgeSet {
		parts := strings.Split(key, "=>")
		from := parts[0]
		to := parts[1]
		u := idMap[from]
		v := idMap[to]
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
			controlOffset := 40
			edge.Curved = true
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
		edge.Title = fmt.Sprintf("%s to %s", nodes[from].Label, nodes[to].Label)
		svgEdges = append(svgEdges, edge)
	}

	svgNodes := []Node{}
	for name, id := range idMap {
		n := nodes[name]
		p := positions[id]
		var fill, stroke, gradientID string
		switch n.Kind {
		case KindPerson:
			fill = theme.PersonFill
			stroke = theme.PersonStroke
			if theme.UseGradients {
				gradientID = "url(#grad-person)"
			}
		case KindSystem:
			fill = theme.SystemFill
			stroke = theme.SystemStroke
			if theme.UseGradients {
				gradientID = "url(#grad-system)"
			}
		default:
			fill = "#eeeeee"
			stroke = "#666"
		}
		node := Node{
			X: p.X, Y: p.Y,
			Width: e.NodeWidth, Height: e.NodeHeight,
			Fill: fill, Stroke: stroke,
			StrokeWidth: 2.5,
			Gradient:    gradientID,
			Label:       n.Label,
			LabelX:      p.X + e.NodeWidth/2,
			LabelY:      p.Y + e.NodeHeight/2,
			LabelColor:  theme.TitleColor,
			LabelSize:   13,
		}
		if theme.UseShadows {
			node.Filter = "url(#shadow)"
		}
		if n.Kind == KindExternal {
			node.Dash = "stroke-dasharray=\"6 4\""
		}
		var iconRef string
		switch n.Kind {
		case KindPerson:
			iconRef = "icon-user"
		case KindSystem:
			iconRef = "icon-home"
		}
		if iconRef != "" && shouldShowIcon(e) {
			node.Icon = iconRef
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
			node.LabelY = p.Y + e.NodeHeight/2
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
		node.Title = n.Label
		node.Description = desc[name]
		svgNodes = append(svgNodes, node)
	}

	legendItems := []LegendItem{}
	if e.ShowLegend {
		hasPerson := false
		hasSystem := false
		for _, n := range nodes {
			if n.Kind == "person" {
				hasPerson = true
			}
			if n.Kind == "system" {
				hasSystem = true
			}
		}
		if hasPerson {
			legendItems = append(legendItems, LegendItem{Icon: "icon-user", Label: "Person", Stroke: theme.PersonStroke, Fill: theme.PersonFill})
		}
		if hasSystem {
			legendItems = append(legendItems, LegendItem{Icon: "icon-home", Label: "System", Stroke: theme.SystemStroke, Fill: theme.SystemFill})
		}
	}

	data := &TemplateData{
		Width:      width,
		Height:     height,
		Background: theme.Background,
		Theme:      theme,
		ShowGrid:   e.ShowGrid,
		GridColor:  theme.GridColor,
		GridSize:   20,
		ShowTitle:  e.ShowTitle && arch.Name != "",
		Title:      arch.Name,
		TitleX:     e.Padding,
		TitleY:     e.Padding + theme.TitleFontSize,
		Defs:       defs,
		GridLines:  gridLines,
		Edges:      svgEdges,
		Nodes:      svgNodes,
		Legend:     Legend{Show: e.ShowLegend, X: width - e.Padding - 210, Y: e.Padding, Items: legendItems, Title: "Legend"},
		Metadata:   e.Metadata,
		EmbedFonts: e.EmbedFonts,
	}

	return renderSVG(data)
}

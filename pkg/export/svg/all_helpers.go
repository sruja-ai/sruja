package svg

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

//nolint:gocritic // Results are map sets, naming them adds little value
func collectMetadata(arch *language.Architecture) (map[string]string, map[string]string, map[string]string) {
	kind := map[string]string{}
	label := map[string]string{}
	desc := map[string]string{}

	for _, p := range arch.Persons {
		kind[p.ID] = KindPerson
		label[p.ID] = firstNonEmpty(p.Label, p.ID)
		desc[p.ID] = deref(p.Description)
	}
	for _, s := range arch.Systems {
		kind[s.ID] = KindSystem
		label[s.ID] = firstNonEmpty(s.Label, s.ID)
		desc[s.ID] = deref(s.Description)

		for _, c := range s.Containers {
			kind[c.ID] = KindContainer
			label[c.ID] = firstNonEmpty(c.Label, c.ID)
			desc[c.ID] = deref(c.Description)
			for _, comp := range c.Components {
				kind[comp.ID] = KindComponent
				label[comp.ID] = firstNonEmpty(comp.Label, comp.ID)
				desc[comp.ID] = deref(comp.Description)
			}
		}
		for _, ds := range s.DataStores {
			kind[ds.ID] = KindDatabase
			label[ds.ID] = firstNonEmpty(ds.Label, ds.ID)
			desc[ds.ID] = deref(ds.Description)
		}
		for _, q := range s.Queues {
			kind[q.ID] = KindQueue
			label[q.ID] = firstNonEmpty(q.Label, q.ID)
			desc[q.ID] = deref(q.Description)
		}
	}
	return kind, label, desc
}

//nolint:gocritic // Return signature is clear enough for internal helper
func (e *Exporter) layoutSystemInternals(s *language.System, arch *language.Architecture) (layoutNode, map[string]point) {
	internalNodes := []layoutNode{}
	ids := []string{}

	// Collect internal elements
	for _, c := range s.Containers {
		ids = append(ids, c.ID)
		internalNodes = append(internalNodes, layoutNode{ID: c.ID, Width: e.NodeWidth, Height: e.NodeHeight})
		for _, comp := range c.Components {
			ids = append(ids, comp.ID)
			internalNodes = append(internalNodes, layoutNode{ID: comp.ID, Width: e.NodeWidth, Height: e.NodeHeight})
		}
	}
	for _, ds := range s.DataStores {
		ids = append(ids, ds.ID)
		internalNodes = append(internalNodes, layoutNode{ID: ds.ID, Width: e.NodeWidth, Height: e.NodeHeight})
	}
	for _, q := range s.Queues {
		ids = append(ids, q.ID)
		internalNodes = append(internalNodes, layoutNode{ID: q.ID, Width: e.NodeWidth, Height: e.NodeHeight})
	}

	// Collect internal edges
	internalEdges := []edgePair{}
	idSet := make(map[string]bool)
	for _, id := range ids {
		idSet[id] = true
	}

	for _, rel := range arch.Relations {
		from := lastSegment(rel.From.String())
		to := lastSegment(rel.To.String())
		if idSet[from] && idSet[to] && from != to {
			internalEdges = append(internalEdges, edgePair{From: from, To: to})
		}
	}

	// Layout internals
	res := layoutCluster(internalNodes, internalEdges, layoutConfig{
		Direction:         e.Direction,
		HorizontalSpacing: e.HorizontalSpacing,
		VerticalSpacing:   e.VerticalSpacing,
		Padding:           e.Padding,
	})

	// Calculate system size including title and padding
	titleHeight := e.TitleFontSize + 20
	width := res.Width + 32 // + padding
	height := res.Height + 32 + titleHeight

	// Ensure minimum size
	if width < e.NodeWidth {
		width = e.NodeWidth
	}
	if height < e.NodeHeight {
		height = e.NodeHeight
	}

	return layoutNode{ID: s.ID, Width: width, Height: height}, res.Positions
}

//nolint:gocyclo,gocritic // Return signature is clear enough for internal helper
func (e *Exporter) determineTopLevelNodes(arch *language.Architecture, sysDims map[string]layoutNode, kind map[string]string) ([]layoutNode, map[string]string, map[string]string) {
	topNodes := []layoutNode{}
	kindMap := kind // copies map reference? no, maps are references. we should probably modify inplace or be careful.
	// Actually we want to modify kind so we can track externals.
	labelMap := map[string]string{}

	for _, p := range arch.Persons {
		topNodes = append(topNodes, layoutNode{ID: p.ID, Width: e.NodeWidth, Height: e.NodeHeight})
	}
	for _, s := range arch.Systems {
		topNodes = append(topNodes, sysDims[s.ID])
	}

	// Include external nodes referenced in relations
	findTop := func(id string) string {
		for _, s := range arch.Systems {
			if s.ID == id {
				return s.ID
			}
			for _, c := range s.Containers {
				if c.ID == id {
					return s.ID
				}
				for _, comp := range c.Components {
					if comp.ID == id {
						return s.ID
					}
				}
			}
			for _, ds := range s.DataStores {
				if ds.ID == id {
					return s.ID
				}
			}
			for _, q := range s.Queues {
				if q.ID == id {
					return s.ID
				}
			}
		}
		return id // Person or External
	}

	knownTopNodes := make(map[string]bool)
	for _, n := range topNodes {
		knownTopNodes[n.ID] = true
	}

	for _, rel := range arch.Relations {
		from := lastSegment(rel.From.String())
		to := lastSegment(rel.To.String())
		topFrom := findTop(from)
		topTo := findTop(to)

		if !knownTopNodes[topFrom] {
			topNodes = append(topNodes, layoutNode{ID: topFrom, Width: e.NodeWidth, Height: e.NodeHeight})
			knownTopNodes[topFrom] = true
			kindMap[topFrom] = KindExternal
			labelMap[topFrom] = topFrom
		}
		if !knownTopNodes[topTo] {
			topNodes = append(topNodes, layoutNode{ID: topTo, Width: e.NodeWidth, Height: e.NodeHeight})
			knownTopNodes[topTo] = true
			kindMap[topTo] = KindExternal
			labelMap[topTo] = topTo
		}
	}
	return topNodes, kindMap, labelMap
}

func collectTopLevelEdges(arch *language.Architecture, _ []layoutNode) []edgePair {
	topEdges := []edgePair{}

	// Helper to find top-level parent (duplicated logic, should extract?)
	findTop := func(id string) string {
		for _, s := range arch.Systems {
			if s.ID == id {
				return s.ID
			}
			for _, c := range s.Containers {
				if c.ID == id {
					return s.ID
				}
				for _, comp := range c.Components {
					if comp.ID == id {
						return s.ID
					}
				}
			}
			for _, ds := range s.DataStores {
				if ds.ID == id {
					return s.ID
				}
			}
			for _, q := range s.Queues {
				if q.ID == id {
					return s.ID
				}
			}
		}
		return id
	}

	topEdgeSet := make(map[string]bool)
	for _, rel := range arch.Relations {
		from := lastSegment(rel.From.String())
		to := lastSegment(rel.To.String())
		topFrom := findTop(from)
		topTo := findTop(to)

		if topFrom != topTo {
			key := topFrom + "=>" + topTo
			if !topEdgeSet[key] {
				topEdgeSet[key] = true
				topEdges = append(topEdges, edgePair{From: topFrom, To: topTo})
			}
		}
	}
	return topEdges
}

//nolint:funlen,gocyclo // Edge collection logic is complex
func (e *Exporter) collectSVGEdges(arch *language.Architecture, positions map[string]point, sysDims map[string]layoutNode, label map[string]string, theme *Theme) []Edge {
	svgEdges := []Edge{}
	edgeSet := make(map[string]bool)

	for _, rel := range arch.Relations {
		from := lastSegment(rel.From.String())
		to := lastSegment(rel.To.String())
		if from == to {
			continue
		}

		if _, ok1 := positions[from]; !ok1 {
			continue
		}
		if _, ok2 := positions[to]; !ok2 {
			continue
		}

		key := from + "=>" + to
		if !edgeSet[key] {
			edgeSet[key] = true

			pu := positions[from]
			pv := positions[to]

			var x1, y1, x2, y2 int
			w1, h1 := e.NodeWidth, e.NodeHeight
			w2, h2 := e.NodeWidth, e.NodeHeight

			if dim, ok := sysDims[from]; ok {
				w1, h1 = dim.Width, dim.Height
			}
			if dim, ok := sysDims[to]; ok {
				w2, h2 = dim.Width, dim.Height
			}

			switch strings.ToUpper(e.Direction) {
			case DirLR:
				x1 = pu.X + w1
				y1 = pu.Y + h1/2
				x2 = pv.X
				y2 = pv.Y + h2/2
			case DirRL:
				x1 = pu.X
				y1 = pu.Y + h1/2
				x2 = pv.X + w2
				y2 = pv.Y + h2/2
			case DirTB:
				x1 = pu.X + w1/2
				y1 = pu.Y + h1
				x2 = pv.X + w2/2
				y2 = pv.Y
			case DirBT:
				x1 = pu.X + w1/2
				y1 = pu.Y
				x2 = pv.X + w2/2
				y2 = pv.Y + h2
			default:
				x1 = pu.X + w1
				y1 = pu.Y + h1/2
				x2 = pv.X
				y2 = pv.Y + h2/2
			}

			edge := Edge{X1: x1, Y1: y1, X2: x2, Y2: y2, Color: theme.EdgeColor, Width: theme.EdgeWidth}
			if theme.UseCurvedEdges && strings.EqualFold(e.Direction, DirLR) {
				edge.Curved = true
				controlOffset := 40
				edge.CX1 = x1 + controlOffset
				edge.CY1 = y1
				edge.CX2 = x2 - controlOffset
				edge.CY2 = y2
			}
			lbl := relationLabel(rel)
			if lbl != "" {
				edge.Label = lbl
				edge.LabelX = (x1 + x2) / 2
				edge.LabelY = (y1 + y2) / 2
				edge.LabelBg = true
			}
			if rel.Label != nil {
				edge.Description = *rel.Label
			}
			edge.Title = fmt.Sprintf("%s to %s", label[from], label[to])
			svgEdges = append(svgEdges, edge)
		}
	}
	return svgEdges
}

//nolint:funlen,gocritic // Node collection logic is complex
func (e *Exporter) collectSVGNodes(positions map[string]point, sysDims map[string]layoutNode, kind map[string]string, label map[string]string, desc map[string]string, theme *Theme) []Node {
	svgNodes := []Node{}
	for name, p := range positions {
		if _, isSystem := sysDims[name]; isSystem {
			continue
		}

		k := kind[name]
		node := Node{X: p.X, Y: p.Y, Width: e.NodeWidth, Height: e.NodeHeight, StrokeWidth: 2, LabelColor: theme.LabelColor, LabelSize: 13}
		switch k {
		case KindPerson:
			node.Fill = theme.PersonFill
			node.Stroke = theme.PersonStroke
			node.Icon = IconUser
		case KindContainer:
			node.Fill = theme.ContainerFill
			node.Stroke = theme.ContainerStroke
			node.Icon = IconServer
		case KindDatabase:
			node.Fill = theme.DatabaseFill
			node.Stroke = theme.DatabaseStroke
			node.Icon = IconDatabase
		case KindQueue:
			node.Fill = theme.QueueFill
			node.Stroke = theme.QueueStroke
			node.Icon = IconInbox
		case KindComponent:
			node.Fill = theme.ContainerFill
			node.Stroke = theme.ContainerStroke
			node.Icon = IconBox
		default:
			node.Fill = ColorDefaultFill
			node.Stroke = ColorDefaultStroke
		}
		node.Label = label[name]
		node.LabelX = p.X + e.NodeWidth/2
		node.LabelY = p.Y + e.NodeHeight/2

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
		node.Title = node.Label
		node.Description = desc[name]
		svgNodes = append(svgNodes, node)
	}
	return svgNodes
}

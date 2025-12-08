package svg

import (
	"sort"
	"strings"

	"gonum.org/v1/gonum/graph/simple"
	"gonum.org/v1/gonum/graph/topo"
)

type layoutNode struct {
	ID     string
	Width  int
	Height int
}

type edgePair struct {
	From, To string
}

type layoutResult struct {
	Width     int
	Height    int
	Positions map[string]point
}

type layoutConfig struct {
	Direction         string
	HorizontalSpacing int
	VerticalSpacing   int
	Padding           int
}

//nolint:funlen,gocyclo // Layout logic is complex
func layoutCluster(nodes []layoutNode, edges []edgePair, config layoutConfig) layoutResult {
	if len(nodes) == 0 {
		return layoutResult{Width: 0, Height: 0, Positions: make(map[string]point)}
	}

	g := simple.NewDirectedGraph()
	idMap := make(map[string]int64)
	nameMap := make(map[int64]string)
	nodeMap := make(map[string]layoutNode)
	nextID := int64(1)

	for _, n := range nodes {
		id := nextID
		nextID++
		idMap[n.ID] = id
		nameMap[id] = n.ID
		nodeMap[n.ID] = n
		g.AddNode(simple.Node(id))
	}

	for _, e := range edges {
		u, okU := idMap[e.From]
		v, okV := idMap[e.To]
		if okU && okV && u != v {
			g.SetEdge(g.NewEdge(simple.Node(u), simple.Node(v)))
		}
	}

	layers := longestPathLayers(g)
	layerNodes := make(map[int][]int64)
	for id, l := range layers {
		layerNodes[l] = append(layerNodes[l], id)
	}

	// Initial sort by name for stability
	for l := range layerNodes {
		sort.Slice(layerNodes[l], func(i, j int) bool {
			return nameMap[layerNodes[l][i]] < nameMap[layerNodes[l][j]]
		})
	}

	maxLayer := 0
	for l := range layerNodes {
		if l > maxLayer {
			maxLayer = l
		}
	}

	// Barycenter heuristic to minimize crossings
	// We run a few passes of up/down sweeps
	iterations := 8
	for iter := 0; iter < iterations; iter++ {
		// Down sweep (1 -> maxLayer)
		for l := 1; l <= maxLayer; l++ {
			// For each node in current layer, calculate avg index of parents in prev layer
			type nodeVal struct {
				id  int64
				val float64
			}
			vals := []nodeVal{}

			// Map of parent ID to its index in previous layer
			prevLayerIdx := make(map[int64]int)
			for i, id := range layerNodes[l-1] {
				prevLayerIdx[id] = i
			}

			for _, id := range layerNodes[l] {
				sum := 0.0
				count := 0.0
				it := g.To(id)
				for it.Next() {
					pid := it.Node().ID()
					if idx, ok := prevLayerIdx[pid]; ok {
						sum += float64(idx)
						count++
					}
				}
				val := 0.0
				if count > 0 {
					val = sum / count
				} else {
					// Keep relative order if no parents in prev layer
					// Use current index as fallback
					for i, currID := range layerNodes[l] {
						if currID == id {
							val = float64(i)
							break
						}
					}
				}
				vals = append(vals, nodeVal{id: id, val: val})
			}

			sort.Slice(vals, func(i, j int) bool {
				if vals[i].val != vals[j].val {
					return vals[i].val < vals[j].val
				}
				return nameMap[vals[i].id] < nameMap[vals[j].id]
			})

			// Update layer
			newLayer := make([]int64, len(vals))
			for i, v := range vals {
				newLayer[i] = v.id
			}
			layerNodes[l] = newLayer
		}

		// Up sweep (maxLayer-1 -> 0)
		for l := maxLayer - 1; l >= 0; l-- {
			type nodeVal struct {
				id  int64
				val float64
			}
			vals := []nodeVal{}

			// Map of child ID to its index in next layer
			nextLayerIdx := make(map[int64]int)
			for i, id := range layerNodes[l+1] {
				nextLayerIdx[id] = i
			}

			for _, id := range layerNodes[l] {
				sum := 0.0
				count := 0.0
				it := g.From(id)
				for it.Next() {
					cid := it.Node().ID()
					if idx, ok := nextLayerIdx[cid]; ok {
						sum += float64(idx)
						count++
					}
				}
				val := 0.0
				if count > 0 {
					val = sum / count
				} else {
					for i, currID := range layerNodes[l] {
						if currID == id {
							val = float64(i)
							break
						}
					}
				}
				vals = append(vals, nodeVal{id: id, val: val})
			}

			sort.Slice(vals, func(i, j int) bool {
				if vals[i].val != vals[j].val {
					return vals[i].val < vals[j].val
				}
				return nameMap[vals[i].id] < nameMap[vals[j].id]
			})

			// Update layer
			newLayer := make([]int64, len(vals))
			for i, v := range vals {
				newLayer[i] = v.id
			}
			layerNodes[l] = newLayer
		}
	}

	// Calculate grid cell dimensions via colWidths/rowHeights

	// For grid calculation
	colWidths := make(map[int]int)
	rowHeights := make(map[int]int)

	// We need to map layers/indices to grid coordinates based on direction
	// But simple grid approach:
	// LR: layer = col, index in layer = row
	// TB: layer = row, index in layer = col

	isHorizontal := strings.EqualFold(config.Direction, "LR") || strings.EqualFold(config.Direction, "RL")

	// First pass: determine max dimensions for each row/col
	for l := 0; l <= maxLayer; l++ {
		ids := layerNodes[l]
		for i, id := range ids {
			n := nodeMap[nameMap[id]]
			if isHorizontal {
				// layer is col, i is row
				if n.Width > colWidths[l] {
					colWidths[l] = n.Width
				}
				if n.Height > rowHeights[i] {
					rowHeights[i] = n.Height
				}
			} else {
				// layer is row, i is col
				if n.Height > rowHeights[l] {
					rowHeights[l] = n.Height
				}
				if n.Width > colWidths[i] {
					colWidths[i] = n.Width
				}
			}
		}
	}

	positions := make(map[string]point)

	// Calculate positions
	totalWidth := 0
	totalHeight := 0

	if isHorizontal {
		// Calculate X positions for columns
		colX := make(map[int]int)
		currentX := config.Padding
		for l := 0; l <= maxLayer; l++ {
			colX[l] = currentX
			currentX += colWidths[l] + config.HorizontalSpacing
		}
		totalWidth = currentX - config.HorizontalSpacing + config.Padding

		// Calculate Y positions for rows
		rowY := make(map[int]int)
		currentY := config.Padding
		maxRow := 0
		for r := range rowHeights {
			if r > maxRow {
				maxRow = r
			}
		}
		for r := 0; r <= maxRow; r++ {
			rowY[r] = currentY
			currentY += rowHeights[r] + config.VerticalSpacing
		}
		totalHeight = currentY - config.VerticalSpacing + config.Padding

		// Assign positions
		for l := 0; l <= maxLayer; l++ {
			ids := layerNodes[l]
			for i, id := range ids {
				n := nodeMap[nameMap[id]]
				// Center in cell
				x := colX[l] + (colWidths[l]-n.Width)/2
				y := rowY[i] + (rowHeights[i]-n.Height)/2

				// Handle RL direction
				if strings.EqualFold(config.Direction, "RL") {
					x = totalWidth - config.Padding - (colX[l] - config.Padding) - colWidths[l] + (colWidths[l]-n.Width)/2
				}

				positions[nameMap[id]] = point{X: x, Y: y}
			}
		}
	} else {
		// Vertical (TB/BT)
		// Calculate Y positions for rows (layers)
		rowY := make(map[int]int)
		currentY := config.Padding
		for l := 0; l <= maxLayer; l++ {
			rowY[l] = currentY
			currentY += rowHeights[l] + config.VerticalSpacing
		}
		totalHeight = currentY - config.VerticalSpacing + config.Padding

		// Calculate X positions for columns (indices)
		colX := make(map[int]int)
		currentX := config.Padding
		maxCol := 0
		for c := range colWidths {
			if c > maxCol {
				maxCol = c
			}
		}
		for c := 0; c <= maxCol; c++ {
			colX[c] = currentX
			currentX += colWidths[c] + config.HorizontalSpacing
		}
		totalWidth = currentX - config.HorizontalSpacing + config.Padding

		// Assign positions
		for l := 0; l <= maxLayer; l++ {
			ids := layerNodes[l]
			for i, id := range ids {
				n := nodeMap[nameMap[id]]
				// Center in cell
				x := colX[i] + (colWidths[i]-n.Width)/2
				y := rowY[l] + (rowHeights[l]-n.Height)/2

				// Handle BT direction
				if strings.EqualFold(config.Direction, "BT") {
					y = totalHeight - config.Padding - (rowY[l] - config.Padding) - rowHeights[l] + (rowHeights[l]-n.Height)/2
				}

				positions[nameMap[id]] = point{X: x, Y: y}
			}
		}
	}

	return layoutResult{Width: totalWidth, Height: totalHeight, Positions: positions}
}

func longestPathLayers(g *simple.DirectedGraph) map[int64]int {
	layers := make(map[int64]int)
	order, err := topo.Sort(g)
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
		return layers
	}
	visited := make(map[int64]bool)
	queue := make([]int64, 0)
	zeroIn := make([]int64, 0)
	nodes := g.Nodes()
	for nodes.Next() {
		n := nodes.Node()
		if g.To(n.ID()).Len() == 0 {
			zeroIn = append(zeroIn, n.ID())
		}
	}
	if len(zeroIn) == 0 {
		nodes = g.Nodes()
		if nodes.Next() {
			zeroIn = append(zeroIn, nodes.Node().ID())
		}
	}
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
	return layers
}

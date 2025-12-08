package svg

import (
	"testing"
)

func TestLayoutCluster(t *testing.T) {
	tests := []struct {
		name     string
		nodes    []layoutNode
		edges    []edgePair
		config   layoutConfig
		wantDims bool // just check if width/height > 0
		wantErr  bool
	}{
		{
			name:   "Empty",
			nodes:  []layoutNode{},
			edges:  []edgePair{},
			config: layoutConfig{Direction: "TB", Padding: 10},
		},
		{
			name: "Single Node",
			nodes: []layoutNode{
				{ID: "A", Width: 100, Height: 50},
			},
			edges:    []edgePair{},
			config:   layoutConfig{Direction: "TB", Padding: 10},
			wantDims: true,
		},
		{
			name: "Simple Pair TB",
			nodes: []layoutNode{
				{ID: "A", Width: 100, Height: 50},
				{ID: "B", Width: 100, Height: 50},
			},
			edges: []edgePair{
				{From: "A", To: "B"},
			},
			config:   layoutConfig{Direction: "TB", Padding: 10, VerticalSpacing: 20},
			wantDims: true,
		},
		{
			name: "Disconnected",
			nodes: []layoutNode{
				{ID: "A", Width: 100, Height: 50},
				{ID: "B", Width: 100, Height: 50},
			},
			edges:    []edgePair{},
			config:   layoutConfig{Direction: "TB", Padding: 10},
			wantDims: true,
		},
		{
			name: "Cycle",
			nodes: []layoutNode{
				{ID: "A", Width: 100, Height: 50},
				{ID: "B", Width: 100, Height: 50},
			},
			edges: []edgePair{
				{From: "A", To: "B"},
				{From: "B", To: "A"}, // Cycle should be handled gracefully
			},
			config:   layoutConfig{Direction: "TB", Padding: 10},
			wantDims: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			res := layoutCluster(tt.nodes, tt.edges, tt.config)

			// Verify all nodes have positions
			if len(res.Positions) != len(tt.nodes) {
				t.Errorf("Expected %d positions, got %d", len(tt.nodes), len(res.Positions))
			}

			for _, n := range tt.nodes {
				if _, ok := res.Positions[n.ID]; !ok {
					t.Errorf("Node %s missing from positions", n.ID)
				}
			}

			if tt.wantDims {
				if res.Width <= 0 || res.Height <= 0 {
					t.Errorf("Expected positive dimensions, got %dx%d", res.Width, res.Height)
				}
			}
		})
	}
}

func TestLayoutCluster_Direction(t *testing.T) {
	nodes := []layoutNode{
		{ID: "A", Width: 100, Height: 50},
		{ID: "B", Width: 100, Height: 50},
	}
	edges := []edgePair{{From: "A", To: "B"}}

	// Test TB
	resTB := layoutCluster(nodes, edges, layoutConfig{Direction: "TB", VerticalSpacing: 50, Padding: 0})
	posATB := resTB.Positions["A"]
	posBTB := resTB.Positions["B"]
	if posBTB.Y <= posATB.Y {
		t.Error("TB: B should be below A")
	}

	// Test BT
	resBT := layoutCluster(nodes, edges, layoutConfig{Direction: "BT", VerticalSpacing: 50, Padding: 0})
	posABT := resBT.Positions["A"]
	posBBT := resBT.Positions["B"]
	if posBBT.Y >= posABT.Y {
		t.Error("BT: B should be above A")
	}

	// Test LR
	resLR := layoutCluster(nodes, edges, layoutConfig{Direction: "LR", HorizontalSpacing: 50, Padding: 0})
	posALR := resLR.Positions["A"]
	posBLR := resLR.Positions["B"]
	if posBLR.X <= posALR.X {
		t.Error("LR: B should be to the right of A")
	}

	// Test RL
	resRL := layoutCluster(nodes, edges, layoutConfig{Direction: "RL", HorizontalSpacing: 50, Padding: 0})
	posARL := resRL.Positions["A"]
	posBRL := resRL.Positions["B"]
	if posBRL.X >= posARL.X {
		t.Error("RL: B should be to the left of A")
	}
}

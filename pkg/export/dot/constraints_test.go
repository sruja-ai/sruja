package dot

import (
	"testing"
)

func TestBuildConstraints(t *testing.T) {
	config := DefaultConfig()

	t.Run("Empty Graph", func(t *testing.T) {
		constraints := BuildConstraints(nil, nil, 1, config)
		if constraints.Global.NodeSep == 0 {
			t.Error("Expected non-zero NodeSep even for empty graph")
		}
	})

	t.Run("L1 Context View Spacing", func(t *testing.T) {
		elements := []*Element{
			{ID: "A", Kind: "person"},
			{ID: "B", Kind: "system"},
		}
		constraints := BuildConstraints(elements, nil, 1, config)

		expectedNodeSep := pxToInchFloat(float64(config.NodeSep)) * L1NodeSepScale
		if constraints.Global.NodeSep != expectedNodeSep {
			t.Errorf("Expected NodeSep %v, got %v", expectedNodeSep, constraints.Global.NodeSep)
		}
	})

	t.Run("Dense Graph Scaling", func(t *testing.T) {
		// Create enough nodes to trigger dynamic scaling
		elements := make([]*Element, 10)
		for i := 0; i < 10; i++ {
			elements[i] = &Element{ID: "Node", Kind: "component"}
		}

		constraints := BuildConstraints(elements, nil, 3, config) // L3 view (no L1 boost)

		// Expected scaling: 1.0 + 0.25 * (10/8) = 1.3125
		baseSep := pxToInchFloat(float64(config.NodeSep))
		expectedScale := DynamicScalingBase + DynamicScalingFactor*float64(10)/DynamicScalingDivisor
		expectedNodeSep := baseSep * expectedScale

		if constraints.Global.NodeSep <= baseSep {
			t.Error("Expected increased NodeSep for dense graph")
		}
		if constraints.Global.Splines != "polyline" {
			t.Errorf("Expected polyline splines for dense graph, got %s", constraints.Global.Splines)
		}
		// Allow some float precision difference
		if constraints.Global.NodeSep < expectedNodeSep-0.001 || constraints.Global.NodeSep > expectedNodeSep+0.001 {
			t.Errorf("Expected NodeSep approx %v, got %v", expectedNodeSep, constraints.Global.NodeSep)
		}
	})

	t.Run("Hub Detection", func(t *testing.T) {
		elements := []*Element{
			{ID: "Hub", Kind: "system", Width: 200, Height: 100},
			{ID: "Other", Kind: "system", Width: 200, Height: 100},
		}

		// Create many connections to Hub
		relations := make([]*Relation, HubDegreeThreshold+1)
		for i := 0; i < HubDegreeThreshold+1; i++ {
			relations[i] = &Relation{From: "Hub", To: "Other", Label: "rel"}
		}

		constraints := BuildConstraints(elements, relations, 2, config)

		var hubSize SizeConstraint
		found := false
		for _, s := range constraints.Sizes {
			if s.NodeID == "Hub" {
				hubSize = s
				found = true
				break
			}
		}

		if !found {
			t.Fatal("Hub node size constraint not found")
		}

		// Should be scaled up
		expectedMinWidth := MinWidthSystem * HubScaleWidth
		if hubSize.MinWidth < expectedMinWidth {
			t.Errorf("Expected Hub MinWidth >= %v, got %v", expectedMinWidth, hubSize.MinWidth)
		}
	})
}

package dot

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/export/views"
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
		elements := []*views.Element{
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
		elements := make([]*views.Element, 10)
		for i := 0; i < 10; i++ {
			elements[i] = &views.Element{ID: "Node", Kind: "component"}
		}

		constraints := BuildConstraints(elements, nil, 3, config) // L3 view

		// Expected scaling: 1.0 + 0.25 * (10/8) = 1.3125
		baseSep := pxToInchFloat(float64(config.NodeSep))
		expectedScale := DynamicScalingBase + DynamicScalingFactor*float64(10)/DynamicScalingDivisor
		expectedNodeSep := baseSep * expectedScale

		if constraints.Global.NodeSep <= baseSep {
			t.Error("Expected increased NodeSep for dense graph")
		}
		// Round 3: L3 uses spline for better crossing reduction (not polyline)
		if constraints.Global.Splines != "spline" {
			t.Errorf("Expected spline for L3 dense graph (Round 3 strategy), got %s", constraints.Global.Splines)
		}
		// Apply L3 Round 1 spacing: multiply by 1.15 (L3NodeSepScale) then 1.10 for >=15 (but only 10 nodes here so no extra boost)
		expectedNodeSep *= L3NodeSepScale
		// Note: ComplexGraphThreshold is 20 now, so 10 nodes doesn't trigger extra boost.
		// Allow some float precision difference
		if constraints.Global.NodeSep < expectedNodeSep-0.001 || constraints.Global.NodeSep > expectedNodeSep+0.001 {
			t.Errorf("Expected NodeSep approx %v, got %v", expectedNodeSep, constraints.Global.NodeSep)
		}
	})

	t.Run("Hub Detection", func(t *testing.T) {
		elements := []*views.Element{
			{ID: "Hub", Kind: "system", Width: 200, Height: 100},
			{ID: "Other", Kind: "system", Width: 200, Height: 100},
		}

		// Create many connections to Hub
		relations := make([]*views.Relation, HubDegreeThreshold+1)
		for i := 0; i < HubDegreeThreshold+1; i++ {
			relations[i] = &views.Relation{From: "Hub", To: "Other", Label: "rel"}
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

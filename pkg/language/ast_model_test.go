package language

import (
	"testing"
)

func TestElementDef_Helpers(t *testing.T) {
	// Test Assignment branch
	title := "My Title"
	assign := &ElementAssignment{
		Name:    "A",
		Kind:    "system",
		Title:   &title,
		TagRefs: []string{"#tag1"},
	}
	def := &ElementDef{Assignment: assign}

	if def.GetID() != "A" {
		t.Errorf("expected A, got %s", def.GetID())
	}
	if def.GetKind() != "system" {
		t.Errorf("expected system, got %s", def.GetKind())
	}
	if def.GetTitle() == nil || *def.GetTitle() != "My Title" {
		t.Error("expected My Title")
	}
	if len(def.GetTagRefs()) != 1 {
		t.Error("expected 1 tagref")
	}
	_ = def.Location()

	// Empty case
	def = &ElementDef{}
	if def.GetID() != "" {
		t.Error("expected empty ID")
	}
	if def.GetKind() != "" {
		t.Error("expected empty Kind")
	}
	if def.GetTitle() != nil {
		t.Error("expected nil Title")
	}
	if def.GetBody() != nil {
		t.Error("expected nil Body")
	}
	if def.GetTagRefs() != nil {
		t.Error("expected nil TagRefs")
	}
}

func TestViewExpr_String(t *testing.T) {
	selector := "sys"
	suffix := "sub"
	tests := []struct {
		expr ViewExpr
		want string
	}{
		{ViewExpr{Wildcard: true}, "*"},
		{ViewExpr{Recursive: true}, "**"},
		{ViewExpr{Selector: &selector}, "sys"},
		{ViewExpr{Selector: &selector, Sub: &ViewExprSuffix{Recursive: true}}, "sys.**"},
		{ViewExpr{Selector: &selector, Sub: &ViewExprSuffix{Wildcard: true}}, "sys.*"},
		{ViewExpr{Selector: &selector, Sub: &ViewExprSuffix{Ident: &suffix}}, "sys.sub"},
	}

	for _, tt := range tests {
		if got := tt.expr.String(); got != tt.want {
			t.Errorf("%v.String() = %q, want %q", tt.expr, got, tt.want)
		}
	}
}

func TestModelPostProcess(_ *testing.T) {
	// Test Views PostProcess
	vBlock := &Views{
		Items: []*ViewsItem{
			{View: &ViewDef{}},
		},
	}
	vBlock.PostProcess()

	// Test ElementDef PostProcess
	item := &BodyItem{
		Element: &ElementDef{
			Assignment: &ElementAssignment{
				Body: &ElementDefBody{
					Items: []*BodyItem{
						{SLO: &SLOBlock{}},
						{Scale: &ScaleBlock{}},
						{Relation: &Relation{}},
					},
				},
			},
		},
	}
	item.PostProcess()
}

func TestRequirementADR_Location(_ *testing.T) {
	r := &Requirement{}
	_ = r.Location()
	a := &ADR{}
	_ = a.Location()
}

func TestBlocks_PostProcess(_ *testing.T) {
	// DeploymentNode
	d := &DeploymentNode{
		Items: []DeploymentNodeItem{
			{Node: &DeploymentNode{}},
			{ContainerInstance: &ContainerInstance{}},
			{Infrastructure: &InfrastructureNode{}},
		},
	}
	d.PostProcess()

	// Scenario
	s := &Scenario{
		Items: []*ScenarioItem{
			{Step: &ScenarioStep{FromParts: []string{"A"}, ToParts: []string{"B"}}},
		},
	}
	s.PostProcess()

	// Policy
	p := &Policy{
		Body: &PolicyBody{
			Properties: []PolicyProperty{
				{Category: mkStrJSON("C")},
				{Enforcement: mkStrJSON("E")},
				{Description: mkStrJSON("D")},
				{Tags: []string{"T"}},
				{Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "K", Value: mkStrJSON("V")}}}},
			},
		},
	}
	p.PostProcess()

	// Flow
	f := &Flow{
		Items: []*ScenarioItem{
			{Step: &ScenarioStep{FromParts: []string{"A"}, ToParts: []string{"B"}}},
		},
	}
	f.PostProcess()

	// Requirement
	req := &Requirement{
		Body: &RequirementBody{
			Properties: []RequirementProperty{
				{Type: mkStrJSON("T")},
				{Description: mkStrJSON("D")},
				{Tags: []string{"T"}},
				{Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "K", Value: mkStrJSON("V")}}}},
			},
		},
	}
	req.PostProcess()

	// ADR
	adr := &ADR{
		Body: &ADRBody{
			Properties: []ADRProperty{
				{Status: mkStrJSON("S")},
				{Context: mkStrJSON("C")},
				{Decision: mkStrJSON("D")},
				{Consequences: mkStrJSON("C")},
				{Tags: []string{"T"}},
			},
		},
	}
	adr.PostProcess()

	// ScaleBlock
	scale := &ScaleBlock{
		Items: []*ScaleItem{
			{Min: &ScaleMin{Val: 1}},
			{Max: &ScaleMax{Val: 10}},
			{Metric: &ScaleMetric{Val: "m"}},
		},
	}
	scale.PostProcess()

	// SLOBlock
	slo := &SLOBlock{
		Items: []*SLOItem{
			{Availability: &SLOAvailability{}},
			{Latency: &SLOLatency{}},
			{ErrorRate: &SLOErrorRate{}},
			{Throughput: &SLOThroughput{}},
		},
	}
	slo.PostProcess()
}

func TestElements_PostProcess(_ *testing.T) {
	// System
	sys := &System{
		Items: []SystemItem{
			{Description: mkStrJSON("D")},
			{Container: &Container{}},
			{DataStore: &DataStore{}},
			{Queue: &Queue{}},
			{Person: &Person{}},
			{Relation: &Relation{}},
			{Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "K", Value: mkStrJSON("V")}}}},
			{ConstraintsBlock: &ConstraintsBlock{Entries: []*ConstraintEntry{{Key: "K"}}}},
			{ConventionsBlock: &ConventionsBlock{Entries: []*ConventionEntry{{Key: "K"}}}},

			{Style: &StyleDecl{Body: &StyleBlock{Entries: []*StyleEntry{{Key: "SK", Value: mkStrJSON("SV")}}}}},
			{SLO: &SLOBlock{}},
		},
	}
	sys.PostProcess()

	// Container
	cont := &Container{
		Items: []ContainerItem{
			{Description: mkStrJSON("D")},
			{Component: &Component{}},
			{DataStore: &DataStore{}},
			{Queue: &Queue{}},
			{Relation: &Relation{}},
			{Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "K", Value: mkStrJSON("V")}}}},
			{ConstraintsBlock: &ConstraintsBlock{Entries: []*ConstraintEntry{{Key: "K"}}}},
			{ConventionsBlock: &ConventionsBlock{Entries: []*ConventionEntry{{Key: "K"}}}},

			{Style: &StyleDecl{Body: &StyleBlock{Entries: []*StyleEntry{{Key: "SK", Value: mkStrJSON("SV")}}}}},
			{Scale: &ScaleBlock{}},
			{Version: mkStrJSON("1.0")},
			{SLO: &SLOBlock{}},
		},
	}
	cont.PostProcess()

	// Component
	comp := &Component{
		Items: []ComponentItem{
			{Technology: mkStrJSON("T")},
			{Description: mkStrJSON("D")},
			{Relation: &Relation{}},
			{Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "K", Value: mkStrJSON("V")}}}},

			{Style: &StyleDecl{Body: &StyleBlock{Entries: []*StyleEntry{{Key: "SK", Value: mkStrJSON("SV")}}}}},
			{Scale: &ScaleBlock{}},
		},
	}
	comp.PostProcess()

	// DataStore
	ds := &DataStore{
		Items: []DataStoreItem{
			{Technology: mkStrJSON("T")},
			{Description: mkStrJSON("D")},
			{Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "K", Value: mkStrJSON("V")}}}},

			{Style: &StyleDecl{Body: &StyleBlock{Entries: []*StyleEntry{{Key: "SK", Value: mkStrJSON("SV")}}}}},
		},
	}
	ds.PostProcess()

	// Queue
	q := &Queue{
		Items: []QueueItem{
			{Technology: mkStrJSON("T")},
			{Description: mkStrJSON("D")},
			{Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "K", Value: mkStrJSON("V")}}}},

			{Style: &StyleDecl{Body: &StyleBlock{Entries: []*StyleEntry{{Key: "SK", Value: mkStrJSON("SV")}}}}},
		},
	}
	q.PostProcess()

	// Person
	p := &Person{
		Items: []PersonItem{
			{Description: mkStrJSON("D")},
			{Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "K", Value: mkStrJSON("V")}}}},

			{Style: &StyleDecl{Body: &StyleBlock{Entries: []*StyleEntry{{Key: "SK", Value: mkStrJSON("SV")}}}}},
		},
	}
	p.PostProcess()
}

func mkStrJSON(s string) *string {
	return &s
}

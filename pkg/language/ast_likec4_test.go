package language

import (
	"testing"
)

func TestLikeC4ElementDef_Helpers(t *testing.T) {
	// Test Assignment branch
	title := "My Title"
	assign := &LikeC4Assignment{
		Name:    "A",
		Kind:    "system",
		Title:   &title,
		TagRefs: []string{"#tag1"},
	}
	def := &LikeC4ElementDef{Assignment: assign}

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

	// Test Definition branch
	definition := &LikeC4Definition{
		Kind: "person",
		Name: &title, // uses the same variable for name
	}
	def = &LikeC4ElementDef{Definition: definition}
	if def.GetID() != "My Title" {
		t.Errorf("expected My Title, got %s", def.GetID())
	}
	if def.GetKind() != "person" {
		t.Errorf("expected person, got %s", def.GetKind())
	}

	// Empty case
	def = &LikeC4ElementDef{}
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

func TestLikeC4ViewsBlock_Location(_ *testing.T) {
	v := &LikeC4ViewsBlock{}
	_ = v.Location()
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

func TestLikeC4PostProcess(_ *testing.T) {
	// Test LikeC4ViewsBlock PostProcess
	vBlock := &LikeC4ViewsBlock{
		Items: []*LikeC4ViewsItem{
			{View: &LikeC4ViewDef{}},
		},
	}
	vBlock.PostProcess()

	// Test LikeC4ElementDef PostProcess
	item := &LikeC4BodyItem{
		Element: &LikeC4ElementDef{
			Assignment: &LikeC4Assignment{
				Body: &LikeC4ElementDefBody{
					Items: []*LikeC4BodyItem{
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
			{ContractsBlock: &ContractsBlock{Contracts: []*Contract{{ID: "C"}}}},
			{ConstraintsBlock: &ConstraintsBlock{Entries: []*ConstraintEntry{{Key: "K"}}}},
			{ConventionsBlock: &ConventionsBlock{Entries: []*ConventionEntry{{Key: "K"}}}},
			{Properties: &PropertiesBlock{Entries: []*PropertyEntry{{Key: "PK", Value: "PV"}}}},
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
			{ContractsBlock: &ContractsBlock{Contracts: []*Contract{{ID: "C"}}}},
			{ConstraintsBlock: &ConstraintsBlock{Entries: []*ConstraintEntry{{Key: "K"}}}},
			{ConventionsBlock: &ConventionsBlock{Entries: []*ConventionEntry{{Key: "K"}}}},
			{Properties: &PropertiesBlock{Entries: []*PropertyEntry{{Key: "PK", Value: "PV"}}}},
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
			{Properties: &PropertiesBlock{Entries: []*PropertyEntry{{Key: "PK", Value: "PV"}}}},
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
			{Properties: &PropertiesBlock{Entries: []*PropertyEntry{{Key: "PK", Value: "PV"}}}},
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
			{Properties: &PropertiesBlock{Entries: []*PropertyEntry{{Key: "PK", Value: "PV"}}}},
			{Style: &StyleDecl{Body: &StyleBlock{Entries: []*StyleEntry{{Key: "SK", Value: mkStrJSON("SV")}}}}},
		},
	}
	q.PostProcess()

	// Person
	p := &Person{
		Items: []PersonItem{
			{Description: mkStrJSON("D")},
			{Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "K", Value: mkStrJSON("V")}}}},
			{Properties: &PropertiesBlock{Entries: []*PropertyEntry{{Key: "PK", Value: "PV"}}}},
			{Style: &StyleDecl{Body: &StyleBlock{Entries: []*StyleEntry{{Key: "SK", Value: mkStrJSON("SV")}}}}},
		},
	}
	p.PostProcess()
}

func mkStrJSON(s string) *string {
	return &s
}

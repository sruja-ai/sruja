// pkg/export/json/converters_coverage_test.go
// Additional tests to improve coverage for converter functions
package json

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestConvertComponent_AllFields(t *testing.T) {
	verb := "calls"
	rel := &language.Relation{
		From:  language.QualifiedIdent{Parts: []string{"A"}},
		To:    language.QualifiedIdent{Parts: []string{"B"}},
		Verb:  &verb,
		Label: mkStrCoverage("label"),
		Tags:  []string{"tag1", "tag2"},
	}

	tech := "Go"
	scale := &language.ScaleBlock{
		Min:    mkInt(1),
		Max:    mkInt(10),
		Metric: mkStrCoverage("instances"),
	}

	comp := &language.Component{
		ID:          "comp1",
		Label:       "Component",
		Description: mkStrCoverage("Description"),
		Technology:  &tech,
		Relations:   []*language.Relation{rel},
		Metadata: []*language.MetaEntry{
			{Key: "key1", Value: mkStrCoverage("value1")},
			{Key: "key2", Array: []string{"a", "b"}},
		},

		Style: map[string]string{"color": "#fff"},
		Scale: scale,
	}

	result := convertComponent(comp)

	if result.ID != "comp1" {
		t.Errorf("expected ID comp1, got %s", result.ID)
	}
	if result.Label != "Component" {
		t.Errorf("expected Label Component, got %s", result.Label)
	}
	if result.Description == nil || *result.Description != "Description" {
		t.Errorf("expected Description, got %v", result.Description)
	}
	if result.Technology == nil || *result.Technology != "Go" {
		t.Errorf("expected Technology Go, got %v", result.Technology)
	}
	if len(result.Relations) != 1 {
		t.Errorf("expected 1 relation, got %d", len(result.Relations))
	}
	if len(result.Metadata) != 2 {
		t.Errorf("expected 2 metadata entries, got %d", len(result.Metadata))
	}

	if len(result.Style) != 1 || result.Style["color"] != "#fff" {
		t.Errorf("expected style, got %v", result.Style)
	}
	if result.Scale == nil || result.Scale.Min == nil || *result.Scale.Min != 1 {
		t.Errorf("expected scale, got %v", result.Scale)
	}
}

func TestConvertDataStore_AllFields(t *testing.T) {
	tech := "PostgreSQL"
	ds := &language.DataStore{
		ID:          "ds1",
		Label:       "Database",
		Description: mkStrCoverage("Main database"),
		Technology:  &tech,
		Metadata: []*language.MetaEntry{
			{Key: "region", Value: mkStrCoverage("us-east")},
		},

		Style: map[string]string{"shape": "cylinder"},
	}

	result := convertDataStore(ds)

	if result.ID != "ds1" {
		t.Errorf("expected ID ds1, got %s", result.ID)
	}
	if result.Label != "Database" {
		t.Errorf("expected Label Database, got %s", result.Label)
	}
	if result.Description == nil || *result.Description != "Main database" {
		t.Errorf("expected Description, got %v", result.Description)
	}
	if result.Technology == nil || *result.Technology != "PostgreSQL" {
		t.Errorf("expected Technology PostgreSQL, got %v", result.Technology)
	}
	if len(result.Metadata) != 1 {
		t.Errorf("expected 1 metadata entry, got %d", len(result.Metadata))
	}

	if len(result.Style) != 1 || result.Style["shape"] != "cylinder" {
		t.Errorf("expected style, got %v", result.Style)
	}
}

func TestConvertQueue_AllFields(t *testing.T) {
	tech := "RabbitMQ"
	q := &language.Queue{
		ID:          "q1",
		Label:       "Message Queue",
		Description: mkStrCoverage("Event queue"),
		Technology:  &tech,
		Metadata: []*language.MetaEntry{
			{Key: "durable", Value: mkStrCoverage("true")},
		},

		Style: map[string]string{"color": "#00f"},
	}

	result := convertQueue(q)

	if result.ID != "q1" {
		t.Errorf("expected ID q1, got %s", result.ID)
	}
	if result.Label != "Message Queue" {
		t.Errorf("expected Label Message Queue, got %s", result.Label)
	}
	if result.Description == nil || *result.Description != "Event queue" {
		t.Errorf("expected Description, got %v", result.Description)
	}
	if result.Technology == nil || *result.Technology != "RabbitMQ" {
		t.Errorf("expected Technology RabbitMQ, got %v", result.Technology)
	}
	if len(result.Metadata) != 1 {
		t.Errorf("expected 1 metadata entry, got %d", len(result.Metadata))
	}

	if len(result.Style) != 1 || result.Style["color"] != "#00f" {
		t.Errorf("expected style, got %v", result.Style)
	}
}

func TestConvertSLO_AllFields(t *testing.T) {
	slo := &language.SLOBlock{
		Availability: &language.SLOAvailability{
			Target:  mkStrCoverage("99.9%"),
			Window:  mkStrCoverage("30d"),
			Current: mkStrCoverage("99.95"),
		},
		Latency: &language.SLOLatency{
			P95:    mkStrCoverage("100ms"),
			P99:    mkStrCoverage("200ms"),
			Window: mkStrCoverage("7d"),
			Current: &language.SLOCurrent{
				P95: mkStrCoverage("95ms"),
				P99: mkStrCoverage("190ms"),
			},
		},
		ErrorRate: &language.SLOErrorRate{
			Target:  mkStrCoverage("0.1%"),
			Window:  mkStrCoverage("30d"),
			Current: mkStrCoverage("0.05"),
		},
		Throughput: &language.SLOThroughput{
			Target:  mkStrCoverage("1000 req/s"),
			Window:  mkStrCoverage("1h"),
			Current: mkStrCoverage("1200.0"),
		},
	}

	result := convertSLO(slo)

	if result == nil {
		t.Fatal("expected non-nil SLO result")
	}
	if result.Availability == nil {
		t.Fatal("expected Availability")
	}
	if result.Availability.Target != "99.9%" {
		t.Errorf("expected Target 99.9%%, got %s", result.Availability.Target)
	}
	if result.Availability.Window != "30d" {
		t.Errorf("expected Window 30d, got %s", result.Availability.Window)
	}
	if result.Availability.Current == nil || *result.Availability.Current != "99.95" {
		t.Errorf("expected Current 99.95, got %v", result.Availability.Current)
	}

	if result.Latency == nil {
		t.Fatal("expected Latency")
	}
	if result.Latency.P95 != "100ms" {
		t.Errorf("expected P95 100ms, got %s", result.Latency.P95)
	}
	if result.Latency.Current == nil {
		t.Fatal("expected Latency.Current")
	}
	if result.Latency.Current.P95 != "95ms" {
		t.Errorf("expected Current.P95 95ms, got %s", result.Latency.Current.P95)
	}

	if result.ErrorRate == nil {
		t.Fatal("expected ErrorRate")
	}
	if result.ErrorRate.Target != "0.1%" {
		t.Errorf("expected ErrorRate.Target 0.1%%, got %s", result.ErrorRate.Target)
	}

	if result.Throughput == nil {
		t.Fatal("expected Throughput")
	}
	if result.Throughput.Target != "1000 req/s" {
		t.Errorf("expected Throughput.Target 1000 req/s, got %s", result.Throughput.Target)
	}
}

func TestConvertSLO_Nil(t *testing.T) {
	result := convertSLO(nil)
	if result != nil {
		t.Errorf("expected nil for nil SLO, got %v", result)
	}
}

func TestConvertSLO_PartialFields(t *testing.T) {
	slo := &language.SLOBlock{
		Availability: &language.SLOAvailability{
			Target:  mkStrCoverage("99.9%"),
			Window:  mkStrCoverage("30d"),
			Current: mkStrCoverage("99.9"),
		},
	}

	result := convertSLO(slo)

	if result == nil {
		t.Fatal("expected non-nil SLO result")
	}
	if result.Availability == nil {
		t.Fatal("expected Availability")
	}
	if result.Latency != nil {
		t.Error("expected nil Latency")
	}
	if result.ErrorRate != nil {
		t.Error("expected nil ErrorRate")
	}
	if result.Throughput != nil {
		t.Error("expected nil Throughput")
	}
}

func TestConvertScale_AllFields(t *testing.T) {
	scale := &language.ScaleBlock{
		Min:    mkInt(1),
		Max:    mkInt(100),
		Metric: mkStrCoverage("instances"),
	}

	result := convertScale(scale)

	if result == nil {
		t.Fatal("expected non-nil Scale result")
	}
	if result.Min == nil || *result.Min != 1 {
		t.Errorf("expected Min 1, got %v", result.Min)
	}
	if result.Max == nil || *result.Max != 100 {
		t.Errorf("expected Max 100, got %v", result.Max)
	}
	if result.Metric == nil || *result.Metric != "instances" {
		t.Errorf("expected Metric instances, got %v", result.Metric)
	}
}

func TestConvertScale_Nil(t *testing.T) {
	result := convertScale(nil)
	if result != nil {
		t.Errorf("expected nil for nil Scale, got %v", result)
	}
}

func TestConvertSystem_AllFields(t *testing.T) {
	sys := &language.System{
		ID:          "sys1",
		Label:       "System",
		Description: mkStrCoverage("Description"),
		Metadata:    []*language.MetaEntry{{Key: "k", Value: mkStrCoverage("v")}},
		Containers: []*language.Container{
			{ID: "c1", Label: "C1"},
		},
	}
	result := convertSystem(sys)
	if result.ID != "sys1" {
		t.Errorf("expected ID sys1, got %s", result.ID)
	}
	if len(result.Containers) != 1 {
		t.Errorf("expected 1 container, got %d", len(result.Containers))
	}
}

func TestConvertContainer_AllFields(t *testing.T) {
	cont := &language.Container{
		ID:    "cont1",
		Label: "Container",
		Components: []*language.Component{
			{ID: "comp1", Label: "Comp1"},
		},
		DataStores: []*language.DataStore{
			{ID: "ds1", Label: "DS1"},
		},
		Queues: []*language.Queue{
			{ID: "q1", Label: "Q1"},
		},
	}
	result := convertContainer(cont)
	if result.ID != "cont1" {
		t.Errorf("expected ID cont1, got %s", result.ID)
	}
	if len(result.Components) != 1 {
		t.Errorf("expected 1 component, got %d", len(result.Components))
	}
}

func TestConvertPerson_AllFields(t *testing.T) {
	p := &language.Person{
		ID:    "p1",
		Label: "Person",
	}
	result := convertPerson(p)
	if result.ID != "p1" {
		t.Errorf("expected ID p1, got %s", result.ID)
	}
}

func TestConvertPolicy_AllFields(t *testing.T) {
	p := &language.Policy{
		ID:          "pol1",
		Description: "Desc",
	}
	result := convertPolicy(p)
	if result.ID != "pol1" {
		t.Errorf("expected ID pol1, got %s", result.ID)
	}
}

func TestConvertConstraints(t *testing.T) {
	constraints := []*language.ConstraintEntry{
		{Key: "k1", Value: "v1"},
	}
	result := convertConstraints(constraints)
	if len(result) != 1 || result[0].Key != "k1" {
		t.Errorf("expected k1, got %v", result)
	}
}

func TestConvertConventions(t *testing.T) {
	conventions := []*language.ConventionEntry{
		{Key: "k1", Value: "v1"},
	}
	result := convertConventions(conventions)
	if len(result) != 1 || result[0].Key != "k1" {
		t.Errorf("expected k1, got %v", result)
	}
}

func TestStringPtrToIntPtr_NonNil(t *testing.T) {
	s := "123"
	result := stringPtrToIntPtr(&s)
	if result == nil || *result != 123 {
		t.Errorf("expected 123, got %v", result)
	}
}

// TODO: SchemaBlock, SchemaEntry, TypeSpec, and convertSchemaBlock are not yet implemented
// Uncomment this test when schema conversion is implemented
/*
func TestConvertSchemaBlock(t *testing.T) {
	sb := &language.SchemaBlock{
		Entries: []*language.SchemaEntry{
			{Key: "key1", Type: &language.TypeSpec{Name: "String"}},
		},
	}
	result := convertSchemaBlock(sb)
	if result == nil || len(result.Entries) != 1 || result.Entries[0].Key != "key1" {
		t.Errorf("expected key1, got %v", result)
	}
}
*/

func TestConvertPolicies(t *testing.T) {
	policies := []*language.Policy{
		{ID: "p1", Description: "d1"},
	}
	result := convertPolicies(policies)
	if len(result) != 1 || result[0].ID != "p1" {
		t.Errorf("expected 1 policy, got %v", result)
	}
}

func TestStrVal_Nil_Coverage(t *testing.T) {
	result := strVal(nil)
	if result != "" {
		t.Errorf("expected empty string for nil, got %s", result)
	}
}

func mkStrCoverage(s string) *string {
	return &s
}

func mkInt(i int) *int {
	return &i
}

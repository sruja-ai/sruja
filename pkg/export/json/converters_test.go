// pkg/export/json/converters_test.go
package json

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func strPtr(s string) *string { return &s }

func TestStringPtrToIntPtr(t *testing.T) {
	if got := stringPtrToIntPtr(nil); got != nil {
		t.Fatalf("expected nil, got %v", got)
	}
	s := "42"
	got := stringPtrToIntPtr(&s)
	if got == nil || *got != 42 {
		t.Fatalf("expected 42, got %v", got)
	}
	bad := "notint"
	if got := stringPtrToIntPtr(&bad); got != nil {
		t.Fatalf("expected nil for invalid int, got %v", got)
	}
}

func TestConvertSystemMinimal(t *testing.T) {
	sys := &language.System{ID: "sys1", Label: "System 1", Description: nil}
	out := convertSystem(sys)
	if out.ID != "sys1" || out.Label != "System 1" || out.Description != nil {
		t.Fatalf("system fields not copied correctly: %+v", out)
	}
	// all slice fields should be nil when empty
	if out.Containers != nil || out.Components != nil || out.DataStores != nil || out.Queues != nil || out.Relations != nil {
		t.Fatalf("expected nil slices for empty system, got %+v", out)
	}
}

func TestConvertContainerAndComponent(t *testing.T) {
	// component
	comp := &language.Component{ID: "c1", Label: "Comp", Description: nil}
	// relation between components (using QualifiedIdent)
	rel := &language.Relation{From: language.QualifiedIdent{Parts: []string{"c1"}}, To: language.QualifiedIdent{Parts: []string{"c2"}}}
	// container with component and relation
	cont := &language.Container{ID: "cont1", Label: "Cont", Description: nil, Components: []*language.Component{comp}, Relations: []*language.Relation{rel}}
	out := convertContainer(cont)
	if out.ID != "cont1" || out.Label != "Cont" {
		t.Fatalf("container fields mismatch: %+v", out)
	}
	if len(out.Components) != 1 || out.Components[0].ID != "c1" {
		t.Fatalf("component conversion failed: %+v", out.Components)
	}
	if len(out.Relations) != 1 || out.Relations[0].From != "c1" || out.Relations[0].To != "c2" {
		t.Fatalf("relation conversion failed: %+v", out.Relations)
	}
}

func TestUtilityFunctions(t *testing.T) {
	if sanitize("  abc ") != "abc" {
		t.Fatalf("sanitize failed")
	}
	if idOrLabel("", "lbl") != "lbl" {
		t.Fatalf("idOrLabel fallback failed")
	}
	if idOrLabel("id", "lbl") != "id" {
		t.Fatalf("idOrLabel primary failed")
	}
}

func TestConvertSLO(t *testing.T) {
	slo := &language.SLOBlock{Availability: &language.SLOAvailability{Target: strPtr("99.9%"), Window: strPtr("1y"), Current: strPtr("99.5")}}
	out := convertSLO(slo)
	if out == nil || out.Availability == nil {
		t.Fatalf("convertSLO missing fields")
	}
	if out.Availability.Target != "99.9%" || out.Availability.Window != "1y" {
		t.Fatalf("convertSLO values incorrect: %+v", out.Availability)
	}
	if out.Availability.Current == nil || *out.Availability.Current != "99.5" {
		t.Fatalf("convertSLO Current incorrect: %+v", out.Availability.Current)
	}
}

func TestConvertSLO_Latency(t *testing.T) {
	slo := &language.SLOBlock{
		Latency: &language.SLOLatency{
			P95:    strPtr("200ms"),
			P99:    strPtr("500ms"),
			Window: strPtr("7d"),
			Current: &language.SLOCurrent{
				P95: strPtr("180ms"),
				P99: strPtr("420ms"),
			},
		},
	}
	out := convertSLO(slo)
	if out == nil || out.Latency == nil {
		t.Fatalf("convertSLO missing latency")
	}
	if out.Latency.P95 != "200ms" || out.Latency.P99 != "500ms" {
		t.Fatalf("convertSLO latency values incorrect")
	}
	if out.Latency.Current == nil || out.Latency.Current.P95 != "180ms" {
		t.Fatalf("convertSLO latency current incorrect")
	}
}

func TestConvertSLO_ErrorRate(t *testing.T) {
	slo := &language.SLOBlock{
		ErrorRate: &language.SLOErrorRate{
			Target:  strPtr("0.1%"),
			Window:  strPtr("30d"),
			Current: strPtr("0.08%"),
		},
	}
	out := convertSLO(slo)
	if out == nil || out.ErrorRate == nil {
		t.Fatalf("convertSLO missing errorRate")
	}
	if out.ErrorRate.Target != "0.1%" {
		t.Fatalf("convertSLO errorRate target incorrect")
	}
}

func TestConvertSLO_Throughput(t *testing.T) {
	slo := &language.SLOBlock{
		Throughput: &language.SLOThroughput{
			Target:  strPtr("1000 req/s"),
			Window:  strPtr("1h"),
			Current: strPtr("950 req/s"),
		},
	}
	out := convertSLO(slo)
	if out == nil || out.Throughput == nil {
		t.Fatalf("convertSLO missing throughput")
	}
	if out.Throughput.Target != "1000 req/s" {
		t.Fatalf("convertSLO throughput target incorrect")
	}
}

func TestConvertScale(t *testing.T) {
	scale := &language.ScaleBlock{
		Min:    intPtr(1),
		Max:    intPtr(10),
		Metric: strPtr("instances"),
	}
	out := convertScale(scale)
	if out == nil {
		t.Fatalf("convertScale returned nil")
	}
	if out.Min == nil || *out.Min != 1 {
		t.Fatalf("convertScale Min incorrect")
	}
	if out.Max == nil || *out.Max != 10 {
		t.Fatalf("convertScale Max incorrect")
	}
	if out.Metric == nil || *out.Metric != "instances" {
		t.Fatalf("convertScale Metric incorrect")
	}
}

func TestConvertPerson(t *testing.T) {
	person := &language.Person{
		ID:          "p1",
		Label:       "Person 1",
		Description: strPtr("A person"),
		Metadata: []*language.MetaEntry{
			{Key: "role", Value: strPtr("admin")},
		},
	}
	out := convertPerson(person)
	if out.ID != "p1" || out.Label != "Person 1" {
		t.Fatalf("convertPerson fields incorrect")
	}
	if out.Description == nil || *out.Description != "A person" {
		t.Fatalf("convertPerson description incorrect")
	}
}

func TestConvertDataStore(t *testing.T) {
	ds := &language.DataStore{
		ID:          "ds1",
		Label:       "Database",
		Description: strPtr("A database"),
		Technology:  strPtr("PostgreSQL"),
	}
	out := convertDataStore(ds)
	if out.ID != "ds1" || out.Label != "Database" {
		t.Fatalf("convertDataStore fields incorrect")
	}
	if out.Technology == nil || *out.Technology != "PostgreSQL" {
		t.Fatalf("convertDataStore technology incorrect")
	}
}

func TestConvertQueue(t *testing.T) {
	q := &language.Queue{
		ID:          "q1",
		Label:       "Queue",
		Description: strPtr("A queue"),
		Technology:  strPtr("RabbitMQ"),
	}
	out := convertQueue(q)
	if out.ID != "q1" || out.Label != "Queue" {
		t.Fatalf("convertQueue fields incorrect")
	}
	if out.Technology == nil || *out.Technology != "RabbitMQ" {
		t.Fatalf("convertQueue technology incorrect")
	}
}

func TestStrVal(t *testing.T) {
	if strVal(nil) != "" {
		t.Fatalf("strVal(nil) should return empty string")
	}
	s := "test"
	if strVal(&s) != "test" {
		t.Fatalf("strVal should return string value")
	}
}

func TestIdOrLabel_WithID(t *testing.T) {
	if idOrLabel("id", "label") != "id" {
		t.Fatalf("idOrLabel should prefer id")
	}
}

func TestIdOrLabel_EmptyID(t *testing.T) {
	if idOrLabel("", "label") != "label" {
		t.Fatalf("idOrLabel should use label when id is empty")
	}
	if idOrLabel("   ", "label") != "label" {
		t.Fatalf("idOrLabel should use label when id is whitespace")
	}
}

func intPtr(i int) *int { return &i }

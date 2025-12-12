package json

import (
    "testing"
    "github.com/sruja-ai/sruja/pkg/language"
)

func TestBuildNavigation_Scenarios(t *testing.T) {
    sc := &language.Scenario{ID: "Sc1", Title: "Checkout"}
    arch := &language.Architecture{Scenarios: []*language.Scenario{sc}}
    nav := buildNavigation(arch)
    if len(nav.Levels) == 0 { t.Fatalf("expected levels set") }
    if len(nav.Scenarios) != 1 || nav.Scenarios[0].Label != "Checkout" { t.Fatalf("scenarios nav not built correctly: %+v", nav.Scenarios) }
}


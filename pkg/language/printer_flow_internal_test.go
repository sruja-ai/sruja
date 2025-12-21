package language

import (
	"strings"
	"testing"
)

func TestPrinter_printFlow_PrintsDFDStyleSteps(t *testing.T) {
	desc := "Order details"
	order := "1"
	flow := &Flow{
		ID:    "F1",
		Title: stringPtr("Checkout"),
		Items: []*ScenarioItem{
			{Step: &ScenarioStep{From: QualifiedIdent{Parts: []string{"Customer"}}, To: QualifiedIdent{Parts: []string{"Shop"}}, Description: &desc, Tags: []string{"critical", "edge"}, Order: &order}},
			{Step: &ScenarioStep{From: QualifiedIdent{Parts: []string{"Shop", "WebApp"}}, To: QualifiedIdent{Parts: []string{"Shop", "DB"}}}},
		},
	}

	var sb strings.Builder
	p := NewPrinter()
	p.printFlow(&sb, flow)
	out := sb.String()

	if !strings.Contains(out, "flow F1 \"Checkout\" {") {
		t.Fatalf("missing flow header in output:\n%s", out)
	}
	if !strings.Contains(out, "Customer -> Shop \"Order details\" [critical, edge] order \"1\"") {
		t.Fatalf("missing first step details in output:\n%s", out)
	}
	if !strings.Contains(out, "Shop.WebApp -> Shop.DB") {
		t.Fatalf("missing qualified step in output:\n%s", out)
	}
	if !strings.HasSuffix(strings.TrimSpace(out), "}") {
		t.Fatalf("flow block not closed properly:\n%s", out)
	}
}

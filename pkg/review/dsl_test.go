package review

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestRuleDSL_ParseAndEvaluate_ContainerMetadataExists(t *testing.T) {
	src := `rule PublicAPIsRequireRateLimit {
        description "Public APIs must define rate limits."
        applies_to container
        when { metadata.public == "true" }
        ensure { metadata.rate_limit exists }
        severity error
        message "Public APIs must define rate limits."
    }`

	parser, err := BuildRuleParser()
	if err != nil {
		t.Fatalf("parser init error: %v", err)
	}
	rs, err := parser.ParseString("rules.srujarule", src)
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}
	rules := ToRules(rs)

	arch := &language.Architecture{Name: "Test"}
	sys := &language.System{ID: "API", Label: "API"}
	cont := &language.Container{ID: "BillingAPI", Label: "Billing API", Metadata: []*language.MetaEntry{{Key: "public", Value: "true"}}}
	sys.Containers = []*language.Container{cont}
	arch.Systems = []*language.System{sys}

	diags := Evaluate(arch, rules)
	if len(diags) != 1 {
		t.Fatalf("expected 1 diagnostic, got %d", len(diags))
	}
	if diags[0].ElementID != "BillingAPI" {
		t.Fatalf("expected element BillingAPI, got %s", diags[0].ElementID)
	}
}

func TestRuleDSL_RelationOperators_InAndExists(t *testing.T) {
	src := `rule RelationLabelRequired {
        description "Relations with certain verbs must have labels."
        applies_to relation
        when { verb in ["uses", "reads"] }
        ensure { label exists }
        severity warning
        message "Relation should have a label."
    }`

	parser, err := BuildRuleParser()
	if err != nil {
		t.Fatalf("parser init error: %v", err)
	}
	rs, err := parser.ParseString("rules.srujarule", src)
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}
	rules := ToRules(rs)

	v := "uses"
	rel := &language.Relation{From: "A", Arrow: "->", To: "B", Verb: &v}
	arch := &language.Architecture{Name: "Test", Relations: []*language.Relation{rel}}

	diags := Evaluate(arch, rules)
	if len(diags) != 1 {
		t.Fatalf("expected 1 diagnostic, got %d", len(diags))
	}
	if diags[0].ElementID != "A->B" {
		t.Fatalf("expected element A->B, got %s", diags[0].ElementID)
	}
}

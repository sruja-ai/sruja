package policy

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/review"
)

func TestPolicyDSL_ParseAndEvaluate_ComplianceAndViolations(t *testing.T) {
	psrc := `policy apiSecurity {
        description "Security policy for public APIs"
        applies_to container where metadata.public == "true"
        rules { require_rate_limit }
        controls {
            metadata.auth.method == "jwt"
        }
        severity critical
        owner "platform-security"
        version "1.0"
    }`

	pp, err := BuildPolicyParser()
	if err != nil {
		t.Fatalf("policy parser init error: %v", err)
	}
	pf, err := pp.ParseString("policy.srujapolicy", psrc)
	if err != nil {
		t.Fatalf("policy parse error: %v", err)
	}
	if len(pf.Policies) != 1 {
		t.Fatalf("expected 1 policy")
	}
	pol := pf.Policies[0]

	// Architecture with one public container missing rate_limit and auth.method
	arch := &language.Architecture{Name: "Test"}
	sys := &language.System{ID: "API", Label: "API"}
	cont := &language.Container{ID: "PublicAPI", Label: "Public API", Metadata: []*language.MetaEntry{{Key: "public", Value: "true"}}}
	sys.Containers = []*language.Container{cont}
	arch.Systems = []*language.System{sys}

	// Provide rule map with require_rate_limit
	when := &review.Expr{Left: &review.Term{Field: &review.Field{Path: []string{"metadata", "public"}}, Op: strPtr("=="), Value: &review.Value{S: strPtr("true")}}}
	ensure := &review.Expr{Left: &review.Term{Field: &review.Field{Path: []string{"metadata", "rate_limit"}}, Op: strPtr("exists")}}
	rules := map[string]review.Rule{
		"require_rate_limit": {
			ID:        "require_rate_limit",
			AppliesTo: review.AppliesContainer,
			Message:   "Public APIs must define rate limits.",
			Severity:  review.SeverityError,
			When:      when,
			Ensure:    ensure,
		},
	}

	res := EvaluatePolicy(arch, rules, pol)
	if res.PolicyID != "apiSecurity" {
		t.Fatalf("expected policy id apiSecurity, got %s", res.PolicyID)
	}
	if len(res.AppliesTo) != 1 || res.AppliesTo[0] != "PublicAPI" {
		t.Fatalf("expected applies to PublicAPI, got %v", res.AppliesTo)
	}
	if len(res.Violations) != 1 {
		t.Fatalf("expected 1 violation, got %d", len(res.Violations))
	}
	if res.Compliance == 100 {
		t.Fatalf("expected partial compliance, got 100")
	}
	if len(res.ControlsFailed) == 0 {
		t.Fatalf("expected failed controls due to missing auth.method")
	}
}

func strPtr(s string) *string { return &s }

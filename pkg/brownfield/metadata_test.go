package brownfield

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestMetadataHelpers(t *testing.T) {
	meta := []*language.MetaEntry{
		{Key: KeyInferred, Value: "true"},
		{Key: KeyVerified, Value: "false"},
		{Key: KeyLocked, Value: "true"},
		{Key: KeyConfidence, Value: "0.82"},
		{Key: KeyOrigin, Value: "k8s/deploy.yaml; api/payment.go"},
		{Key: KeyExplanation, Value: "Detected from SQL + DTO"},
	}
	if !IsInferred(meta) {
		t.Fatalf("expected inferred true")
	}
	if IsVerified(meta) {
		t.Fatalf("expected verified false")
	}
	if !IsLocked(meta) {
		t.Fatalf("expected locked true")
	}
	if Confidence(meta) < 0.8 {
		t.Fatalf("expected confidence >= 0.8")
	}
	org := Origin(meta)
	if len(org) != 2 {
		t.Fatalf("expected 2 origins, got %d", len(org))
	}
	if Explanation(meta) == "" {
		t.Fatalf("expected explanation")
	}
}

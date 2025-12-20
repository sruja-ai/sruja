//nolint:gocritic // appendCombine acceptable
package converter

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func stringPtr(s string) *string {
	return &s
}

// TestConvertToJSON removed - Architecture struct removed (old syntax no longer supported)
// TestConvertFromJSON removed - Architecture struct removed (old syntax no longer supported)

func TestMetadataConversion(t *testing.T) {
	meta := &language.MetadataBlock{
		Entries: []*language.MetaEntry{
			{
				Key:   "k1",
				Value: stringPtr("v1"),
			},
			{
				Key:   "tags",
				Array: []string{"t1", "t2"},
			},
		},
	}

	// Test To JSON
	jsonMeta := convertMetadataToJSON(meta)
	if len(jsonMeta) != 2 {
		t.Errorf("Expected 2 metadata entries, got %d", len(jsonMeta))
	}
	if jsonMeta[0].Key != "k1" || *jsonMeta[0].Value != "v1" {
		t.Errorf("Mismatch in simple metadata entry")
	}
	if jsonMeta[1].Key != "tags" || len(jsonMeta[1].Array) != 2 {
		t.Errorf("Mismatch in array metadata entry")
	}

	// Test From JSON
	backMeta := convertMetadataFromJSON(jsonMeta)
	if len(backMeta.Entries) != 2 {
		t.Errorf("Expected 2 metadata entries back, got %d", len(backMeta.Entries))
	}
	if backMeta.Entries[0].Key != "k1" {
		t.Errorf("Mismatch in simple metadata entry back")
	}
}

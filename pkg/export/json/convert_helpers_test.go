package json

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestConvertSchemaBlock_OptionalAndGenerics(t *testing.T) {
	// Build TypeSpec with optional and generics
	ts := &language.TypeSpec{Name: "map", Generics: []string{"string", "int"}, Optional: "?"}
	sb := &language.SchemaBlock{Entries: []*language.SchemaEntry{{Key: "field", Type: ts}}}
	out := convertSchemaBlock(sb)
	if out == nil || len(out.Entries) != 1 {
		t.Fatalf("schema not converted")
	}
	if out.Entries[0].Type == nil || out.Entries[0].Type.Name != "map" || !out.Entries[0].Type.Optional {
		t.Fatalf("type spec conversion failed: %+v", out.Entries[0].Type)
	}
}

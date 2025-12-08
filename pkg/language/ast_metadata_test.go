// pkg/language/ast_metadata_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func stringPtrMeta(s string) *string {
	return &s
}

func TestContract_Location(t *testing.T) {
	contract := &language.Contract{
		Kind: "api",
		ID:   "C1",
	}
	loc := contract.Location()
	if loc.File != "" || loc.Line != 0 || loc.Column != 0 {
		t.Errorf("Expected empty SourceLocation, got %+v", loc)
	}
}

func TestGetLabel_Methods(t *testing.T) {
	tests := []struct {
		name string
		elem interface {
			GetLabel() string
		}
		expected string
	}{
		{"System", &language.System{Label: "Sys1"}, "Sys1"},
		{"Container", &language.Container{Label: "Cont1"}, "Cont1"},
		{"Component", &language.Component{Label: "Comp1"}, "Comp1"},
		{"Person", &language.Person{Label: "User1"}, "User1"},
		{"DataStore", &language.DataStore{Label: "DB1"}, "DB1"},
		{"Queue", &language.Queue{Label: "Q1"}, "Q1"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.elem.GetLabel()
			if result != tt.expected {
				t.Errorf("Expected GetLabel() to return %q, got %q", tt.expected, result)
			}
		})
	}
}

func TestMetaString_System(t *testing.T) {
	sys := &language.System{
		Metadata: []*language.MetaEntry{
			{Key: "team", Value: stringPtrMeta("backend")},
			{Key: "tier", Value: stringPtrMeta("critical")},
		},
	}

	value, ok := sys.MetaString("team")
	if !ok {
		t.Error("MetaString should find existing key")
	}
	if value != "backend" {
		t.Errorf("Expected 'backend', got %q", value)
	}

	_, ok = sys.MetaString("nonexistent")
	if ok {
		t.Error("MetaString should return false for non-existent key")
	}
}

func TestMetaString_System_NotFound(t *testing.T) {
	sys := &language.System{
		Metadata: []*language.MetaEntry{
			{Key: "team", Value: stringPtrMeta("backend")},
		},
	}

	value, ok := sys.MetaString("nonexistent")
	if ok {
		t.Error("MetaString should return false for non-existent key")
	}
	if value != "" {
		t.Errorf("MetaString should return empty string for non-existent key, got %q", value)
	}
}

func TestMetaString_Container(t *testing.T) {
	cont := &language.Container{
		Metadata: []*language.MetaEntry{
			{Key: "technology", Value: stringPtrMeta("Go")},
		},
	}

	value, ok := cont.MetaString("technology")
	if !ok {
		t.Error("MetaString should find existing key")
	}
	if value != "Go" {
		t.Errorf("Expected 'Go', got %q", value)
	}
}

func TestMetaString_Component(t *testing.T) {
	comp := &language.Component{
		Metadata: []*language.MetaEntry{
			{Key: "critical", Value: stringPtrMeta("true")},
		},
	}

	value, ok := comp.MetaString("critical")
	if !ok {
		t.Error("MetaString should find existing key")
	}
	if value != "true" {
		t.Errorf("Expected 'true', got %q", value)
	}
}

func TestMetaString_DataStore(t *testing.T) {
	ds := &language.DataStore{
		Metadata: []*language.MetaEntry{
			{Key: "engine", Value: stringPtrMeta("postgres")},
		},
	}

	value, ok := ds.MetaString("engine")
	if !ok {
		t.Error("MetaString should find existing key")
	}
	if value != "postgres" {
		t.Errorf("Expected 'postgres', got %q", value)
	}
}

func TestMetaString_Queue(t *testing.T) {
	q := &language.Queue{
		Metadata: []*language.MetaEntry{
			{Key: "topic", Value: stringPtrMeta("events")},
		},
	}

	value, ok := q.MetaString("topic")
	if !ok {
		t.Error("MetaString should find existing key")
	}
	if value != "events" {
		t.Errorf("Expected 'events', got %q", value)
	}
}

func TestMetaString_Person(t *testing.T) {
	p := &language.Person{
		Metadata: []*language.MetaEntry{
			{Key: "persona", Value: stringPtrMeta("customer")},
		},
	}

	value, ok := p.MetaString("persona")
	if !ok {
		t.Error("MetaString should find existing key")
	}
	if value != "customer" {
		t.Errorf("Expected 'customer', got %q", value)
	}
}

func TestMetaString_Architecture(t *testing.T) {
	arch := &language.Architecture{
		Metadata: []*language.MetaEntry{
			{Key: "level", Value: stringPtrMeta("arch")},
		},
	}

	value, ok := arch.MetaString("level")
	if !ok {
		t.Error("MetaString should find existing key")
	}
	if value != "arch" {
		t.Errorf("Expected 'arch', got %q", value)
	}
}

func TestHasMeta_System(t *testing.T) {
	sys := &language.System{
		Metadata: []*language.MetaEntry{
			{Key: "team", Value: stringPtrMeta("backend")},
		},
	}

	if !sys.HasMeta("team") {
		t.Error("HasMeta should return true for existing key")
	}
	if sys.HasMeta("nonexistent") {
		t.Error("HasMeta should return false for non-existent key")
	}
}

func TestHasMeta_Container(t *testing.T) {
	cont := &language.Container{
		Metadata: []*language.MetaEntry{
			{Key: "technology", Value: stringPtrMeta("Go")},
		},
	}

	if !cont.HasMeta("technology") {
		t.Error("HasMeta should return true for existing key")
	}
}

func TestHasMeta_Component(t *testing.T) {
	comp := &language.Component{
		Metadata: []*language.MetaEntry{
			{Key: "critical", Value: stringPtrMeta("true")},
		},
	}

	if !comp.HasMeta("critical") {
		t.Error("HasMeta should return true for existing key")
	}
}

func TestHasMeta_DataStore(t *testing.T) {
	ds := &language.DataStore{
		Metadata: []*language.MetaEntry{
			{Key: "engine", Value: stringPtrMeta("postgres")},
		},
	}

	if !ds.HasMeta("engine") {
		t.Error("HasMeta should return true for existing key")
	}
}

func TestHasMeta_Queue(t *testing.T) {
	q := &language.Queue{
		Metadata: []*language.MetaEntry{
			{Key: "topic", Value: stringPtrMeta("events")},
		},
	}

	if !q.HasMeta("topic") {
		t.Error("HasMeta should return true for existing key")
	}
}

func TestHasMeta_Person(t *testing.T) {
	p := &language.Person{
		Metadata: []*language.MetaEntry{
			{Key: "persona", Value: stringPtrMeta("customer")},
		},
	}

	if !p.HasMeta("persona") {
		t.Error("HasMeta should return true for existing key")
	}
}

func TestHasMeta_Architecture(t *testing.T) {
	arch := &language.Architecture{
		Metadata: []*language.MetaEntry{
			{Key: "level", Value: stringPtrMeta("arch")},
		},
	}

	if !arch.HasMeta("level") {
		t.Error("HasMeta should return true for existing key")
	}
}

func TestAllMetadata_System(t *testing.T) {
	sys := &language.System{
		Metadata: []*language.MetaEntry{
			{Key: "team", Value: stringPtrMeta("backend")},
			{Key: "tier", Value: stringPtrMeta("critical")},
		},
	}

	result := sys.AllMetadata()
	if len(result) != 2 {
		t.Errorf("Expected 2 metadata entries, got %d", len(result))
	}
	if result["team"] != "backend" {
		t.Errorf("Expected team='backend', got %q", result["team"])
	}
}

func TestAllMetadata_Container(t *testing.T) {
	cont := &language.Container{
		Metadata: []*language.MetaEntry{
			{Key: "technology", Value: stringPtrMeta("Go")},
		},
	}

	result := cont.AllMetadata()
	if len(result) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(result))
	}
}

func TestAllMetadata_Component(t *testing.T) {
	comp := &language.Component{
		Metadata: []*language.MetaEntry{
			{Key: "critical", Value: stringPtrMeta("true")},
		},
	}

	result := comp.AllMetadata()
	if len(result) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(result))
	}
}

func TestAllMetadata_DataStore(t *testing.T) {
	ds := &language.DataStore{
		Metadata: []*language.MetaEntry{
			{Key: "engine", Value: stringPtrMeta("postgres")},
		},
	}

	result := ds.AllMetadata()
	if len(result) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(result))
	}
}

func TestAllMetadata_Queue(t *testing.T) {
	q := &language.Queue{
		Metadata: []*language.MetaEntry{
			{Key: "topic", Value: stringPtrMeta("events")},
		},
	}

	result := q.AllMetadata()
	if len(result) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(result))
	}
}

func TestAllMetadata_Person(t *testing.T) {
	p := &language.Person{
		Metadata: []*language.MetaEntry{
			{Key: "persona", Value: stringPtrMeta("customer")},
		},
	}

	result := p.AllMetadata()
	if len(result) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(result))
	}
}

func TestAllMetadata_Architecture(t *testing.T) {
	arch := &language.Architecture{
		Metadata: []*language.MetaEntry{
			{Key: "level", Value: stringPtrMeta("arch")},
		},
	}

	result := arch.AllMetadata()
	if len(result) != 1 {
		t.Errorf("Expected 1 metadata entry, got %d", len(result))
	}
}

func TestAllMetadata_Empty(t *testing.T) {
	sys := &language.System{
		Metadata: []*language.MetaEntry{},
	}

	result := sys.AllMetadata()
	if len(result) != 0 {
		t.Errorf("Expected 0 metadata entries, got %d", len(result))
	}
}

func TestMetaMap_System(t *testing.T) {
	sys := &language.System{
		Metadata: []*language.MetaEntry{
			{Key: "team.backend", Value: stringPtrMeta("api")},
			{Key: "team.frontend", Value: stringPtrMeta("ui")},
			{Key: "tier", Value: stringPtrMeta("critical")},
		},
	}

	result := sys.MetaMap("team.")
	if len(result) != 2 {
		t.Errorf("Expected 2 metadata entries with prefix, got %d", len(result))
	}
	if result["team.backend"] != "api" {
		t.Errorf("Expected team.backend='api', got %q", result["team.backend"])
	}
}

func TestMetaMap_Container(t *testing.T) {
	cont := &language.Container{
		Metadata: []*language.MetaEntry{
			{Key: "env.prod", Value: stringPtrMeta("production")},
			{Key: "env.dev", Value: stringPtrMeta("development")},
		},
	}

	result := cont.MetaMap("env.")
	if len(result) != 2 {
		t.Errorf("Expected 2 metadata entries with prefix, got %d", len(result))
	}
}

func TestMetaMap_Component(t *testing.T) {
	comp := &language.Component{
		Metadata: []*language.MetaEntry{
			{Key: "config.timeout", Value: stringPtrMeta("30s")},
			{Key: "config.retries", Value: stringPtrMeta("3")},
		},
	}

	result := comp.MetaMap("config.")
	if len(result) != 2 {
		t.Errorf("Expected 2 metadata entries with prefix, got %d", len(result))
	}
}

func TestMetaMap_NoMatch(t *testing.T) {
	sys := &language.System{
		Metadata: []*language.MetaEntry{
			{Key: "team", Value: stringPtrMeta("backend")},
		},
	}

	result := sys.MetaMap("nonexistent.")
	if len(result) != 0 {
		t.Errorf("Expected 0 metadata entries, got %d", len(result))
	}
}

func TestMetaMap_EmptyPrefix(t *testing.T) {
	sys := &language.System{
		Metadata: []*language.MetaEntry{
			{Key: "team", Value: stringPtrMeta("backend")},
		},
	}

	result := sys.MetaMap("")
	if len(result) != 1 {
		t.Errorf("Expected 1 metadata entry with empty prefix, got %d", len(result))
	}
}

func TestMetaMap_KeyShorterThanPrefix(t *testing.T) {
	sys := &language.System{
		Metadata: []*language.MetaEntry{
			{Key: "a", Value: stringPtr("value")},
		},
	}

	result := sys.MetaMap("longprefix.")
	if len(result) != 0 {
		t.Errorf("Expected 0 metadata entries when key is shorter than prefix, got %d", len(result))
	}
}

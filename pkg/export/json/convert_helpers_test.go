package json

import (
    "testing"
    "github.com/sruja-ai/sruja/pkg/language"
)

func TestConvertArchitecture_SLOAndContainerExtraction(t *testing.T) {
    // Build system with SLO
    avail := "99.9%"
    window := "30d"
    sys := &language.System{ID: "S", Label: "Sys", SLO: &language.SLOBlock{Items: []*language.SLOItem{{Availability: &language.SLOAvailability{Target: &avail, Window: &window}}}}}
    sys.SLO.PostProcess()

    // Container with technology and tags via Items
    tech := "Go"
    cont := &language.Container{ID: "C", Label: "Cont", Items: []language.ContainerItem{{Technology: &tech, Tags: []string{"api", "backend"}}}}
    sys.Containers = []*language.Container{cont}

    arch := &language.Architecture{Name: "A", Systems: []*language.System{sys}}

    body := convertArchitectureToJSON(arch)
    if len(body.Systems) != 1 { t.Fatalf("expected 1 system") }
    if body.Systems[0].SLO == nil || body.Systems[0].SLO.Availability == nil || body.Systems[0].SLO.Availability.Target != avail { t.Fatalf("SLO availability not converted correctly: %+v", body.Systems[0].SLO) }
    if len(body.Systems[0].Containers) != 1 { t.Fatalf("expected 1 container") }
    c := body.Systems[0].Containers[0]
    if c.Technology == nil || *c.Technology != tech { t.Fatalf("container technology not extracted: %+v", c) }
    if len(c.Tags) != 2 { t.Fatalf("container tags not extracted: %+v", c.Tags) }
}

func TestConvertSchemaBlock_OptionalAndGenerics(t *testing.T) {
    // Build TypeSpec with optional and generics
    ts := &language.TypeSpec{Name: "map", Generics: []string{"string", "int"}, Optional: "?"}
    sb := &language.SchemaBlock{Entries: []*language.SchemaEntry{{Key: "field", Type: ts}}}
    out := convertSchemaBlock(sb)
    if out == nil || len(out.Entries) != 1 { t.Fatalf("schema not converted") }
    if out.Entries[0].Type == nil || out.Entries[0].Type.Name != "map" || !out.Entries[0].Type.Optional { t.Fatalf("type spec conversion failed: %+v", out.Entries[0].Type) }
}


package language

import (
    "strings"
    "testing"
)

func TestPrinter_printChangeBlock_PrintsSections(t *testing.T) {
    ver := "1.0"
    status := "approved"
    req := "R1"
    adr := "ADR-1"
    change := &ChangeBlock{
        ID: "Ch1",
        Version: &ver,
        Status: &status,
        Requirement: &req,
        ADR: &adr,
        Add: &ArchitectureBlock{Items: []ArchitectureItem{{System: &System{ID: "S", Label: "Shop"}}}},
        Modify: &ArchitectureBlock{Items: []ArchitectureItem{{Container: &Container{ID: "Web", Label: "Web"}}}},
        Remove: &ArchitectureBlock{Items: []ArchitectureItem{{Component: &Component{ID: "Svc", Label: "Service"}}}},
    }

    var sb strings.Builder
    p := NewPrinter()
    p.printChangeBlock(&sb, change)
    out := sb.String()

    checks := []string{"change \"Ch1\" {", "version \"1.0\"", "status \"approved\"", "requirement \"R1\"", "adr \"ADR-1\"", "add {", "system S \"Shop\"", "modify {", "container Web \"Web\"", "remove {", "component Svc \"Service\""}
    for _, c := range checks {
        if !strings.Contains(out, c) {
            t.Fatalf("missing %q in output:\n%s", c, out)
        }
    }
}

func TestPrinter_printSnapshotBlock_PrintsFields(t *testing.T) {
    ver := "2.0"
    desc := "Release snapshot"
    ts := "2025-01-01T00:00:00Z"
    preview := true
    snap := &SnapshotBlock{
        Name: "Snap1",
        Version: &ver,
        Description: &desc,
        Timestamp: &ts,
        Preview: &preview,
        Changes: []string{"Ch1", "Ch2"},
        ArchName: ptr("Arch"),
        Architecture: &Architecture{Name: "Arch"},
    }

    var sb strings.Builder
    p := NewPrinter()
    p.printSnapshotBlock(&sb, snap)
    out := sb.String()

    checks := []string{"snapshot \"Snap1\" {", "version \"2.0\"", "description \"Release snapshot\"", "timestamp \"2025-01-01T00:00:00Z\"", "preview true", "changes [\"Ch1\", \"Ch2\"]", "architecture \"Arch\" {"}
    for _, c := range checks {
        if !strings.Contains(out, c) {
            t.Fatalf("missing %q in output:\n%s", c, out)
        }
    }
}

func ptr[T any](v T) *T { return &v }


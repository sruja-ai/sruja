package language

import (
    "strings"
    "testing"
)

func TestPrinter_printLibrary_WithItems(t *testing.T) {
    ver := "1.2.3"
    owner := "platform"
    d := "Use for API"
    req := &Requirement{ID: "R1", Description: ptr("Rule"), Type: ptr("quality")}
    pol := &Policy{ID: "P1", Description: "Policy", Category: ptr("compliance"), Enforcement: ptr("mandatory")}
    mval := "stable"
    lib := &Library{
        ID: "Lib",
        Label: "Library",
        Version: &ver,
        Owner: &owner,
        Items: []*LibraryItem{
            {Description: &d},
            {Policy: pol},
            {Requirement: req},
            {Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "status", Value: &mval}}}},
        },
    }

    var sb strings.Builder
    p := NewPrinter()
    p.printLibrary(&sb, lib)
    out := sb.String()
    checks := []string{"library Lib \"Library\" version \"1.2.3\" owner \"platform\"", "description \"Use for API\"", "policy P1", "requirement R1", "metadata {", "status \"stable\""}
    for _, c := range checks {
        if !strings.Contains(out, c) {
            t.Fatalf("missing %q in output:\n%s", c, out)
        }
    }
}


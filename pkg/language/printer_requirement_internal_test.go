package language

import (
    "strings"
    "testing"
)

func TestPrinter_printRequirement_BodyAndInline(t *testing.T) {
    desc := "Encrypt data"
    typ := "security"
    // Inline-only
    inline := &Requirement{ID: "R1", Type: &typ, Description: &desc}
    // With body and metadata
    mval := "high"
    body := &RequirementBody{Type: &typ, Description: &desc, Metadata: &MetadataBlock{Entries: []*MetaEntry{{Key: "priority", Value: &mval}}}}
    withBody := &Requirement{ID: "R2", Body: body}

    var sb1 strings.Builder
    p := NewPrinter()
    p.printRequirement(&sb1, inline)
    out1 := sb1.String()
    if !strings.Contains(out1, "requirement R1 security \"Encrypt data\"") {
        t.Fatalf("inline requirement not printed correctly: %s", out1)
    }

    var sb2 strings.Builder
    p.printRequirement(&sb2, withBody)
    out2 := sb2.String()
    checks := []string{"requirement R2 {", "type \"security\"", "description \"Encrypt data\"", "metadata {", "priority \"high\""}
    for _, c := range checks {
        if !strings.Contains(out2, c) {
            t.Fatalf("missing %q in output: %s", c, out2)
        }
    }
}


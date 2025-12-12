package language

import (
    "strings"
    "testing"
)

func TestPrinter_printRelation_VerbFormsAndTags(t *testing.T) {
    verbQuoted := "calls"
    verbIdent := "uses"
    label := "HTTP"
    rel1 := &Relation{From: QualifiedIdent{Parts: []string{"A"}}, To: QualifiedIdent{Parts: []string{"B"}}, Verb: &verbQuoted, Label: &label, Tags: []string{"sync", "critical"}}
    rel2 := &Relation{From: QualifiedIdent{Parts: []string{"B"}}, To: QualifiedIdent{Parts: []string{"C"}}, Verb: &verbIdent}

    var sb strings.Builder
    p := NewPrinter()
    p.printRelation(&sb, rel1)
    p.printRelation(&sb, rel2)
    out := sb.String()
    checks := []string{"A -> B [sync, critical] calls \"HTTP\"", "B -> C uses"}
    for _, c := range checks {
        if !strings.Contains(out, c) {
            t.Fatalf("missing %q in output:\n%s", c, out)
        }
    }
}

func TestPrinter_printMetadataBlock_ArrayAndScalar(t *testing.T) {
    v := "v1"
    mb := &MetadataBlock{Entries: []*MetaEntry{{Key: "version", Value: &v}, {Key: "owners", Array: []string{"platform", "security"}}}}
    var sb strings.Builder
    p := NewPrinter()
    p.printMetadataBlock(&sb, mb)
    out := sb.String()
    checks := []string{"metadata {", "version \"v1\"", "owners [\"platform\", \"security\"]"}
    for _, c := range checks {
        if !strings.Contains(out, c) {
            t.Fatalf("missing %q in output:\n%s", c, out)
        }
    }
}

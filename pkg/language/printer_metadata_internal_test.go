package language

import (
    "strings"
    "testing"
)

func TestPrinter_printMetadata_ScalarAndArray(t *testing.T) {
    v := "val"
    entries := []*MetaEntry{{Key: "k", Value: &v}, {Key: "arr", Array: []string{"a", "b"}}}
    var sb strings.Builder
    p := NewPrinter()
    p.printMetadata(&sb, entries)
    out := sb.String()
    if !strings.Contains(out, "metadata {") || !strings.Contains(out, "k \"val\"") || !strings.Contains(out, "arr [\"a\", \"b\"]") {
        t.Fatalf("unexpected metadata output:\n%s", out)
    }
}


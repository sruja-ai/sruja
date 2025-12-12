package lsp

import (
    "testing"
    "github.com/sourcegraph/go-lsp"
)

func TestFindDefinitionRange_MatchesDeclarations(t *testing.T) {
    text := "architecture \"A\" {\n" +
        "system S \"Shop\" {\n" +
        "  container WebApp \"Web App\"\n" +
        "  component Service \"Svc\"\n" +
        "  datastore DB \"Database\"\n" +
        "  queue Q \"Queue\"\n" +
        "}\n" +
        "person P \"Person\"\n" +
        "}\n"
    doc := NewDocument(lsp.DocumentURI("file://def.sruja"), text, 1)

    cases := []struct{
        id string
        line int
    }{
        {"S", 1},
        {"WebApp", 2},
        {"Service", 3},
        {"DB", 4},
        {"Q", 5},
        {"P", 7},
    }
    for _, c := range cases {
        r := findDefinitionRange(doc, c.id)
        if r.Start.Line != c.line || r.End.Line != c.line {
            t.Fatalf("id %s expected line %d, got %+v", c.id, c.line, r)
        }
        if r.End.Character-r.Start.Character != len(c.id) {
            t.Fatalf("id %s unexpected char span: %+v", c.id, r)
        }
    }
}


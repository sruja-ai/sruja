package json

import (
    "testing"
    "encoding/json"
    "github.com/sruja-ai/sruja/pkg/language"
)

func TestJSONExport_PreservesImports(t *testing.T) {
    dsl := `architecture "Test" {
  import "shared.sruja"
  import "billing.sruja" as Billing
  person Customer "Customer"
  system API "API Service" {}
}`

    p, err := language.NewParser()
    if err != nil { t.Fatalf("parser error: %v", err) }
    prog, err := p.Parse("test.sruja", dsl)
    if err != nil { t.Fatalf("parse error: %v", err) }

    e := NewExporter()
    out, err := e.Export(prog.Architecture)
    if err != nil { t.Fatalf("export error: %v", err) }

    var doc ArchitectureJSON
    if err := json.Unmarshal([]byte(out), &doc); err != nil {
        t.Fatalf("json unmarshal: %v", err)
    }

    if len(doc.Architecture.Imports) != 2 {
        t.Fatalf("expected 2 imports, got %d", len(doc.Architecture.Imports))
    }
    if doc.Architecture.Imports[0].Path != "shared.sruja" || doc.Architecture.Imports[0].Alias != nil {
        t.Fatalf("first import mismatch: %+v", doc.Architecture.Imports[0])
    }
    if doc.Architecture.Imports[1].Path != "billing.sruja" {
        t.Fatalf("second import path mismatch: %+v", doc.Architecture.Imports[1])
    }
    if doc.Architecture.Imports[1].Alias == nil || *doc.Architecture.Imports[1].Alias != "Billing" {
        t.Fatalf("second import alias mismatch: %+v", doc.Architecture.Imports[1])
    }
}


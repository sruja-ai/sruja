package main

import (
	"os"
	"path/filepath"
	"testing"

	glsp "github.com/sourcegraph/go-lsp"
	slsp "github.com/sruja-ai/sruja/pkg/lsp"
)

func TestFindDefinitionOnExamples(t *testing.T) {
	path := filepath.Join("..", "..", "examples", "sruja_architecture_v2.sruja")
	b, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read example: %v", err)
	}
	ws := slsp.NewWorkspace()
	uri := glsp.DocumentURI(path)
	ws.AddDocument(uri, string(b), 1)

	ids := []string{"Sruja.CLI", "Sruja.Engine", "Sruja.Language", "Sruja.LSP", "Sruja.WASM", "Sruja.Playground", "Sruja.Website"}
	for _, id := range ids {
		u, r, ok := ws.FindDefinition(id)
		if !ok {
			t.Errorf("definition not found for %s", id)
			continue
		}
		if u != uri {
			t.Errorf("uri mismatch for %s: got %s want %s", id, u, uri)
		}
		if r.Start.Line == 0 && r.Start.Character == 0 && r.End.Line == 0 && r.End.Character == 0 {
			t.Errorf("empty range for %s", id)
		}
	}
}

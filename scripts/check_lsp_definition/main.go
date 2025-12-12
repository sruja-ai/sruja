package main

import (
	"fmt"
	"os"

	glsp "github.com/sourcegraph/go-lsp"
	slsp "github.com/sruja-ai/sruja/pkg/lsp"
)

func main() {
	// Load the example file
	path := "examples/sruja_architecture_v2.sruja"
	b, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}
	ws := slsp.NewWorkspace()
	uri := glsp.DocumentURI(path)
	ws.AddDocument(uri, string(b), 1)

	// Check qualified ID resolution
	ids := []string{"Sruja.CLI", "Sruja.Engine", "Sruja.Language", "Sruja.LSP", "Sruja.WASM", "Sruja.Playground", "Sruja.Website"}
	for _, id := range ids {
		u, r, ok := ws.FindDefinition(id)
		fmt.Printf("%s -> ok=%v uri=%s range=[%d:%d-%d:%d]\n", id, ok, u, r.Start.Line, r.Start.Character, r.End.Line, r.End.Character)
	}
}

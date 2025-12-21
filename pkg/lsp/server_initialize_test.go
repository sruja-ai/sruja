package lsp

import (
	"context"
	"github.com/sourcegraph/go-lsp"
	"testing"
)

func TestServerInitializeCapabilities(t *testing.T) {
	s := NewServer()
	res, err := s.Initialize(context.Background(), lsp.InitializeParams{})
	if err != nil {
		t.Fatalf("initialize error: %v", err)
	}
	if res == nil || !res.Capabilities.HoverProvider || !res.Capabilities.DefinitionProvider || !res.Capabilities.DocumentFormattingProvider {
		t.Fatalf("capabilities missing")
	}
}

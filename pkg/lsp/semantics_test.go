package lsp

import (
	"context"
	"github.com/sourcegraph/go-lsp"
	"testing"
)

func TestSemanticTokensBasic(t *testing.T) {
	srv := NewServer()
	uri := lsp.DocumentURI("file:///sem.sruja")
	text := "system S \"System\"\nS -> C \"uses\"\ncontainer C \"Container\""
	srv.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{TextDocument: lsp.TextDocumentItem{URI: uri, Text: text}})

	legend := srv.SemanticTokensLegend()
	toks, err := srv.SemanticTokensFull(context.Background(), lsp.TextDocumentIdentifier{URI: uri})
	if err != nil {
		t.Fatalf("SemanticTokensFull failed: %v", err)
	}
	if len(toks.Data) == 0 {
		t.Fatalf("Expected some semantic tokens")
	}

	typeIdx := func(name string) int {
		for i, n := range legend.TokenTypes {
			if n == name {
				return i
			}
		}
		return -1
	}

	hasClassDecl := false
	hasOperator := false
	hasKeyword := false

	var prevLine, prevChar uint32
	for i := 0; i+4 < len(toks.Data); i += 5 {
		dl := toks.Data[i]
		ds := toks.Data[i+1]
		length := toks.Data[i+2]
		ttype := toks.Data[i+3]
		mods := toks.Data[i+4]
		line := prevLine + dl
		start := ds
		if dl == 0 {
			start = prevChar + ds
		}
		prevLine = line
		prevChar = start

		if int(ttype) == typeIdx("keyword") && line == 0 {
			hasKeyword = true
		}
		if int(ttype) == typeIdx("operator") && line == 1 && length == 2 {
			hasOperator = true
		}
		if int(ttype) == typeIdx("class") && (mods&1) == 1 {
			hasClassDecl = true
		}
	}

	if !hasKeyword {
		t.Fatalf("Expected keyword token on line 0")
	}
	if !hasOperator {
		t.Fatalf("Expected operator token '->' on line 1")
	}
	if !hasClassDecl {
		t.Fatalf("Expected class declaration token for 'S'")
	}
}

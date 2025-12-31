package lsp

import (
	"context"
	"testing"

	"github.com/sourcegraph/go-lsp"
	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

func TestCodeAction_GeneratesActionsForDiagnostics(t *testing.T) {
	s := NewServer()
	uri := lsp.DocumentURI("file:///actions.sruja")
	text := "System=kind \"System\"\nContainer=kind \"Container\"\nS = System \"S\" {\n    Cont = Container \"Container\"\n}\n"
	_ = s.DidOpen(context.Background(), lsp.DidOpenTextDocumentParams{
		TextDocument: lsp.TextDocumentItem{URI: uri, Text: text, Version: 1},
	})

	diags := []lsp.Diagnostic{
		{
			Code:    diagnostics.CodeReferenceNotFound,
			Message: "Undefined reference 'Cont1'",
			Range: lsp.Range{
				Start: lsp.Position{Line: 3, Character: 14}, // Line 3: Cont = ...
				End:   lsp.Position{Line: 3, Character: 19},
			},
		},
		{
			Code:    diagnostics.CodeDuplicateIdentifier,
			Message: "Duplicate ID 'API'",
			Range: lsp.Range{
				Start: lsp.Position{Line: 0, Character: 0},
				End:   lsp.Position{Line: 0, Character: 3},
			},
		},
		{
			Code:    diagnostics.CodeOrphanElement,
			Message: "Orphan element 'Unused'",
			Range: lsp.Range{
				Start: lsp.Position{Line: 2, Character: 2}, // S
				End:   lsp.Position{Line: 2, Character: 8},
			},
		},
		{
			Code:    diagnostics.CodeUnexpectedToken,
			Message: "unexpected token 'foo'",
			Range: lsp.Range{
				Start: lsp.Position{Line: 1, Character: 0},
				End:   lsp.Position{Line: 1, Character: 3},
			},
		},
	}

	params := lsp.CodeActionParams{
		TextDocument: lsp.TextDocumentIdentifier{URI: uri},
		Context:      lsp.CodeActionContext{Diagnostics: diags},
	}
	actions, err := s.CodeAction(context.Background(), params)
	if err != nil {
		t.Fatalf("CodeAction error: %v", err)
	}
	if len(actions) == 0 {
		t.Fatalf("expected some actions, got 0")
	}

	var hasReplace bool
	var hasRename bool
	var hasAddRelation bool
	var hasRemoveToken bool
	for _, a := range actions {
		if a.Command == "sruja.replaceElement" {
			hasReplace = true
		}
		if a.Command == "sruja.renameElement" && a.Title != "" {
			hasRename = true
		}
		if a.Command == "sruja.addRelation" {
			hasAddRelation = true
		}
		if a.Command == "sruja.removeToken" {
			hasRemoveToken = true
		}
	}
	if !hasReplace {
		t.Fatalf("expected replaceElement action for undefined reference")
	}
	if !hasRename {
		t.Fatalf("expected renameElement action for duplicate identifier")
	}
	if !hasAddRelation {
		t.Fatalf("expected addRelation action for orphan element")
	}
	if !hasRemoveToken {
		t.Fatalf("expected removeToken action for unexpected token")
	}
}

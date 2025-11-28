package d2

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExport_Journey(t *testing.T) {
	dsl := `
architecture "Journey Test" {
	system Sys1 "System 1"
	person User "User"

	journey UserJourney {
		title "User Journey"
		steps {
			User -> Sys1 "Login"
			Sys1 -> User "Welcome"
		}
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	program.Architecture.PostProcess()

	exporter := NewExporter()
	output, err := exporter.Export(program.Architecture)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	expected := []string{
		"layers: {",
		"\"User Journey\"",
		"\"Architecture\"",
		"}",
		"\"User Journey\": {",
		"_Title: \"User Journey\" {",
		"shape: text",
		"User -> Sys1: \"1. Login\"",
		"Sys1 -> User: \"2. Welcome\"",
	}

	for _, exp := range expected {
		if !strings.Contains(output, exp) {
			t.Errorf("Expected output to contain:\n%s\nBut got:\n%s", exp, output)
		}
	}
}

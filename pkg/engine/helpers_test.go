// pkg/engine/helpers_test.go
package engine_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func parse(t *testing.T, dsl string) *language.Program {
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}
	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}
	return program
}

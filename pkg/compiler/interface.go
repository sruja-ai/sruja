package compiler

import "github.com/sruja-ai/sruja/pkg/language"

// Compiler is the interface that all compilers must implement
type Compiler interface {
	// Name returns the compiler name (e.g., "mermaid", "plantuml")
	Name() string

	// Compile takes a Sruja program and returns the compiled output
	Compile(program *language.Program) (string, error)
}

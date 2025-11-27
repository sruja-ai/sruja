package extensions

import (
	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/engine"
)

// Extension is the base interface for all extensions
type Extension interface {
	Name() string
	Version() string
}

// RuleExtension provides custom validation rules
type RuleExtension interface {
	Extension
	Rules() []engine.Rule
}

// CompilerExtension provides custom compilers
type CompilerExtension interface {
	Extension
	Compiler() compiler.Compiler
}

// Compiler interface (to be implemented by compilers)
// This is a placeholder - we'll need to define this in pkg/compiler
// For now, we'll use a simple interface

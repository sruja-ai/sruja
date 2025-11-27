// Package compiler provides compilation from model to various diagram formats.
package compiler

import (
	"fmt"
	"sync"

	"github.com/sruja-ai/sruja/pkg/language"
)

// Registry manages available diagram format compilers.
//
// The registry allows:
//   - Registering new compilers
//   - Looking up compilers by name
//   - Listing all available formats
//   - Automatic format selection based on model characteristics
type Registry struct {
	compilers map[string]Compiler
	mu        sync.RWMutex
}

// NewRegistry creates a new compiler registry with default compilers.
func NewRegistry() *Registry {
	r := &Registry{
		compilers: make(map[string]Compiler),
	}

	// Register default compilers
	r.Register(NewD2Compiler())
	r.Register(NewMermaidCompiler())

	return r
}

// Register registers a compiler in the registry.
func (r *Registry) Register(compiler Compiler) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.compilers[compiler.Name()] = compiler
}

// Get retrieves a compiler by name.
func (r *Registry) Get(name string) (Compiler, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	compiler, ok := r.compilers[name]
	if !ok {
		return nil, fmt.Errorf("compiler '%s' not found. Available: %v", name, r.List())
	}
	return compiler, nil
}

// List returns all registered compiler names.
func (r *Registry) List() []string {
	r.mu.RLock()
	defer r.mu.RUnlock()
	names := make([]string, 0, len(r.compilers))
	for name := range r.compilers {
		names = append(names, name)
	}
	return names
}

// Compile compiles a program using the specified compiler.
func (r *Registry) Compile(format string, program *language.Program) (string, error) {
	compiler, err := r.Get(format)
	if err != nil {
		return "", err
	}
	return compiler.Compile(program)
}

// DefaultRegistry is the global default registry.
var DefaultRegistry = NewRegistry()

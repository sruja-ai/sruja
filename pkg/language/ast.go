// pkg/language/ast.go
// Package language provides DSL parsing and AST (Abstract Syntax Tree) structures.
//
// The AST represents the parsed structure of a Sruja DSL file. Each node in the AST
// corresponds to a construct in the DSL (architecture, system, container, etc.) and
// includes source location information for error reporting and IDE features.
//
// Example usage:
//
//	parser, _ := language.NewParser()
//	program, _ := parser.Parse("example.sruja", dslText)
//	model := program.Model
//	// Access elements via model.Items
package language

// This file is intentionally empty.
// AST definitions have been modularized into:
// - ast_core.go: Base types and helpers
// - ast_architecture.go: Top-level architecture structures
// - ast_elements.go: Core C4 elements (System, Container, Component, etc.)
// - ast_relations.go: Relations
// - ast_requirements.go: Requirements and ADRs
// - ast_library.go: Policies
// - ast_scenarios.go: Scenarios and Flows
// - ast_deployment.go: Deployment nodes

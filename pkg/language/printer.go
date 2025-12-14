// pkg/language/printer.go
// Package language provides DSL parsing and AST structures.
//
//nolint:gocritic // Printer logic triggers many style warnings (preferFprint, nestingReduce)
package language

import (
	"strings"
)

//nolint:gocritic // Printer logic triggers many style warnings (preferFprint, nestingReduce)

// Printer prints an AST back to DSL format.
type Printer struct {
	IndentLevel int
}

// NewPrinter creates a new printer.
func NewPrinter() *Printer {
	return &Printer{IndentLevel: 0}
}

// indentCache holds pre-computed indentation strings
var indentCache = make([]string, 20)

func init() {
	for i := range indentCache {
		indentCache[i] = strings.Repeat("  ", i)
	}
}

// indent returns the indentation string for the current indent level.
func (p *Printer) indent() string {
	if p.IndentLevel < len(indentCache) {
		return indentCache[p.IndentLevel]
	}
	return strings.Repeat("  ", p.IndentLevel)
}

// Print prints a Program back to DSL format.
func (p *Printer) Print(program *Program) string {
	if program == nil || program.Architecture == nil {
		return ""
	}

	// Pre-allocate builder with estimated capacity
	estimatedSize := estimateOutputSize(program.Architecture)
	sb := strings.Builder{}
	sb.Grow(estimatedSize)
	p.printArchitecture(&sb, program.Architecture)
	return sb.String()
}

// estimateOutputSize provides a rough estimate of output size for builder pre-allocation.
func estimateOutputSize(arch *Architecture) int {
	if arch == nil {
		return 1024
	}
	// Rough estimate: ~100 bytes per element
	count := len(arch.Systems) + len(arch.Persons) + len(arch.Relations) +
		len(arch.Requirements) + len(arch.ADRs) + len(arch.Policies) +
		len(arch.Flows) + len(arch.Libraries)
	return count*100 + 512 // Add buffer
}

// printArchitecture prints an architecture node.
func (p *Printer) printArchitecture(sb *strings.Builder, arch *Architecture) {
	indent := p.indent()
	sb.WriteString(indent)
	sb.WriteString("architecture \"")
	sb.WriteString(arch.Name)
	sb.WriteString("\" {\n")
	p.IndentLevel++

	// Print systems
	for _, sys := range arch.Systems {
		p.printSystem(sb, sys)
	}

	// Print persons
	for _, person := range arch.Persons {
		p.printPerson(sb, person)
	}

	// Print relations
	for _, rel := range arch.Relations {
		p.printRelation(sb, rel)
	}

	// Print requirements
	for _, req := range arch.Requirements {
		p.printRequirement(sb, req)
	}

	// Print ADRs
	for _, adr := range arch.ADRs {
		p.printADR(sb, adr)
	}

	// Print Policies
	for _, policy := range arch.Policies {
		p.printPolicy(sb, policy)
	}

	// Print Flows
	for _, flow := range arch.Flows {
		p.printFlow(sb, flow)
	}

	// Print shared artifacts
	for _, sa := range arch.SharedArtifacts {
		p.printSharedArtifact(sb, sa)
	}

	// Print libraries
	for _, lib := range arch.Libraries {
		p.printLibrary(sb, lib)
	}

	// Print architecture-level metadata
	if len(arch.Metadata) > 0 {
		p.printMetadata(sb, arch.Metadata)
	}

	p.IndentLevel--
	indent = p.indent()
	sb.WriteString(indent)
	sb.WriteString("}\n")
}

package mermaid

import (
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

// Config represents Mermaid diagram configuration.
type Config struct {
	Layout         string
	Theme          string
	Look           string
	Direction      string
	UseFrontmatter bool
}

// DefaultConfig returns the default Mermaid configuration.
func DefaultConfig() Config {
	return Config{
		Layout:    "elk",
		Theme:     "default",
		Direction: "LR",
	}
}

// Exporter handles Mermaid diagram generation.
type Exporter struct {
	Config Config
}

// NewExporter creates a new Mermaid exporter.
func NewExporter(config Config) *Exporter {
	return &Exporter{Config: config}
}

// Export generates a Mermaid diagram from a program.
func (e *Exporter) Export(prog *language.Program) string {
	if prog == nil || prog.Model == nil {
		return ""
	}

	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	e.writeHeader(sb)
	e.writeStyles(sb)

	// Extract elements from Model
	systems := extractSystemsFromModel(prog)
	persons := extractPersonsFromModel(prog)
	containers := extractTopLevelContainers(prog)
	relations := extractRelationsFromModel(prog)

	// Index things for easier lookup
	indexedArch := indexProgram(prog)

	// Render persons
	for _, p := range persons {
		e.writePerson(sb, p)
	}

	// Render systems
	for _, sys := range systems {
		e.writeSystem(sb, sys, indexedArch)
	}

	// Render standalone containers (not in a system)
	for _, cont := range containers {
		e.writeContainer(sb, cont, "", "    ")
	}

	// Render relations
	for _, rel := range relations {
		e.writeRelation(sb, rel, indexedArch)
	}

	return sb.String()
}

// Package compiler provides compilation from model to various diagram formats.
//
// The compiler package transforms architecture models into diagram formats:
//   - Mermaid C4: Primary format (text-based, Git-friendly)
//   - D2: Beautiful themes and animations (future)
//   - PlantUML: UML-style diagrams (future)
//
// The compilation pipeline:
//
//	DSL → AST → Model → Diagram Format
//
// Example usage:
//
//	parser, _ := language.NewParser()
//	program, _ := parser.Parse("example.sruja", dslText)
//	compiler := compiler.NewMermaidCompiler()
//	mermaid, err := compiler.Compile(program)
package compiler

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/model"
)

// MermaidCompiler compiles a program to Mermaid C4 format.
//
// Mermaid C4 is a text-based diagram format that can be rendered by:
//   - Mermaid.js (in browsers, VS Code, GitHub, etc.)
//   - Mermaid CLI (for generating images)
//
// The compiler transforms the architecture model into Mermaid C4 syntax,
// which supports the C4 model hierarchy (Person, System, Container, Component).
type MermaidCompiler struct {
}

// NewMermaidCompiler creates a new Mermaid compiler.
func NewMermaidCompiler() *MermaidCompiler {
	return &MermaidCompiler{}
}

// Name returns the compiler name.
//
// This is used to identify the compiler when multiple formats are supported.
func (c *MermaidCompiler) Name() string {
	return "mermaid"
}

// Compile compiles a program to Mermaid C4 diagram syntax.
//
// This is the main compilation function. It:
//  1. Transforms AST to Model (using Transformer)
//  2. Compiles Model to Mermaid syntax
//
// Parameters:
//   - program: The parsed AST program
//
// Returns:
//   - string: Mermaid C4 diagram syntax
//   - error: Compilation error if any
//
// Example:
//
//	parser, _ := language.NewParser()
//	program, _ := parser.Parse("example.sruja", dslText)
//	compiler := compiler.NewMermaidCompiler()
//	mermaid, err := compiler.Compile(program)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Println(mermaid)
func (c *MermaidCompiler) Compile(program *language.Program) (string, error) {
	// Step 1: Transform AST to Model
	transformer := NewTransformer()
	m, err := transformer.Transform(program)
	if err != nil {
		return "", fmt.Errorf("transformation error: %w", err)
	}

	// Step 2: Compile Model to Mermaid
	// Mermaid temporarily disabled as default; use D2 by default in CLI/MCP.
	return c.compileModel(m)
}

// CompileFromModel compiles a model directly to Mermaid C4 diagram syntax.
//
// This allows compilation from the IR model without needing the AST.
// Useful for kernel-based diagram generation from the architecture store.
//
// Parameters:
//   - m: The architecture model (IR)
//
// Returns:
//   - string: Mermaid C4 diagram syntax
//   - error: Compilation error if any
func (c *MermaidCompiler) CompileFromModel(m *model.Model) (string, error) {
	return c.compileModel(m)
}

// compileModel compiles a model to Mermaid C4 diagram syntax.
//
// This generates Mermaid C4 syntax from the model. The output can be:
//   - Rendered in browsers (using Mermaid.js)
//   - Rendered in VS Code (using Mermaid extension)
//   - Rendered in GitHub (automatic rendering in markdown)
//   - Converted to images (using Mermaid CLI)
//
// If the model contains journeys, it will generate multiple diagrams:
//   - C4Context diagram for architecture elements
//   - Journey diagrams for user journeys
//   - Sequence diagrams for journey steps
//
// Parameters:
//   - m: The architecture model
//
// Returns:
//   - string: Mermaid diagram syntax (may contain multiple diagrams)
//   - error: Compilation error if any
func (c *MermaidCompiler) compileModel(m *model.Model) (string, error) {
	var b strings.Builder

	// Write C4Context header
	// This tells Mermaid to use C4 diagram syntax
	b.WriteString("C4Context\n")

	// Write optional title
	if m.Architecture != nil && len(m.Architecture.Elements) > 0 {
		b.WriteString("    title Architecture Diagram\n")
	}

	// Group elements for stable, readable ordering
	persons := []model.Element{}
	systems := []model.Element{}
	containers := []model.Element{}
	components := []model.Element{}
	datastores := []model.Element{}
	queues := []model.Element{}
	externals := []model.Element{}

	for _, elem := range m.Architecture.Elements {
		switch elem.Type {
		case model.ElementTypePerson:
			persons = append(persons, elem)
		case model.ElementTypeSystem:
			systems = append(systems, elem)
		case model.ElementTypeContainer:
			containers = append(containers, elem)
		case model.ElementTypeComponent:
			components = append(components, elem)
		case model.ElementTypeDataStore:
			datastores = append(datastores, elem)
		case model.ElementTypeQueue:
			queues = append(queues, elem)
		case model.ElementTypeExternalService:
			externals = append(externals, elem)
		}
	}

	// Write in canonical order
	for _, e := range persons {
		c.writeElement(&b, e)
	}
	for _, e := range systems {
		c.writeElement(&b, e)
	}
	for _, e := range containers {
		c.writeElement(&b, e)
	}
	for _, e := range components {
		c.writeElement(&b, e)
	}
	for _, e := range datastores {
		c.writeElement(&b, e)
	}
	for _, e := range queues {
		c.writeElement(&b, e)
	}
	for _, e := range externals {
		c.writeElement(&b, e)
	}

	// Write all relations
	// Relations connect elements together
	for _, rel := range m.Architecture.Relations {
		c.writeRelation(&b, rel)
	}

	// If there are journeys, add them as separate diagrams
	if m.Architecture != nil && len(m.Architecture.Journeys) > 0 {
		b.WriteString("\n")
		for _, journey := range m.Architecture.Journeys {
			// Generate journey diagram
			c.writeJourneyDiagram(&b, journey)
			b.WriteString("\n")
			// Generate sequence diagram
			c.writeSequenceDiagram(&b, journey)
			b.WriteString("\n")
		}
	}

	return b.String(), nil
}

// writeElement writes a Mermaid element declaration.
//
// Each element type maps to a different Mermaid C4 function:
//   - Person → Person()
//   - System → System()
//   - Container → Container()
//   - Component → Component()
//   - DataStore → SystemDb()
//   - Queue → SystemQueue()
//   - ExternalService → SystemExt()
//
// Parameters:
//   - b: String builder to write to
//   - elem: The element to write
func (c *MermaidCompiler) writeElement(b *strings.Builder, elem model.Element) {
	switch elem.Type {
	case model.ElementTypePerson:
		b.WriteString(fmt.Sprintf("    Person(%s, %q)\n", elem.ID, elem.Name))
	case model.ElementTypeSystem:
		b.WriteString(fmt.Sprintf("    System(%s, %q", elem.ID, elem.Name))
		if elem.Technology != "" {
			b.WriteString(fmt.Sprintf(", %q", elem.Technology))
		}
		b.WriteString(")\n")
	case model.ElementTypeContainer:
		b.WriteString(fmt.Sprintf("    Container(%s, %q", elem.ID, elem.Name))
		if elem.Technology != "" {
			b.WriteString(fmt.Sprintf(", %q", elem.Technology))
		}
		b.WriteString(")\n")
	case model.ElementTypeComponent:
		b.WriteString(fmt.Sprintf("    Component(%s, %q", elem.ID, elem.Name))
		if elem.Technology != "" {
			b.WriteString(fmt.Sprintf(", %q", elem.Technology))
		}
		b.WriteString(")\n")
	case model.ElementTypeDataStore:
		b.WriteString(fmt.Sprintf("    SystemDb(%s, %q", elem.ID, elem.Name))
		if elem.Technology != "" {
			b.WriteString(fmt.Sprintf(", %q", elem.Technology))
		}
		b.WriteString(")\n")
	case model.ElementTypeQueue:
		b.WriteString(fmt.Sprintf("    SystemQueue(%s, %q", elem.ID, elem.Name))
		if elem.Technology != "" {
			b.WriteString(fmt.Sprintf(", %q", elem.Technology))
		}
		b.WriteString(")\n")
	case model.ElementTypeExternalService:
		b.WriteString(fmt.Sprintf("    SystemExt(%s, %q", elem.ID, elem.Name))
		if elem.Technology != "" {
			b.WriteString(fmt.Sprintf(", %q", elem.Technology))
		}
		b.WriteString(")\n")
	}
}

// writeRelation writes a Mermaid relation declaration.
//
// Relations in Mermaid C4 use the Rel() function:
//
//	Rel(from, to, label)
//
// The label is taken from the relation description, or falls back to the relation type.
//
// Parameters:
//   - b: String builder to write to
//   - rel: The relation to write
//
// Example output:
//
//	Rel(User, WebApp, "Uses")
//	Rel(WebApp, Database, "Reads/Writes")
func (c *MermaidCompiler) writeRelation(b *strings.Builder, rel model.Relation) {
	// Use description as label, or fall back to relation type
	label := rel.Description
	if label == "" {
		label = string(rel.Type)
	}
	b.WriteString(fmt.Sprintf("    Rel(%s, %s, %q)\n", rel.From, rel.To, label))
}

// writeJourneyDiagram writes a Mermaid journey diagram.
//
// Journey diagrams show user journeys with stages and tasks.
// Each step in the journey becomes a task in a stage.
//
// Parameters:
//   - b: String builder to write to
//   - journey: The journey to write
//
// Example output:
//
//	journey
//	    title User Login Journey
//	    section Enter Credentials
//	        Enter username/password: 5: User
//	    section Authenticate
//	        POST /login: 3: API
//	        Check credentials: 3: Auth
func (c *MermaidCompiler) writeJourneyDiagram(b *strings.Builder, journey model.Journey) {
	b.WriteString("journey\n")

	// Write title
	title := journey.Title
	if title == "" {
		title = journey.ID
	}
	b.WriteString(fmt.Sprintf("    title %s\n", title))

	// Group steps by "from" element to create sections
	// Each section represents a stage in the journey
	sectionMap := make(map[string][]model.Relation)
	for _, step := range journey.Steps {
		sectionMap[step.From] = append(sectionMap[step.From], step)
	}

	// Write sections
	sectionNum := 1
	for from, steps := range sectionMap {
		sectionName := from
		// Try to find element name for better readability
		b.WriteString(fmt.Sprintf("    section %s\n", sectionName))
		for _, step := range steps {
			label := step.Description
			if label == "" {
				label = string(step.Type)
			}
			// Journey syntax: task: score: participant
			// We use a default score of 5 and the "to" element as participant
			b.WriteString(fmt.Sprintf("        %s: 5: %s\n", label, step.To))
		}
		sectionNum++
	}
}

// writeSequenceDiagram writes a Mermaid sequence diagram.
//
// Sequence diagrams show the interaction between participants over time.
// Each step in the journey becomes a message in the sequence.
//
// Parameters:
//   - b: String builder to write to
//   - journey: The journey to write
//
// Example output:
//
//	sequenceDiagram
//	    participant User
//	    participant UI
//	    participant API
//	    User->>UI: enters username/password
//	    UI->>API: POST /login
//	    API->>Auth: check credentials
func (c *MermaidCompiler) writeSequenceDiagram(b *strings.Builder, journey model.Journey) {
	b.WriteString("sequenceDiagram\n")

	// Collect all unique participants from journey steps
	participants := make(map[string]bool)
	for _, step := range journey.Steps {
		participants[step.From] = true
		participants[step.To] = true
	}

	// Write participants in order they appear in steps
	participantOrder := []string{}
	seen := make(map[string]bool)
	for _, step := range journey.Steps {
		if !seen[step.From] {
			participantOrder = append(participantOrder, step.From)
			seen[step.From] = true
		}
		if !seen[step.To] {
			participantOrder = append(participantOrder, step.To)
			seen[step.To] = true
		}
	}

	// Write participant declarations
	for _, p := range participantOrder {
		b.WriteString(fmt.Sprintf("    participant %s\n", p))
	}

	// Write messages (steps)
	for _, step := range journey.Steps {
		label := step.Description
		if label == "" {
			label = string(step.Type)
		}
		// Use -> for synchronous messages
		b.WriteString(fmt.Sprintf("    %s->>%s: %s\n", step.From, step.To, label))
	}
}

// Package compiler provides compilation from model to various diagram formats.
package compiler

import (
	"fmt"
	"sort"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/model"
)

// D2Compiler compiles a program to D2 format.
//
// D2 is a modern declarative diagramming language that supports:
//   - Beautiful themes and styling
//   - Animations and sketch mode
//   - Containers and nested structures
//   - Custom layouts and positioning
//   - SVG, PNG, PDF export
//
// The compiler transforms the architecture model into D2 syntax,
// preserving C4 model hierarchy through containers.
type D2Compiler struct {
	Theme    string // D2 theme (e.g., "default", "dark", "neon")
	Sketch   bool   // Enable sketch mode (hand-drawn aesthetic)
	Layout   string // Layout engine (e.g., "dagre", "elk")
	Animated bool   // Enable animations
}

// D2Options configures D2 compiler behavior.
type D2Options struct {
	Theme    string
	Sketch   bool
	Layout   string
	Animated bool
}

// NewD2Compiler creates a new D2 compiler with default options.
func NewD2Compiler() *D2Compiler {
	return &D2Compiler{
		Theme:    "default",
		Sketch:   false,
		Layout:   "dagre",
		Animated: false,
	}
}

// NewD2CompilerWithOptions creates a new D2 compiler with custom options.
func NewD2CompilerWithOptions(opts D2Options) *D2Compiler {
	return &D2Compiler{
		Theme:    opts.Theme,
		Sketch:   opts.Sketch,
		Layout:   opts.Layout,
		Animated: opts.Animated,
	}
}

func (c *D2Compiler) Name() string { return "d2" }

func (c *D2Compiler) Compile(program *language.Program) (string, error) {
	transformer := NewTransformer()
	m, err := transformer.Transform(program)
	if err != nil {
		return "", fmt.Errorf("transformation error: %w", err)
	}
	return c.compileModel(m)
}

// CompileFromModel compiles a model directly to D2 diagram syntax.
//
// This allows compilation from the IR model without needing the AST.
// Useful for kernel-based diagram generation from the architecture store.
//
// Parameters:
//   - m: The architecture model (IR)
//
// Returns:
//   - string: D2 diagram syntax
//   - error: Compilation error if any
func (c *D2Compiler) CompileFromModel(m *model.Model) (string, error) {
	return c.compileModel(m)
}

func (c *D2Compiler) compileModel(m *model.Model) (string, error) {
	var b strings.Builder

	// Apply theme if specified
	if c.Theme != "" && c.Theme != "default" {
		fmt.Fprintf(&b, "theme: %s\n", c.Theme)
	}

	// Enable sketch mode if requested
	if c.Sketch {
		b.WriteString("sketch: true\n")
	}

	// Set layout engine
	if c.Layout != "" && c.Layout != "dagre" {
		fmt.Fprintf(&b, "layout: %s\n", c.Layout)
	}

	// Enable animations if requested
	if c.Animated {
		b.WriteString("animated: true\n")
	}

	b.WriteString("\n")

	// Group elements by type
	people := []model.Element{}
	systems := []model.Element{}
	containers := []model.Element{}
	components := []model.Element{}
	datastores := []model.Element{}
	queues := []model.Element{}
	externals := []model.Element{}

	for _, e := range m.Architecture.Elements {
		switch e.Type {
		case model.ElementTypePerson:
			people = append(people, e)
		case model.ElementTypeSystem:
			systems = append(systems, e)
		case model.ElementTypeContainer:
			containers = append(containers, e)
		case model.ElementTypeComponent:
			components = append(components, e)
		case model.ElementTypeDataStore:
			datastores = append(datastores, e)
		case model.ElementTypeQueue:
			queues = append(queues, e)
		case model.ElementTypeExternalService:
			externals = append(externals, e)
		}
	}

	// Stable ordering
	sorter := func(xs []model.Element) { sort.Slice(xs, func(i, j int) bool { return xs[i].ID < xs[j].ID }) }
	sorter(people)
	sorter(systems)
	sorter(containers)
	sorter(components)
	sorter(datastores)
	sorter(queues)
	sorter(externals)

	// Build hierarchy: Systems contain Containers, Containers contain Components
	// Use D2 containers to represent this hierarchy
	systemMap := make(map[string][]model.Element)
	containerMap := make(map[string][]model.Element)

	// Group containers by system (if we can infer from relations)
	// For now, we'll use a flat structure but with proper D2 container syntax
	for _, e := range uniqueByID(systems) {
		systemMap[e.ID] = []model.Element{}
	}

	// Group components by container
	for _, e := range uniqueByID(containers) {
		containerMap[e.ID] = []model.Element{}
	}

	// Write people (top level)
	for _, e := range uniqueByID(people) {
		c.writeElement(&b, e, 0)
	}

	// Write systems with containers inside
	for _, sys := range uniqueByID(systems) {
		c.writeSystemWithContainers(&b, sys, containers, components, datastores, queues, 0)
	}

	// Write standalone containers (not in systems)
	for _, cont := range uniqueByID(containers) {
		// Check if this container is already written inside a system
		written := false
		for _, sys := range uniqueByID(systems) {
			// Simple check: if container ID is referenced in system relations
			for _, rel := range m.Architecture.Relations {
				if (rel.From == sys.ID && rel.To == cont.ID) || (rel.To == sys.ID && rel.From == cont.ID) {
					written = true
					break
				}
			}
			if written {
				break
			}
		}
		if !written {
			c.writeContainerWithComponents(&b, cont, components, 0)
		}
	}

	// Write standalone components (not in containers)
	for _, comp := range uniqueByID(components) {
		written := false
		for _, cont := range uniqueByID(containers) {
			for _, rel := range m.Architecture.Relations {
				if (rel.From == cont.ID && rel.To == comp.ID) || (rel.To == cont.ID && rel.From == comp.ID) {
					written = true
					break
				}
			}
			if written {
				break
			}
		}
		if !written {
			c.writeElement(&b, comp, 0)
		}
	}

	// Write datastores
	for _, e := range uniqueByID(datastores) {
		c.writeElement(&b, e, 0)
	}

	// Write queues
	for _, e := range uniqueByID(queues) {
		c.writeElement(&b, e, 0)
	}

	// Write externals
	for _, e := range uniqueByID(externals) {
		c.writeElement(&b, e, 0)
	}

	// Write edges
	b.WriteString("\n")
	for _, r := range m.Architecture.Relations {
		c.writeRelation(&b, r)
	}

	return b.String(), nil
}

// writeElement writes a D2 element with proper styling based on type.
func (c *D2Compiler) writeElement(b *strings.Builder, e model.Element, indent int) {
	indentStr := strings.Repeat("  ", indent)
	label := e.Name
	if e.Technology != "" {
		label = fmt.Sprintf("%s\\n%s", e.Name, e.Technology)
	}

	// D2 syntax: id: "Label" { ... }
	// Add styling based on element type
	fmt.Fprintf(b, "%s%s: \"%s\"", indentStr, e.ID, label)

	// Add type-specific styling
	style := c.getElementStyle(e.Type)
	if style != "" {
		fmt.Fprintf(b, " {\n%s  %s\n%s}", indentStr, style, indentStr)
	}
	b.WriteString("\n")
}

// writeSystemWithContainers writes a system with its containers nested inside.
func (c *D2Compiler) writeSystemWithContainers(b *strings.Builder, sys model.Element, containers, components, datastores, queues []model.Element, indent int) {
	indentStr := strings.Repeat("  ", indent)
	label := sys.Name
	if sys.Technology != "" {
		label = fmt.Sprintf("%s\\n%s", sys.Name, sys.Technology)
	}

	fmt.Fprintf(b, "%s%s: \"%s\" {\n", indentStr, sys.ID, label)

	// Find containers that belong to this system (simplified: by naming or relations)
	// For now, write all containers at this level
	for _, cont := range containers {
		c.writeContainerWithComponents(b, cont, components, indent+1)
	}

	fmt.Fprintf(b, "%s}\n", indentStr)
}

// writeContainerWithComponents writes a container with its components nested inside.
func (c *D2Compiler) writeContainerWithComponents(b *strings.Builder, cont model.Element, components []model.Element, indent int) {
	indentStr := strings.Repeat("  ", indent)
	label := cont.Name
	if cont.Technology != "" {
		label = fmt.Sprintf("%s\\n%s", cont.Name, cont.Technology)
	}

	fmt.Fprintf(b, "%s%s: \"%s\" {\n", indentStr, cont.ID, label)

	// Find components that belong to this container
	for _, comp := range components {
		c.writeElement(b, comp, indent+1)
	}

	fmt.Fprintf(b, "%s}\n", indentStr)
}

// writeRelation writes a D2 relation with proper styling.
func (c *D2Compiler) writeRelation(b *strings.Builder, r model.Relation) {
	label := r.Description
	if label == "" {
		label = string(r.Type)
	}

	// D2 supports different arrow styles based on relation type
	arrow := "->"
	switch r.Type {
	case model.RelationTypePublishes:
		arrow = "->"
	case model.RelationTypeSubscribes:
		arrow = "<-"
	case model.RelationTypeDepends:
		arrow = "->"
	default:
		arrow = "->"
	}

	fmt.Fprintf(b, "%s %s %s: \"%s\"\n", r.From, arrow, r.To, label)
}

// getElementStyle returns D2 styling for element types.
func (c *D2Compiler) getElementStyle(eType model.ElementType) string {
	switch eType {
	case model.ElementTypePerson:
		return "shape: person"
	case model.ElementTypeSystem:
		return "shape: rectangle\n    style.fill: \"#4A90E2\"\n    style.stroke: \"#2E5C8A\""
	case model.ElementTypeContainer:
		return "shape: rectangle\n    style.fill: \"#7ED321\"\n    style.stroke: \"#5BA317\""
	case model.ElementTypeComponent:
		return "shape: rectangle\n    style.fill: \"#F5A623\"\n    style.stroke: \"#D68910\""
	case model.ElementTypeDataStore:
		return "shape: cylinder\n    style.fill: \"#BD10E0\"\n    style.stroke: \"#9012FE\""
	case model.ElementTypeQueue:
		return "shape: queue\n    style.fill: \"#50E3C2\"\n    style.stroke: \"#2ECC71\""
	case model.ElementTypeExternalService:
		return "shape: hexagon\n    style.fill: \"#B8E986\"\n    style.stroke: \"#7ED321\""
	default:
		return ""
	}
}

func uniqueByID(xs []model.Element) []model.Element {
	m := make(map[string]model.Element)
	for _, e := range xs {
		m[e.ID] = e
	}
	out := make([]model.Element, 0, len(m))
	for _, e := range m {
		out = append(out, e)
	}
	sort.Slice(out, func(i, j int) bool { return out[i].ID < out[j].ID })
	return out
}

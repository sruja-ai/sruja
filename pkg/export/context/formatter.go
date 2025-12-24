package context

import (
	"fmt"
	"strings"
)

// Element represents a simplified architectural element for context generation.
type Element struct {
	Name        string
	Kind        string
	Description string
	Metadata    map[string]string
}

// Relation represents a simplified relationship.
type Relation struct {
	Source string
	Target string
	Label  string
}

// Formatter handles the Markdown generation for AI context.
type Formatter struct {
	sb strings.Builder
}

// NewFormatter creates a new formatter.
func NewFormatter() *Formatter {
	return &Formatter{}
}

// String returns the formatted context.
func (f *Formatter) String() string {
	return f.sb.String()
}

// WriteHeader writes the main document header.
func (f *Formatter) WriteHeader(title string, instruction string) {
	f.sb.WriteString(fmt.Sprintf("# Context: %s\n\n", title))
	if instruction != "" {
		f.sb.WriteString("<!-- INSTRUCTION START -->\n")
		f.sb.WriteString(instruction)
		f.sb.WriteString("\n<!-- INSTRUCTION END -->\n\n")
	}
}

// WriteElement writes a simplified element representation.
func (f *Formatter) WriteElement(el Element, depth int) {
	indent := strings.Repeat("  ", depth)

	// Basic Info: Name (Kind)
	f.sb.WriteString(fmt.Sprintf("%s- **%s** (%s)", indent, el.Name, el.Kind))

	// Description
	if el.Description != "" {
		f.sb.WriteString(fmt.Sprintf(": %s", el.Description))
	}
	f.sb.WriteString("\n")

	// Metadata props text
	if len(el.Metadata) > 0 {
		var props []string
		for k, v := range el.Metadata {
			props = append(props, fmt.Sprintf("%s: %s", k, v))
		}
		f.sb.WriteString(fmt.Sprintf("%s  *Metadata*: { %s }\n", indent, strings.Join(props, ", ")))
	}
}

// WriteRelationship writes a readable prose relationship.
func (f *Formatter) WriteRelationship(rel Relation) {
	label := rel.Label
	if label == "" {
		label = "connects to"
	}

	f.sb.WriteString(fmt.Sprintf("- [%s] -> [%s]: %s\n",
		rel.Source, rel.Target, label))
}

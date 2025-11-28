// pkg/language/printer.go
// Package language provides DSL parsing and AST structures.
package language

import (
	"fmt"
	"strings"
)

// Printer prints an AST back to DSL format.
type Printer struct {
	IndentLevel int
}

// NewPrinter creates a new printer.
func NewPrinter() *Printer {
	return &Printer{IndentLevel: 0}
}

// indent returns the indentation string for the current indent level.
func (p *Printer) indent() string {
	return strings.Repeat("  ", p.IndentLevel)
}

// Print prints a Program back to DSL format.
func (p *Printer) Print(program *Program) string {
	if program == nil || program.Architecture == nil {
		return ""
	}

	var sb strings.Builder
	p.printArchitecture(&sb, program.Architecture)
	return sb.String()
}

// printArchitecture prints an architecture node.
func (p *Printer) printArchitecture(sb *strings.Builder, arch *Architecture) {
	indent := p.indent()
	sb.WriteString(indent + "architecture \"" + arch.Name + "\" {\n")
	p.IndentLevel++

	// Print imports
	for _, imp := range arch.Imports {
		p.printImport(sb, imp)
	}

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

	// Print journeys
	for _, journey := range arch.Journeys {
		p.printJourney(sb, journey)
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
	sb.WriteString(indent + "}\n")
}

// printImport prints an import statement.
func (p *Printer) printImport(sb *strings.Builder, imp *ImportSpec) {
	indent := p.indent()
	fmt.Fprintf(sb, "%simport %q", indent, imp.Path)
	if imp.Alias != nil {
		fmt.Fprintf(sb, " as %s", *imp.Alias)
	}
	sb.WriteString("\n")
}

// printSystem prints a system node.
func (p *Printer) printSystem(sb *strings.Builder, sys *System) {
	indent := p.indent()
	fmt.Fprintf(sb, "%ssystem %s %q", indent, sys.ID, sys.Label)

	if sys.Description != nil && *sys.Description != "" {
		fmt.Fprintf(sb, " %q", *sys.Description)
	}

	if len(sys.Items) > 0 || len(sys.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++

		for _, item := range sys.Items {
			if item.Container != nil {
				p.printContainer(sb, item.Container)
			}
			if item.DataStore != nil {
				p.printDataStore(sb, item.DataStore)
			}
			if item.Queue != nil {
				p.printQueue(sb, item.Queue)
			}
			if item.Person != nil {
				p.printPerson(sb, item.Person)
			}
			if item.Relation != nil {
				p.printRelation(sb, item.Relation)
			}
			if item.Requirement != nil {
				p.printRequirement(sb, item.Requirement)
			}
			if item.ADR != nil {
				p.printADR(sb, item.ADR)
			}
			if item.Metadata != nil {
				p.printMetadata(sb, item.Metadata.Entries)
			}
		}

		if len(sys.Metadata) > 0 {
			p.printMetadata(sb, sys.Metadata)
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent + "}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printContainer prints a container node.
func (p *Printer) printContainer(sb *strings.Builder, cont *Container) {
	indent := p.indent()
	fmt.Fprintf(sb, "%scontainer %s %q", indent, cont.ID, cont.Label)

	if cont.Description != nil && *cont.Description != "" {
		fmt.Fprintf(sb, " %q", *cont.Description)
	}

	if len(cont.Items) > 0 || len(cont.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++

		for _, item := range cont.Items {
			if item.Technology != nil {
				indent = p.indent()
				fmt.Fprintf(sb, "%stechnology %q\n", indent, *item.Technology)
			}
			if len(item.Tags) > 0 {
				indent = p.indent()
				fmt.Fprintf(sb, "%stags [", indent)
				for i, tag := range item.Tags {
					if i > 0 {
						sb.WriteString(", ")
					}
					fmt.Fprintf(sb, "%q", tag)
				}
				sb.WriteString("]\n")
			}
			if item.Version != nil {
				indent = p.indent()
				fmt.Fprintf(sb, "%sversion %q\n", indent, *item.Version)
			}
			if item.Component != nil {
				p.printComponent(sb, item.Component)
			}
			if item.DataStore != nil {
				p.printDataStore(sb, item.DataStore)
			}
			if item.Queue != nil {
				p.printQueue(sb, item.Queue)
			}
			if item.Relation != nil {
				p.printRelation(sb, item.Relation)
			}
			if item.Requirement != nil {
				p.printRequirement(sb, item.Requirement)
			}
			if item.ADR != nil {
				p.printADR(sb, item.ADR)
			}
			if item.Metadata != nil {
				p.printMetadata(sb, item.Metadata.Entries)
			}
		}

		if len(cont.Metadata) > 0 {
			p.printMetadata(sb, cont.Metadata)
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent + "}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printComponent prints a component node.
func (p *Printer) printComponent(sb *strings.Builder, comp *Component) {
	indent := p.indent()
	fmt.Fprintf(sb, "%scomponent %s %q", indent, comp.ID, comp.Label)

	if comp.Description != nil && *comp.Description != "" {
		fmt.Fprintf(sb, " %q", *comp.Description)
	}

	if comp.Technology != nil || len(comp.Items) > 0 || len(comp.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++

		if comp.Technology != nil {
			indent = p.indent()
			fmt.Fprintf(sb, "%stechnology %q\n", indent, *comp.Technology)
		}

		for _, item := range comp.Items {
			if item.Requirement != nil {
				p.printRequirement(sb, item.Requirement)
			}
			if item.ADR != nil {
				p.printADR(sb, item.ADR)
			}
			if item.Relation != nil {
				p.printRelation(sb, item.Relation)
			}
			if item.Metadata != nil {
				p.printMetadata(sb, item.Metadata.Entries)
			}
		}

		if len(comp.Metadata) > 0 {
			p.printMetadata(sb, comp.Metadata)
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent + "}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printDataStore prints a datastore node.
func (p *Printer) printDataStore(sb *strings.Builder, ds *DataStore) {
	indent := p.indent()
	fmt.Fprintf(sb, "%sdatastore %s %q", indent, ds.ID, ds.Label)
	if ds.Description != nil && *ds.Description != "" {
		fmt.Fprintf(sb, " %q", *ds.Description)
	}
	if len(ds.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++
		p.printMetadata(sb, ds.Metadata)
		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent + "}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printQueue prints a queue node.
func (p *Printer) printQueue(sb *strings.Builder, q *Queue) {
	indent := p.indent()
	fmt.Fprintf(sb, "%squeue %s %q", indent, q.ID, q.Label)
	if q.Description != nil && *q.Description != "" {
		fmt.Fprintf(sb, " %q", *q.Description)
	}
	if len(q.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++
		p.printMetadata(sb, q.Metadata)
		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent + "}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printPerson prints a person node.
func (p *Printer) printPerson(sb *strings.Builder, person *Person) {
	indent := p.indent()
	fmt.Fprintf(sb, "%sperson %s %q", indent, person.ID, person.Label)
	if len(person.Metadata) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++
		p.printMetadata(sb, person.Metadata)
		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent + "}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printRelation prints a relation node.
func (p *Printer) printRelation(sb *strings.Builder, rel *Relation) {
	indent := p.indent()
	fmt.Fprintf(sb, "%s%s -> %s", indent, rel.From, rel.To)
	if rel.Verb != nil {
		fmt.Fprintf(sb, " %s", *rel.Verb)
	}
	if rel.Label != nil && *rel.Label != "" {
		fmt.Fprintf(sb, " %q", *rel.Label)
	}
	sb.WriteString("\n")
}

// printRequirement prints a requirement node.
func (p *Printer) printRequirement(sb *strings.Builder, req *Requirement) {
	indent := p.indent()
	fmt.Fprintf(sb, "%srequirement %s %s %q\n", indent, req.ID, req.Type, req.Description)
}

// printADR prints an ADR node.
func (p *Printer) printADR(sb *strings.Builder, adr *ADR) {
	indent := p.indent()
	fmt.Fprintf(sb, "%sadr %s %q", indent, adr.ID, adr.Title)

	if adr.Body != nil {
		sb.WriteString(" {\n")
		p.IndentLevel++
		indent = p.indent()

		if adr.Body.Status != nil {
			fmt.Fprintf(sb, "%sstatus %q\n", indent, *adr.Body.Status)
		}
		if adr.Body.Context != nil {
			fmt.Fprintf(sb, "%scontext %q\n", indent, *adr.Body.Context)
		}
		if adr.Body.Decision != nil {
			fmt.Fprintf(sb, "%sdecision %q\n", indent, *adr.Body.Decision)
		}
		if adr.Body.Consequences != nil {
			fmt.Fprintf(sb, "%sconsequences %q\n", indent, *adr.Body.Consequences)
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent + "}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printJourney prints a journey node.
func (p *Printer) printJourney(sb *strings.Builder, journey *Journey) {
	indent := p.indent()
	fmt.Fprintf(sb, "%sjourney %s {\n", indent, journey.ID)
	p.IndentLevel++

	if journey.Title != "" {
		indent = p.indent()
		fmt.Fprintf(sb, "%stitle %q\n", indent, journey.Title)
	}

	if len(journey.Steps) > 0 {
		indent = p.indent()
		sb.WriteString(indent + "steps {\n")
		p.IndentLevel++
		for _, step := range journey.Steps {
			p.printJourneyStep(sb, step)
		}
		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent + "}\n")
	}

	p.IndentLevel--
	indent = p.indent()
	sb.WriteString(indent + "}\n")
}

// printJourneyStep prints a journey step node.
func (p *Printer) printJourneyStep(sb *strings.Builder, step *JourneyStep) {
	indent := p.indent()
	fmt.Fprintf(sb, "%s%s %s %s", indent, step.From, step.Arrow, step.To)
	if step.Label != nil && *step.Label != "" {
		fmt.Fprintf(sb, " %q", *step.Label)
	}
	sb.WriteString("\n")
}

// printSharedArtifact prints a shared artifact node.
func (p *Printer) printSharedArtifact(sb *strings.Builder, sa *SharedArtifact) {
	indent := p.indent()
	fmt.Fprintf(sb, "%ssharedArtifact %s %q", indent, sa.ID, sa.Label)
	if sa.Version != nil {
		fmt.Fprintf(sb, " version %q", *sa.Version)
	}
	if sa.Owner != nil {
		fmt.Fprintf(sb, " owner %q", *sa.Owner)
	}
	sb.WriteString("\n")
}

// printLibrary prints a library node.
func (p *Printer) printLibrary(sb *strings.Builder, lib *Library) {
	indent := p.indent()
	fmt.Fprintf(sb, "%slibrary %s %q", indent, lib.ID, lib.Label)
	if lib.Version != nil {
		fmt.Fprintf(sb, " version %q", *lib.Version)
	}
	if lib.Owner != nil {
		fmt.Fprintf(sb, " owner %q", *lib.Owner)
	}
	sb.WriteString("\n")
}

// printMetadata prints a metadata block.
func (p *Printer) printMetadata(sb *strings.Builder, entries []*MetaEntry) {
	if len(entries) == 0 {
		return
	}
	indent := p.indent()
	sb.WriteString(indent + "metadata {\n")
	p.IndentLevel++
	for _, entry := range entries {
		indent = p.indent()
		fmt.Fprintf(sb, "%s%s: %q\n", indent, entry.Key, entry.Value)
	}
	p.IndentLevel--
	indent = p.indent()
	sb.WriteString(indent + "}\n")
}

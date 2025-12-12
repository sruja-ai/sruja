// pkg/language/printer_change_snapshot.go
// Package language provides DSL parsing and AST structures.
package language

import (
	"fmt"
	"strings"
)

// printChangeBlock prints a change block
func (p *Printer) printChangeBlock(sb *strings.Builder, change *ChangeBlock) {
	indent := p.indent()
	sb.WriteString(indent)
	sb.WriteString("change ")
	sb.WriteString(fmt.Sprintf("%q", change.ID))
	sb.WriteString(" {\n")
	p.IndentLevel++

	if change.Version != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("version ")
		sb.WriteString(fmt.Sprintf("%q", *change.Version))
		sb.WriteString("\n")
	}
	if change.Status != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("status ")
		sb.WriteString(fmt.Sprintf("%q", *change.Status))
		sb.WriteString("\n")
	}
	if change.Requirement != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("requirement ")
		sb.WriteString(fmt.Sprintf("%q", *change.Requirement))
		sb.WriteString("\n")
	}
	if change.ADR != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("adr ")
		sb.WriteString(fmt.Sprintf("%q", *change.ADR))
		sb.WriteString("\n")
	}

	if change.Metadata != nil {
		p.printMetadataBlock(sb, change.Metadata)
	}

	if change.Add != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("add {\n")
		p.IndentLevel++
		p.printArchitectureBlock(sb, change.Add)
		p.IndentLevel--
		ind = p.indent()
		sb.WriteString(ind)
		sb.WriteString("}\n")
	}

	if change.Modify != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("modify {\n")
		p.IndentLevel++
		p.printArchitectureBlock(sb, change.Modify)
		p.IndentLevel--
		ind = p.indent()
		sb.WriteString(ind)
		sb.WriteString("}\n")
	}

	if change.Remove != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("remove {\n")
		p.IndentLevel++
		p.printArchitectureBlock(sb, change.Remove)
		p.IndentLevel--
		ind = p.indent()
		sb.WriteString(ind)
		sb.WriteString("}\n")
	}

	p.IndentLevel--
	ind := p.indent()
	sb.WriteString(ind)
	sb.WriteString("}\n")
}

// printArchitectureBlock prints an architecture block (for add/modify/remove)
func (p *Printer) printArchitectureBlock(sb *strings.Builder, block *ArchitectureBlock) {
	for i := range block.Items {
		item := &block.Items[i]
		if item.System != nil {
			p.printSystem(sb, item.System)
		}
		if item.Container != nil {
			p.printContainer(sb, item.Container)
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
		// Add other item types as needed
	}
}

// printSnapshotBlock prints a snapshot block
//
//nolint:unused // Future use
func (p *Printer) printSnapshotBlock(sb *strings.Builder, snapshot *SnapshotBlock) {
	indent := p.indent()
	sb.WriteString(indent)
	sb.WriteString("snapshot ")
	sb.WriteString(fmt.Sprintf("%q", snapshot.Name))
	sb.WriteString(" {\n")
	p.IndentLevel++

	if snapshot.Version != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("version ")
		sb.WriteString(fmt.Sprintf("%q", *snapshot.Version))
		sb.WriteString("\n")
	}
	if snapshot.Description != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("description ")
		sb.WriteString(fmt.Sprintf("%q", *snapshot.Description))
		sb.WriteString("\n")
	}
	if snapshot.Timestamp != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("timestamp ")
		sb.WriteString(fmt.Sprintf("%q", *snapshot.Timestamp))
		sb.WriteString("\n")
	}
	if snapshot.Preview != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("preview ")
		if *snapshot.Preview {
			sb.WriteString("true")
		} else {
			sb.WriteString("false")
		}
		sb.WriteString("\n")
	}
	if len(snapshot.Changes) > 0 {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("changes [")
		quoted := make([]string, len(snapshot.Changes))
		for i, v := range snapshot.Changes {
			quoted[i] = fmt.Sprintf("%q", v)
		}
		sb.WriteString(strings.Join(quoted, ", "))
		sb.WriteString("]\n")
	}
	if snapshot.ArchName != nil && snapshot.Architecture != nil {
		ind := p.indent()
		sb.WriteString(ind)
		sb.WriteString("architecture ")
		sb.WriteString(fmt.Sprintf("%q", *snapshot.ArchName))
		sb.WriteString(" {\n")
		p.IndentLevel++
		p.printArchitecture(sb, snapshot.Architecture)
		p.IndentLevel--
		ind = p.indent()
		sb.WriteString(ind)
		sb.WriteString("}\n")
	}

	p.IndentLevel--
	ind := p.indent()
	sb.WriteString(ind)
	sb.WriteString("}\n")
}

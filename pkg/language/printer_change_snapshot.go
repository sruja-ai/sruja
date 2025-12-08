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
	fmt.Fprintf(sb, "%schange %q {\n", indent, change.ID)
	p.IndentLevel++

	if change.Version != nil {
		fmt.Fprintf(sb, "%sversion %q\n", p.indent(), *change.Version)
	}
	if change.Status != nil {
		fmt.Fprintf(sb, "%sstatus %q\n", p.indent(), *change.Status)
	}
	if change.Requirement != nil {
		fmt.Fprintf(sb, "%srequirement %q\n", p.indent(), *change.Requirement)
	}
	if change.ADR != nil {
		fmt.Fprintf(sb, "%sadr %q\n", p.indent(), *change.ADR)
	}

	if change.Metadata != nil {
		p.printMetadataBlock(sb, change.Metadata)
	}

	if change.Add != nil {
		sb.WriteString(p.indent() + "add {\n")
		p.IndentLevel++
		p.printArchitectureBlock(sb, change.Add)
		p.IndentLevel--
		sb.WriteString(p.indent() + "}\n")
	}

	if change.Modify != nil {
		sb.WriteString(p.indent() + "modify {\n")
		p.IndentLevel++
		p.printArchitectureBlock(sb, change.Modify)
		p.IndentLevel--
		sb.WriteString(p.indent() + "}\n")
	}

	if change.Remove != nil {
		sb.WriteString(p.indent() + "remove {\n")
		p.IndentLevel++
		p.printArchitectureBlock(sb, change.Remove)
		p.IndentLevel--
		sb.WriteString(p.indent() + "}\n")
	}

	p.IndentLevel--
	sb.WriteString(p.indent() + "}\n")
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
	fmt.Fprintf(sb, "%ssnapshot %q {\n", indent, snapshot.Name)
	p.IndentLevel++

	if snapshot.Version != nil {
		fmt.Fprintf(sb, "%sversion %q\n", p.indent(), *snapshot.Version)
	}
	if snapshot.Description != nil {
		fmt.Fprintf(sb, "%sdescription %q\n", p.indent(), *snapshot.Description)
	}
	if snapshot.Timestamp != nil {
		fmt.Fprintf(sb, "%stimestamp %q\n", p.indent(), *snapshot.Timestamp)
	}
	if snapshot.Preview != nil {
		previewStr := "false"
		if *snapshot.Preview {
			previewStr = "true"
		}
		fmt.Fprintf(sb, "%spreview %s\n", p.indent(), previewStr)
	}
	if len(snapshot.Changes) > 0 {
		quoted := make([]string, len(snapshot.Changes))
		for i, v := range snapshot.Changes {
			quoted[i] = fmt.Sprintf("%q", v)
		}
		fmt.Fprintf(sb, "%schanges [%s]\n", p.indent(), strings.Join(quoted, ", "))
	}
	if snapshot.ArchName != nil && snapshot.Architecture != nil {
		fmt.Fprintf(sb, "%sarchitecture %q {\n", p.indent(), *snapshot.ArchName)
		p.IndentLevel++
		p.printArchitecture(sb, snapshot.Architecture)
		p.IndentLevel--
		sb.WriteString(p.indent() + "}\n")
	}

	p.IndentLevel--
	sb.WriteString(p.indent() + "}\n")
}

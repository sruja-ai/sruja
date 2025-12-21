// pkg/language/printer_metadata.go
// Printer methods for metadata
package language

import (
	"fmt"
	"strings"
)

// printMetadataBlock prints a metadata block with support for arrays
func (p *Printer) printMetadataBlock(sb *strings.Builder, block *MetadataBlock) {
	if block == nil || len(block.Entries) == 0 {
		return
	}
	indent := p.indent()
	sb.WriteString(indent)
	sb.WriteString("metadata {\n")
	p.IndentLevel++
	for _, entry := range block.Entries {
		entryIndent := p.indent()
		if entry.Value != nil {
			sb.WriteString(entryIndent)
			sb.WriteString(entry.Key)
			sb.WriteString(" ")
			sb.WriteString(fmt.Sprintf("%q\n", *entry.Value))
		} else if len(entry.Array) > 0 {
			sb.WriteString(entryIndent)
			sb.WriteString(entry.Key)
			sb.WriteString(" [")
			for i, v := range entry.Array {
				if i > 0 {
					sb.WriteString(", ")
				}
				sb.WriteString(fmt.Sprintf("%q", v))
			}
			sb.WriteString("]\n")
		}
	}
	p.IndentLevel--
	indent = p.indent()
	sb.WriteString(indent)
	sb.WriteString("}\n")
}

// printMetadata prints a metadata block.
func (p *Printer) printMetadata(sb *strings.Builder, entries []*MetaEntry) {
	if len(entries) == 0 {
		return
	}
	indent := p.indent()
	sb.WriteString(indent)
	sb.WriteString("metadata {\n")
	p.IndentLevel++
	for _, entry := range entries {
		entryIndent := p.indent()
		if entry.Value != nil {
			sb.WriteString(entryIndent)
			sb.WriteString(entry.Key)
			sb.WriteString(" ")
			sb.WriteString(fmt.Sprintf("%q\n", *entry.Value))
		} else if len(entry.Array) > 0 {
			sb.WriteString(entryIndent)
			sb.WriteString(entry.Key)
			sb.WriteString(" [")
			for i, v := range entry.Array {
				if i > 0 {
					sb.WriteString(", ")
				}
				sb.WriteString(fmt.Sprintf("%q", v))
			}
			sb.WriteString("]\n")
		}
	}
	p.IndentLevel--
	indent = p.indent()
	sb.WriteString(indent)
	sb.WriteString("}\n")
}

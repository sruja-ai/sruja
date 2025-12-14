// pkg/language/printer_blocks.go
// Printer methods for blocks (Policy, Flow, Requirement, ADR, etc.)
package language

import (
	"fmt"
	"strings"
)

// printPolicy prints a policy node.
func (p *Printer) printPolicy(sb *strings.Builder, policy *Policy) {
	indent := p.indent()
	sb.WriteString(indent)
	sb.WriteString("policy ")
	sb.WriteString(policy.ID)
	sb.WriteString(" ")
	sb.WriteString(fmt.Sprintf("%q", policy.Description))

	if policy.Body != nil {
		sb.WriteString(" {\n")
		p.IndentLevel++
		indent = p.indent()

		if policy.Body.Category != nil {
			sb.WriteString(indent)
			sb.WriteString("category ")
			sb.WriteString(fmt.Sprintf("%q\n", *policy.Body.Category))
		}
		if policy.Body.Enforcement != nil {
			sb.WriteString(indent)
			sb.WriteString("enforcement ")
			sb.WriteString(fmt.Sprintf("%q\n", *policy.Body.Enforcement))
		}
		if policy.Body.Description != nil {
			sb.WriteString(indent)
			sb.WriteString("description ")
			sb.WriteString(fmt.Sprintf("%q\n", *policy.Body.Description))
		}
		if policy.Body.Metadata != nil {
			p.printMetadataBlock(sb, policy.Body.Metadata)
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		if policy.Category != nil {
			sb.WriteString(" category ")
			sb.WriteString(fmt.Sprintf("%q", *policy.Category))
		}
		if policy.Enforcement != nil {
			sb.WriteString(" enforcement ")
			sb.WriteString(fmt.Sprintf("%q", *policy.Enforcement))
		}
		sb.WriteString("\n")
	}
}

// printFlow prints a flow node.
func (p *Printer) printFlow(sb *strings.Builder, flow *Flow) {
	indent := p.indent()
	sb.WriteString(indent)
	sb.WriteString("flow ")
	sb.WriteString(flow.ID)
	sb.WriteString(" ")
	sb.WriteString(fmt.Sprintf("%q {\n", flow.Title))
	p.IndentLevel++

	if flow.Description != nil {
		stepIndent := p.indent()
		sb.WriteString(stepIndent)
		sb.WriteString("description ")
		sb.WriteString(fmt.Sprintf("%q\n", *flow.Description))
	}

	for _, item := range flow.Items {
		if item.Step != nil {
			stepIndent := p.indent()
			sb.WriteString(stepIndent)
			sb.WriteString(item.Step.From.String())
			sb.WriteString(" -> ")
			sb.WriteString(item.Step.To.String())
			if item.Step.Description != nil {
				sb.WriteString(" ")
				sb.WriteString(fmt.Sprintf("%q", *item.Step.Description))
			}
			if len(item.Step.Tags) > 0 {
				sb.WriteString(" [")
				for i, tag := range item.Step.Tags {
					if i > 0 {
						sb.WriteString(", ")
					}
					sb.WriteString(tag)
				}
				sb.WriteString("]")
			}
			if item.Step.Order != nil {
				sb.WriteString(" order ")
				sb.WriteString(fmt.Sprintf("%q", *item.Step.Order))
			}
			sb.WriteString("\n")
		}
	}

	p.IndentLevel--
	sb.WriteString(indent)
	sb.WriteString("}\n")
}

// printRequirement prints a requirement node.
func (p *Printer) printRequirement(sb *strings.Builder, req *Requirement) {
	indent := p.indent()
	fmt.Fprintf(sb, "%srequirement %s", indent, req.ID)

	if req.Type != nil {
		fmt.Fprintf(sb, " %s", *req.Type)
	}
	if req.Description != nil {
		fmt.Fprintf(sb, " %q", *req.Description)
	}

	if req.Body != nil {
		sb.WriteString(" {\n")
		p.IndentLevel++
		indent = p.indent()

		if req.Body.Type != nil {
			fmt.Fprintf(sb, "%stype %q\n", indent, *req.Body.Type)
		}
		if req.Body.Description != nil {
			fmt.Fprintf(sb, "%sdescription %q\n", indent, *req.Body.Description)
		}
		if req.Body.Metadata != nil {
			p.printMetadataBlock(sb, req.Body.Metadata)
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		sb.WriteString("\n")
	}
}

// printADR prints an ADR node.
func (p *Printer) printADR(sb *strings.Builder, adr *ADR) {
	indent := p.indent()
	if adr.Title != nil {
		fmt.Fprintf(sb, "%sadr %s %q", indent, adr.ID, *adr.Title)
	} else {
		fmt.Fprintf(sb, "%sadr %s", indent, adr.ID)
	}

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
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		sb.WriteString("\n")
	}
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

	if len(lib.Items) > 0 {
		sb.WriteString(" {\n")
		p.IndentLevel++
		indent = p.indent()

		for _, item := range lib.Items {
			if item.Description != nil {
				fmt.Fprintf(sb, "%sdescription %q\n", indent, *item.Description)
			}
			if item.Policy != nil {
				p.printPolicy(sb, item.Policy)
			}
			if item.Requirement != nil {
				p.printRequirement(sb, item.Requirement)
			}
			if item.Metadata != nil {
				p.printMetadataBlock(sb, item.Metadata)
			}
		}

		p.IndentLevel--
		indent = p.indent()
		sb.WriteString(indent)
		sb.WriteString("}\n")
	} else {
		sb.WriteString("\n")
	}
}

// pkg/export/markdown/toc_helpers.go
// Package markdown provides helper functions for TOC generation.
//
//nolint:gocritic // preferFprint false positive or style choice
package markdown

//nolint:gocritic // Use WriteString for consistency

import (
	"fmt"
	"strings"
)

// writeTOCItem conditionally writes a TOC item
func writeTOCItem(sb *strings.Builder, condition bool, text, anchor string) {
	if condition {
		sb.WriteString(fmt.Sprintf("- [%s](#%s)\n", text, anchor))
	}
}

// writeTOCItems writes common TOC items that are always present
// Order optimized for architecture review: Executive → Architecture → Requirements →
// ADRs → Quality → Security → Operations → Risk → Compliance → Cost → Technical → Reference
func writeTOCItems(sb *strings.Builder) {
	sb.WriteString("- [Quality Attributes](#quality-attributes)\n")
	sb.WriteString("- [Security](#security)\n")
	sb.WriteString("- [Capacity Planning](#capacity-planning)\n")
	sb.WriteString("- [Monitoring & Observability](#monitoring--observability)\n")
	sb.WriteString("- [Failure Modes and Recovery](#failure-modes-and-recovery)\n")
	sb.WriteString("- [Dependency Risk Assessment](#dependency-risk-assessment)\n")
	sb.WriteString("- [Compliance & Certifications](#compliance--certifications)\n")
	sb.WriteString("- [Cost Analysis](#cost-analysis)\n")
	sb.WriteString("- [API Versioning](#api-versioning)\n")
	sb.WriteString("- [Multi-Region Architecture](#multi-region-architecture)\n")
	sb.WriteString("- [Data Lifecycle Management](#data-lifecycle-management)\n")
	sb.WriteString("- [Policies](#policies)\n")
	sb.WriteString("- [Flows](#flows)\n")
	sb.WriteString("- [Integration Contracts](#integration-contracts)\n")
	sb.WriteString("- [Scenarios](#scenarios)\n")
	sb.WriteString("- [Relations](#relations)\n")
	sb.WriteString("- [Data Consistency Models](#data-consistency-models)\n")
	sb.WriteString("- [Constraints](#constraints)\n")
	sb.WriteString("- [Conventions](#conventions)\n")
	sb.WriteString("- [Document Metadata](#document-metadata)\n")
	sb.WriteString("- [Glossary](#glossary)\n")
}

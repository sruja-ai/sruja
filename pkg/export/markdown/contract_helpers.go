// pkg/export/markdown/contract_helpers.go
// Package markdown provides markdown generation.
//
//nolint:gocritic // Use WriteString for consistency
package markdown

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// writeContractHeader writes the contract header
func writeContractHeader(sb *strings.Builder, contract *language.Contract) {
	contractType := contract.Kind
	if contractType == "" {
		contractType = "contract"
	}
	sb.WriteString(fmt.Sprintf("### %s: %s\n\n", contractType, contract.ID))
}

// writeContractDetails writes basic contract details (status, version, endpoint, method)
func writeContractDetails(sb *strings.Builder, body *language.ContractBody) {
	if body.Status != nil {
		sb.WriteString(fmt.Sprintf("**Status**: %s\n\n", *body.Status))
	}
	if body.Version != nil {
		sb.WriteString(fmt.Sprintf("**Version**: %s\n\n", *body.Version))
	}
	if body.Endpoint != nil {
		sb.WriteString(fmt.Sprintf("**Endpoint**: %s\n\n", *body.Endpoint))
	}
	if body.Method != nil {
		sb.WriteString(fmt.Sprintf("**Method**: %s\n\n", *body.Method))
	}
}

// writeSchema writes a schema (request or response)
func writeSchema(sb *strings.Builder, title string, schema *language.SchemaBlock) {
	if schema == nil || len(schema.Entries) == 0 {
		return
	}
	sb.WriteString(fmt.Sprintf("**%s**:\n\n", title))
	for _, entry := range schema.Entries {
		typeStr := "unknown"
		if entry.Type != nil {
			typeStr = entry.Type.Name
		}
		sb.WriteString(fmt.Sprintf("- `%s`: %s\n", entry.Key, typeStr))
	}
	sb.WriteString("\n")
}

// writeContractErrors writes error codes
func writeContractErrors(sb *strings.Builder, errors []string) {
	if len(errors) > 0 {
		sb.WriteString(fmt.Sprintf("**Error Codes**: %s\n\n", strings.Join(errors, ", ")))
	}
}

// writeContractGuarantees writes service level guarantees
func writeContractGuarantees(sb *strings.Builder, guarantees *language.GuaranteesBlock) {
	if guarantees == nil || len(guarantees.Entries) == 0 {
		return
	}
	sb.WriteString("**Service Level Guarantees**:\n\n")
	for _, guarantee := range guarantees.Entries {
		sb.WriteString(fmt.Sprintf("- **%s**: %s\n", guarantee.Key, guarantee.Value))
	}
	sb.WriteString("\n")
}

// groupContractsByKind groups contracts by their kind
func groupContractsByKind(contracts []*language.Contract) (apis, events, data []*language.Contract) {
	for _, c := range contracts {
		switch c.Kind {
		case "api":
			apis = append(apis, c)
		case "event":
			events = append(events, c)
		case "data":
			data = append(data, c)
		default:
			apis = append(apis, c)
		}
	}
	return apis, events, data
}

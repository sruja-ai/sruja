package markdown

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestWriteSchema(t *testing.T) {
	var sb strings.Builder

	// Test with nil schema
	writeSchema(&sb, "Request", nil)
	if sb.Len() > 0 {
		t.Error("Expected empty output for nil schema")
	}

	// Test with empty schema
	sb.Reset()
	writeSchema(&sb, "Request", &language.SchemaBlock{})
	if sb.Len() > 0 {
		t.Error("Expected empty output for empty schema")
	}

	// Test with valid schema
	sb.Reset()
	typeName := "string"
	schema := &language.SchemaBlock{
		Entries: []*language.SchemaEntry{
			{Key: "name", Type: &language.TypeSpec{Name: typeName}},
			{Key: "age", Type: &language.TypeSpec{Name: "int"}},
		},
	}
	writeSchema(&sb, "Request", schema)
	output := sb.String()

	if !strings.Contains(output, "**Request**:") {
		t.Error("Expected Request header")
	}
	if !strings.Contains(output, "`name`: string") {
		t.Error("Expected name field")
	}
	if !strings.Contains(output, "`age`: int") {
		t.Error("Expected age field")
	}

	// Test with nil type
	sb.Reset()
	schema = &language.SchemaBlock{
		Entries: []*language.SchemaEntry{
			{Key: "field", Type: nil},
		},
	}
	writeSchema(&sb, "Response", schema)
	if !strings.Contains(sb.String(), "`field`: unknown") {
		t.Error("Expected unknown type for nil Type")
	}
}

func TestWriteContractHeader(t *testing.T) {
	var sb strings.Builder

	// Test with kind
	contract := &language.Contract{
		ID:   "GetUser",
		Kind: "api",
	}
	writeContractHeader(&sb, contract)
	if !strings.Contains(sb.String(), "### api: GetUser") {
		t.Error("Expected api header")
	}

	// Test without kind (default to "contract")
	sb.Reset()
	contract = &language.Contract{
		ID:   "PostOrder",
		Kind: "",
	}
	writeContractHeader(&sb, contract)
	if !strings.Contains(sb.String(), "### contract: PostOrder") {
		t.Error("Expected contract header")
	}
}

func TestWriteContractDetails(t *testing.T) {
	var sb strings.Builder
	status := "active"
	version := "v1"
	endpoint := "/api/users"
	method := "GET"

	body := &language.ContractBody{
		Status:   &status,
		Version:  &version,
		Endpoint: &endpoint,
		Method:   &method,
	}

	writeContractDetails(&sb, body)
	output := sb.String()

	if !strings.Contains(output, "**Status**: active") {
		t.Error("Expected status")
	}
	if !strings.Contains(output, "**Version**: v1") {
		t.Error("Expected version")
	}
	if !strings.Contains(output, "**Endpoint**: /api/users") {
		t.Error("Expected endpoint")
	}
	if !strings.Contains(output, "**Method**: GET") {
		t.Error("Expected method")
	}
}

func TestWriteContractErrors(t *testing.T) {
	var sb strings.Builder

	// Test with errors
	writeContractErrors(&sb, []string{"404", "500"})
	if !strings.Contains(sb.String(), "**Error Codes**: 404, 500") {
		t.Error("Expected error codes")
	}

	// Test with no errors
	sb.Reset()
	writeContractErrors(&sb, []string{})
	if sb.Len() > 0 {
		t.Error("Expected empty output for no errors")
	}
}

func TestWriteContractGuarantees(t *testing.T) {
	var sb strings.Builder

	// Test with nil guarantees
	writeContractGuarantees(&sb, nil)
	if sb.Len() > 0 {
		t.Error("Expected empty output for nil guarantees")
	}

	// Test with empty guarantees
	sb.Reset()
	writeContractGuarantees(&sb, &language.GuaranteesBlock{})
	if sb.Len() > 0 {
		t.Error("Expected empty output for empty guarantees")
	}

	// Test with valid guarantees
	sb.Reset()
	guarantees := &language.GuaranteesBlock{
		Entries: []*language.GuaranteeEntry{
			{Key: "uptime", Value: "99.9%"},
			{Key: "latency", Value: "< 100ms"},
		},
	}
	writeContractGuarantees(&sb, guarantees)
	output := sb.String()

	if !strings.Contains(output, "**Service Level Guarantees**:") {
		t.Error("Expected guarantees header")
	}
	if !strings.Contains(output, "**uptime**: 99.9%") {
		t.Error("Expected uptime guarantee")
	}
	if !strings.Contains(output, "**latency**: < 100ms") {
		t.Error("Expected latency guarantee")
	}
}

func TestGroupContractsByKind(t *testing.T) {
	contracts := []*language.Contract{
		{ID: "GetUser", Kind: "api"},
		{ID: "UserCreated", Kind: "event"},
		{ID: "UserSchema", Kind: "data"},
		{ID: "PostUser", Kind: "api"},
		{ID: "Unknown", Kind: ""}, // Defaults to api
	}

	apis, events, data := groupContractsByKind(contracts)

	if len(apis) != 3 {
		t.Errorf("Expected 3 APIs, got %d", len(apis))
	}
	if len(events) != 1 {
		t.Errorf("Expected 1 event, got %d", len(events))
	}
	if len(data) != 1 {
		t.Errorf("Expected 1 data contract, got %d", len(data))
	}

	// Verify default behavior
	if apis[2].ID != "Unknown" {
		t.Error("Expected Unknown to be grouped as API")
	}
}

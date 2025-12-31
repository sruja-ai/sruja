// Package tests provides core integration tests for Sruja's fundamental functionality.
// These tests focus on stable, deterministic behavior of core logic:
// - Parsing of valid and invalid programs
// - Validation of critical rules (unique IDs, valid references)
// - Export functionality (JSON, DSL, Markdown)
// - Round-trip consistency
package tests

import (
	"encoding/json"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/engine"
	dslexport "github.com/sruja-ai/sruja/pkg/export/dsl"
	jsonexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/export/markdown"
	"github.com/sruja-ai/sruja/pkg/language"
)

// TestCorePipeline_ValidProgram tests the complete pipeline: Parse -> Validate -> Export
// This is the most fundamental integration test ensuring the core workflow works.
func TestCorePipeline_ValidProgram(t *testing.T) {
	dsl := `
		user = person "User"
		app = system "My App" {
			web = container "Web Server" {
				technology "Node.js"
			}
			db = database "Database" {
				technology "PostgreSQL"
			}
		}
		user -> app.web "visits"
		app.web -> app.db "reads/writes"
	`

	// Step 1: Parse
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, diags, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	if len(diags) > 0 {
		t.Fatalf("Parse returned diagnostics: %v", diags)
	}
	if program == nil {
		t.Fatal("Parse returned nil program")
	}

	// Step 2: Validate
	validator := engine.NewValidator()
	validator.RegisterDefaultRules()
	validationDiags := validator.Validate(program)

	// Filter out informational messages (cycles are valid)
	blockingErrors := filterBlockingErrors(validationDiags)
	if len(blockingErrors) > 0 {
		t.Fatalf("Validation failed: %v", formatDiagnostics(blockingErrors))
	}

	// Step 3: Export to JSON
	jsonExporter := jsonexport.NewExporter()
	jsonOutput, err := jsonExporter.Export(program)
	if err != nil {
		t.Fatalf("JSON export failed: %v", err)
	}
	if jsonOutput == "" {
		t.Fatal("JSON export returned empty string")
	}

	// Verify JSON is valid
	var jsonData map[string]interface{}
	if err := json.Unmarshal([]byte(jsonOutput), &jsonData); err != nil {
		t.Fatalf("Exported JSON is invalid: %v", err)
	}

	// Step 4: Export to DSL (via JSON model dump)
	modelDump := jsonExporter.ToModelDump(program)
	dslOutput := dslexport.Print(modelDump)
	if dslOutput == "" {
		t.Fatal("DSL export returned empty string")
	}

	// Step 5: Export to Markdown
	mdExporter := markdown.NewExporter(markdown.Options{})
	mdOutput := mdExporter.Export(program)
	if mdOutput == "" {
		t.Fatal("Markdown export returned empty string")
	}
}

// TestCorePipeline_InvalidSyntax tests that invalid syntax is properly rejected
func TestCorePipeline_InvalidSyntax(t *testing.T) {
	invalidDSL := `
		sys = system "System" {
			// Missing closing brace
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, diags, err := parser.Parse("test.sruja", invalidDSL)
	// Invalid syntax should either return error or diagnostics
	if err == nil && len(diags) == 0 {
		t.Fatal("Invalid syntax should produce error or diagnostics")
	}
	if program != nil {
		// If program was created despite errors, it should be invalid
		validator := engine.NewValidator()
		validator.RegisterDefaultRules()
		validationDiags := validator.Validate(program)
		blockingErrors := filterBlockingErrors(validationDiags)
		if len(blockingErrors) == 0 {
			t.Fatal("Invalid syntax should result in validation errors")
		}
	}
}

// TestValidation_UniqueIDs tests that duplicate IDs are detected
func TestValidation_UniqueIDs(t *testing.T) {
	dsl := `
		sys1 = system "System 1"
		sys1 = system "System 2"  // Duplicate ID
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}

	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	diags := validator.Validate(program)

	// Should have at least one error about duplicate ID
	hasDuplicateError := false
	for _, d := range diags {
		if d.Severity == diagnostics.SeverityError && strings.Contains(strings.ToLower(d.Message), "duplicate") {
			hasDuplicateError = true
			break
		}
	}

	if !hasDuplicateError {
		t.Fatal("Expected error for duplicate ID, but none found")
	}
}

// TestValidation_ValidReferences tests that invalid references are detected
func TestValidation_ValidReferences(t *testing.T) {
	dsl := `
		sys = system "System" {
			web = container "Web"
		}
		// Reference to non-existent element
		nonexistent -> sys.web "uses"
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}

	validator := engine.NewValidator()
	validator.RegisterRule(&engine.ValidReferenceRule{})
	diags := validator.Validate(program)

	// Should have at least one error about invalid reference
	hasInvalidRefError := false
	for _, d := range diags {
		if d.Severity == diagnostics.SeverityError && strings.Contains(strings.ToLower(d.Message), "reference") {
			hasInvalidRefError = true
			break
		}
	}

	if !hasInvalidRefError {
		t.Fatal("Expected error for invalid reference, but none found")
	}
}

// TestCoreLanguageConstructs_AllElements tests that all core element types can be parsed
func TestCoreLanguageConstructs_AllElements(t *testing.T) {
	dsl := `
		user = person "User"
		sys = system "System" {
			web = container "Web Container"
			api = component "API Component"
			db = database "Database"
			queue = queue "Message Queue"
		}
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, diags, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	if len(diags) > 0 {
		t.Fatalf("Parse returned diagnostics: %v", diags)
	}

	// Verify all element types are present
	if program == nil || program.Model == nil {
		t.Fatal("Program or Model is nil")
	}

	// Check that we have at least one of each type
	hasPerson := false
	hasSystem := false
	hasContainer := false
	hasComponent := false
	hasDatabase := false
	hasQueue := false

	for _, item := range program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			kind := item.ElementDef.Assignment.Kind
			switch kind {
			case "person":
				hasPerson = true
			case "system":
				hasSystem = true
				// Check nested elements
				body := item.ElementDef.GetBody()
				if body != nil {
					for _, bodyItem := range body.Items {
						if bodyItem.Element != nil && bodyItem.Element.Assignment != nil {
							nestedKind := bodyItem.Element.Assignment.Kind
							switch nestedKind {
							case "container":
								hasContainer = true
							case "component":
								hasComponent = true
							case "database":
								hasDatabase = true
							case "queue":
								hasQueue = true
							}
						}
					}
				}
			}
		}
	}

	if !hasPerson {
		t.Error("Person element not found")
	}
	if !hasSystem {
		t.Error("System element not found")
	}
	if !hasContainer {
		t.Error("Container element not found")
	}
	if !hasComponent {
		t.Error("Component element not found")
	}
	if !hasDatabase {
		t.Error("Database element not found")
	}
	if !hasQueue {
		t.Error("Queue element not found")
	}
}

// TestCoreLanguageConstructs_Relations tests that relations can be parsed and validated
func TestCoreLanguageConstructs_Relations(t *testing.T) {
	dsl := `
		user = person "User"
		sys = system "System" {
			web = container "Web"
		}
		user -> sys.web "visits"
		sys.web -> user "responds to"
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, diags, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	if len(diags) > 0 {
		t.Fatalf("Parse returned diagnostics: %v", diags)
	}

	// Verify relations are present
	if program == nil || program.Model == nil {
		t.Fatal("Program or Model is nil")
	}

	// Count top-level relations (may also have nested relations in element bodies)
	topLevelRelationCount := 0
	for _, item := range program.Model.Items {
		if item.Relation != nil {
			topLevelRelationCount++
		}
	}

	// Should have at least 2 top-level relations
	if topLevelRelationCount < 2 {
		t.Errorf("Expected at least 2 top-level relations, found %d", topLevelRelationCount)
	}
}

// TestExport_JSONDeterministic tests that JSON export is deterministic
func TestExport_JSONDeterministic(t *testing.T) {
	dsl := `
		sys = system "System" {
			web = container "Web"
		}
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}

	jsonExporter := jsonexport.NewExporter()

	// Export multiple times - should produce identical output
	output1, err := jsonExporter.Export(program)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	output2, err := jsonExporter.Export(program)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	if output1 != output2 {
		t.Fatal("JSON export is not deterministic")
	}
}

// TestExport_JSONStructure tests that exported JSON has expected structure
func TestExport_JSONStructure(t *testing.T) {
	dsl := `
		sys = system "System" {
			web = container "Web"
		}
		sys -> sys "self"
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}

	jsonExporter := jsonexport.NewExporter()
	jsonOutput, err := jsonExporter.Export(program)
	if err != nil {
		t.Fatalf("Export failed: %v", err)
	}

	var jsonData map[string]interface{}
	if err := json.Unmarshal([]byte(jsonOutput), &jsonData); err != nil {
		t.Fatalf("Invalid JSON: %v", err)
	}

	// Verify required top-level fields
	// Note: keys in JSON are _stage, _metadata, but test unmarshals to map
	requiredFields := []string{"_stage", "project", "elements", "relations", "_metadata"}
	for _, field := range requiredFields {
		if _, exists := jsonData[field]; !exists {
			t.Errorf("Missing required field: %s", field)
		}
	}

	// Verify elements is a map
	elements, ok := jsonData["elements"].(map[string]interface{})
	if !ok {
		t.Fatal("elements should be a map")
	}

	// Verify we have the expected elements
	if len(elements) < 2 {
		t.Errorf("Expected at least 2 elements, got %d", len(elements))
	}
}

// TestExport_DSLRoundTrip tests that DSL export can be re-parsed
func TestExport_DSLRoundTrip(t *testing.T) {
	originalDSL := `
		sys = system "System" {
			web = container "Web Container" {
				technology "Node.js"
			}
		}
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	// Parse original
	program1, _, err := parser.Parse("test.sruja", originalDSL)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}

	// Export to DSL (via JSON model dump)
	jsonExporter := jsonexport.NewExporter()
	modelDump := jsonExporter.ToModelDump(program1)
	exportedDSL := dslexport.Print(modelDump)

	// Parse exported DSL
	program2, _, err := parser.Parse("test-exported.sruja", exportedDSL)
	if err != nil {
		t.Fatalf("Re-parse of exported DSL failed: %v", err)
	}

	// Verify both programs have the same structure
	if program1 == nil || program2 == nil {
		t.Fatal("One of the programs is nil")
	}

	// Basic structure check: both should have a model with at least one system
	if program1.Model == nil || program2.Model == nil {
		t.Fatal("One of the models is nil")
	}

	// Count systems in both
	systems1 := countSystems(program1)
	systems2 := countSystems(program2)

	if systems1 != systems2 {
		t.Errorf("System count mismatch: original=%d, exported=%d", systems1, systems2)
	}
}

// TestValidation_CycleDetection tests that cycles are detected (informational, not blocking)
func TestValidation_CycleDetection(t *testing.T) {
	dsl := `
		sys1 = system "System 1"
		sys2 = system "System 2"
		sys1 -> sys2 "uses"
		sys2 -> sys1 "uses"  // Creates a cycle
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}

	validator := engine.NewValidator()
	validator.RegisterRule(&engine.CycleDetectionRule{})
	diags := validator.Validate(program)

	// Note: Cycle detection may or may not produce messages, but if it does,
	// they should be informational, not blocking errors
	// The important thing is that validation doesn't fail
	blockingErrors := filterBlockingErrors(diags)
	if len(blockingErrors) > 0 {
		t.Fatalf("Cycles should not cause blocking errors: %v", formatDiagnostics(blockingErrors))
	}
}

// TestCoreLanguageConstructs_Scenarios tests that scenarios can be parsed
func TestCoreLanguageConstructs_Scenarios(t *testing.T) {
	dsl := `
		sys = system "System" {
			web = container "Web"
		}
		user = person "User"
		S1 = scenario "User visits site" {
			step user -> sys.web "visits"
		}
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, diags, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	if len(diags) > 0 {
		t.Fatalf("Parse returned diagnostics: %v", diags)
	}

	// Verify scenario is present
	if program == nil || program.Model == nil {
		t.Fatal("Program or Model is nil")
	}

	hasScenario := false
	for _, item := range program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			if item.ElementDef.Assignment.Kind == "scenario" {
				hasScenario = true
				break
			}
		}
	}

	if !hasScenario {
		t.Error("Scenario not found in parsed program")
	}
}

// TestCoreLanguageConstructs_ADRs tests that ADRs can be parsed
func TestCoreLanguageConstructs_ADRs(t *testing.T) {
	dsl := `
		ADR001 = adr "Use microservices" {
			status "accepted"
			context "We need to scale"
			decision "Use microservices architecture"
		}
	`

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, diags, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Parse failed: %v", err)
	}
	if len(diags) > 0 {
		t.Fatalf("Parse returned diagnostics: %v", diags)
	}

	// Verify ADR is present
	if program == nil || program.Model == nil {
		t.Fatal("Program or Model is nil")
	}

	hasADR := false
	for _, item := range program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			if item.ElementDef.Assignment.Kind == "adr" {
				hasADR = true
				if item.ElementDef.GetID() != "ADR001" {
					t.Errorf("ADR ID mismatch: expected 'ADR001', got '%s'", item.ElementDef.GetID())
				}
				break
			}
		}
	}

	if !hasADR {
		t.Error("ADR not found in parsed program")
	}
}

// Helper functions

func filterBlockingErrors(diags []diagnostics.Diagnostic) []diagnostics.Diagnostic {
	var blocking []diagnostics.Diagnostic
	for i := range diags {
		d := &diags[i]
		// Skip informational cycle detection messages
		if d.Code == diagnostics.CodeCycleDetected && d.Severity == diagnostics.SeverityInfo {
			continue
		}
		// Skip simplicity guidance warnings
		if strings.Contains(d.Message, "Consider using") {
			continue
		}
		// Only consider Errors as blocking
		if d.Severity == diagnostics.SeverityError {
			blocking = append(blocking, *d)
		}
	}
	return blocking
}

func formatDiagnostics(diags []diagnostics.Diagnostic) string {
	var msgs []string
	for i := range diags {
		d := &diags[i]
		msgs = append(msgs, diagnostics.FormatDiagnostic(*d))
	}
	return strings.Join(msgs, "; ")
}

func countSystems(program *language.Program) int {
	if program == nil || program.Model == nil {
		return 0
	}
	count := 0
	for _, item := range program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			if item.ElementDef.Assignment.Kind == "system" {
				count++
			}
		}
	}
	return count
}

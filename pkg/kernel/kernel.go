// pkg/kernel/kernel.go
// Package kernel provides the Architecture Kernel for Sruja Notebooks.
package kernel

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/model"
	"github.com/sruja-ai/sruja/pkg/query"
)

var (
	// Default diagram compilers
	defaultMermaidCompiler = compiler.NewMermaidCompiler()
	defaultD2Compiler      = compiler.NewD2Compiler()
)

// CellID represents a unique identifier for a notebook cell.
type CellID string

// CellType represents the type of a notebook cell.
type CellType string

const (
	CellTypeDSL        CellType = "dsl"        // Architecture DSL code
	CellTypeQuery      CellType = "query"      // SrujaQL query
	CellTypeDiagram    CellType = "diagram"    // Diagram generation
	CellTypeValidation CellType = "validation" // Validation execution
	CellTypeSimulation CellType = "simulation" // Event lifecycle simulation
	CellTypeAI         CellType = "ai"         // AI-assisted refinement
	CellTypeMarkdown   CellType = "markdown"   // Markdown documentation
)

// Diagnostic represents a validation or error message.
type Diagnostic struct {
	Severity  string          `json:"severity"` // "error", "warning", "info"
	Message   string          `json:"message"`
	ElementID string          `json:"elementId,omitempty"`
	Location  *model.Location `json:"location,omitempty"`
}

// CellOutput represents the output from executing a cell.
type CellOutput struct {
	CellID      CellID       `json:"cellId"`
	OutputType  string       `json:"outputType"` // "text", "diagram", "json", etc.
	Data        interface{}  `json:"data"`       // Output data (SVG, JSON, text, etc.)
	Diagnostics []Diagnostic `json:"diagnostics,omitempty"`
	Timestamp   time.Time    `json:"timestamp"`
}

// ExecutionResult represents the result of executing a cell.
type ExecutionResult struct {
	CellID      CellID       `json:"cellId"`
	Success     bool         `json:"success"`
	Outputs     []CellOutput `json:"outputs,omitempty"`
	Diagnostics []Diagnostic `json:"diagnostics,omitempty"`
	IRChanged   bool         `json:"irChanged"` // Whether architecture IR was modified
	Timestamp   time.Time    `json:"timestamp"`
	Error       string       `json:"error,omitempty"` // Error message (not error type for JSON)
}

// Kernel is the main execution engine for Sruja Architecture Notebooks.
//
// The kernel maintains stateful architecture model and provides:
//   - Incremental DSL execution
//   - Validation
//   - Diagram generation
//   - Query execution
//   - Snapshot/variant management
type Kernel struct {
	store            *ArchitectureStore
	parser           *language.Parser
	transformer      *compiler.Transformer
	validator        *engine.Validator
	queryEngine      *query.Engine
	symbols          *SymbolTable
	snapshots        *SnapshotManager
	variants         *VariantManager
	simulationEngine *SimulationEngine
	cellHistory      map[CellID]ExecutionResult
	lastProgram      *language.Program // Last parsed program for validation
	cache            *KernelCache      // Performance cache
}

// NewKernel creates a new kernel instance.
func NewKernel() (*Kernel, error) {
	parser, err := language.NewParser()
	if err != nil {
		return nil, fmt.Errorf("failed to create parser: %w", err)
	}

	store := NewArchitectureStore()
	snapshots := NewSnapshotManager(store)

	return &Kernel{
		store:            store,
		parser:           parser,
		transformer:      compiler.NewTransformer(),
		validator:        engine.NewValidator(),
		queryEngine:      query.NewEngineFromModel(store.GetModel()),
		symbols:          NewSymbolTable(),
		snapshots:        snapshots,
		variants:         NewVariantManager(snapshots, store),
		simulationEngine: NewSimulationEngine(),
		cellHistory:      make(map[CellID]ExecutionResult),
	}, nil
}

// ExecuteCell executes a notebook cell and updates the kernel state.
//
// This is the main entry point for executing cells. It:
//  1. Parses DSL (if DSL cell)
//  2. Updates semantic model
//  3. Runs validators
//  4. Produces outputs (diagrams, diagnostics, etc.)
//  5. Updates kernel state
func (k *Kernel) ExecuteCell(cellID CellID, cellType CellType, source string) (*ExecutionResult, error) {
	result := &ExecutionResult{
		CellID:    cellID,
		Success:   false,
		IRChanged: false,
		Timestamp: time.Now(),
	}

	// Remove previous contributions from this cell (for re-execution)
	k.store.RemoveElementsByCell(string(cellID))

	// Check for magic commands in any cell type (except markdown)
	if cellType != CellTypeMarkdown && IsMagicCommand(source) {
		return k.executeMagicCommand(cellID, source, result)
	}

	switch cellType {
	case CellTypeDSL:
		return k.executeDSLCell(cellID, source, result)
	case CellTypeQuery:
		return k.executeQueryCell(cellID, source, result)
	case CellTypeDiagram:
		return k.executeDiagramCell(cellID, source, result)
	case CellTypeValidation:
		return k.executeValidationCell(cellID, source, result)
	case CellTypeSimulation:
		return k.executeSimulationCell(cellID, source, result)
	case CellTypeAI:
		return k.executeAICell(cellID, source, result)
	case CellTypeMarkdown:
		// Markdown cells don't execute, just return success
		result.Success = true
		return result, nil
	default:
		return nil, fmt.Errorf("unknown cell type: %s", cellType)
	}
}

// executeDSLCell executes a DSL cell containing architecture definitions.
func (k *Kernel) executeDSLCell(cellID CellID, source string, result *ExecutionResult) (*ExecutionResult, error) {
	// Parse DSL
	program, err := k.parser.Parse(string(cellID), source)
	if err != nil {
		result.Error = err.Error()
		result.Diagnostics = append(result.Diagnostics, Diagnostic{
			Severity: "error",
			Message:  fmt.Sprintf("Parse error: %v", err),
		})
		k.cellHistory[cellID] = *result
		return result, nil // Return result even on error
	}

	// Transform AST to Model
	model, err := k.transformer.Transform(program)
	if err != nil {
		result.Error = err.Error()
		result.Diagnostics = append(result.Diagnostics, Diagnostic{
			Severity: "error",
			Message:  fmt.Sprintf("Transform error: %v", err),
		})
		k.cellHistory[cellID] = *result
		return result, nil
	}

	// Update symbol table
	k.updateSymbolTable(cellID, program)

	// Update architecture store
	if err := k.store.UpdateModel(model); err != nil {
		result.Error = err.Error()
		result.Diagnostics = append(result.Diagnostics, Diagnostic{
			Severity: "error",
			Message:  fmt.Sprintf("Store update error: %v", err),
		})
		k.cellHistory[cellID] = *result
		return result, nil
	}

	result.IRChanged = true

	// Store last parsed program for validation cells
	k.lastProgram = program

	// Update query engine with latest model
	k.queryEngine.SetModel(k.store.GetModel())

	// Build FSMs from entities and register event effects for simulation
	k.buildSimulationModels(program)

	// Validate
	validationErrors := k.validator.Validate(program)
	result.Diagnostics = append(result.Diagnostics, k.collectDiagnostics(validationErrors, nil)...)

	// Update diagnostics with cell ID for locations
	for i := range result.Diagnostics {
		if result.Diagnostics[i].Location != nil && result.Diagnostics[i].Location.File == "" {
			result.Diagnostics[i].Location.File = string(cellID)
		}
	}

	// Generate output
	result.Outputs = append(result.Outputs, CellOutput{
		CellID:      cellID,
		OutputType:  "text",
		Data:        "Architecture updated successfully",
		Diagnostics: result.Diagnostics,
		Timestamp:   time.Now(),
	})

	// Add IR output
	irJSON, _ := k.store.ToJSON()
	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "application/sruja-ir+json",
		Data:       string(irJSON),
		Timestamp:  time.Now(),
	})

	result.Success = len(validationErrors) == 0
	k.cellHistory[cellID] = *result

	return result, nil
}

// executeQueryCell executes a SrujaQL query cell.
func (k *Kernel) executeQueryCell(cellID CellID, source string, result *ExecutionResult) (*ExecutionResult, error) {
	// Update query engine with current model
	currentModel := k.store.GetModel()
	k.queryEngine.SetModel(currentModel)

	// Execute query
	queryResult, err := k.queryEngine.ExecuteFromModel(source, currentModel)
	if err != nil {
		result.Error = err.Error()
		result.Diagnostics = append(result.Diagnostics, Diagnostic{
			Severity: "error",
			Message:  fmt.Sprintf("Query error: %v", err),
		})
		result.Success = false
		k.cellHistory[cellID] = *result
		return result, nil
	}

	// Format results as JSON
	jsonData, err := json.MarshalIndent(queryResult, "", "  ")
	if err != nil {
		result.Error = err.Error()
		result.Diagnostics = append(result.Diagnostics, Diagnostic{
			Severity: "error",
			Message:  fmt.Sprintf("Failed to serialize query results: %v", err),
		})
		result.Success = false
		k.cellHistory[cellID] = *result
		return result, nil
	}

	// Add JSON output
	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "application/json",
		Data:       string(jsonData),
		Timestamp:  time.Now(),
	})

	// Add human-readable text output
	textOutput := formatQueryResult(queryResult)
	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "text/plain",
		Data:       textOutput,
		Timestamp:  time.Now(),
	})

	// Cache the result
	k.cache.SetQueryResult(source, queryResult)

	result.Success = true
	k.cellHistory[cellID] = *result
	return result, nil
}

// executeDiagramCell generates a diagram from the architecture with caching.
func (k *Kernel) executeDiagramCell(cellID CellID, source string, result *ExecutionResult) (*ExecutionResult, error) {
	// Parse diagram command
	cmd := ParseDiagramCommand(source)

	// Determine format (default to mermaid)
	diagramFormat := cmd.Format
	if diagramFormat == "" {
		diagramFormat = "mermaid" // Default format
	}

	// Create cache key from command and model version
	cacheKey := fmt.Sprintf("%s:%s:%d", source, diagramFormat, k.store.GetVersion())

	// Check cache first
	if cached, ok := k.cache.GetDiagram(cacheKey); ok {
		result.Outputs = append(result.Outputs, CellOutput{
			CellID:     cellID,
			OutputType: "text/plain",
			Data:       fmt.Sprintf("Generated %s diagram\n\n%s", diagramFormat, cached),
			Timestamp:  time.Now(),
		})
		result.Success = true
		return result, nil
	}

	// Get current model from store
	currentModel := k.store.GetModel()
	if currentModel == nil || currentModel.Architecture == nil {
		result.Outputs = append(result.Outputs, CellOutput{
			CellID:     cellID,
			OutputType: "text",
			Data:       "No architecture model available. Define architecture in DSL cells first.",
			Timestamp:  time.Now(),
		})
		result.Success = false
		k.cellHistory[cellID] = *result
		return result, nil
	}

	// Filter model if specific target requested
	diagramModel := currentModel
	if cmd.TargetType != "" || cmd.TargetID != "" {
		var err error
		diagramModel, err = k.filterModelForDiagram(currentModel, cmd)
		if err != nil {
			result.Error = err.Error()
			result.Diagnostics = append(result.Diagnostics, Diagnostic{
				Severity: "error",
				Message:  fmt.Sprintf("Failed to filter model: %v", err),
			})
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
	}

	// Generate diagram
	var diagramOutput string
	var mimeType string
	var err error

	switch diagramFormat {
	case "d2":
		diagramOutput, err = defaultD2Compiler.CompileFromModel(diagramModel)
		mimeType = "text/d2"
	case "mermaid":
		fallthrough
	default:
		diagramOutput, err = defaultMermaidCompiler.CompileFromModel(diagramModel)
		mimeType = "text/mermaid"
	}

	if err != nil {
		result.Error = err.Error()
		result.Diagnostics = append(result.Diagnostics, Diagnostic{
			Severity: "error",
			Message:  fmt.Sprintf("Diagram generation error: %v", err),
		})
		result.Success = false
		k.cellHistory[cellID] = *result
		return result, nil
	}

	// Add diagram output
	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: mimeType,
		Data:       diagramOutput,
		Timestamp:  time.Now(),
	})

	// Also add plain text output for readability
	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "text/plain",
		Data:       fmt.Sprintf("Generated %s diagram\n\n%s", diagramFormat, diagramOutput),
		Timestamp:  time.Now(),
	})

	// Cache the diagram
	k.cache.SetDiagram(cacheKey, diagramOutput)

	result.Success = true
	k.cellHistory[cellID] = *result
	return result, nil
}

// filterModelForDiagram filters the model to include only relevant elements for a diagram.
func (k *Kernel) filterModelForDiagram(m *model.Model, cmd *DiagramCommand) (*model.Model, error) {
	// Clone the model
	filtered := m.Clone()

	// Filter elements
	var filteredElements []model.Element
	for _, elem := range filtered.Architecture.Elements {
		// Filter by type if specified
		if cmd.TargetType != "" {
			switch cmd.TargetType {
			case "system":
				if elem.Type != model.ElementTypeSystem {
					continue
				}
			case "container":
				if elem.Type != model.ElementTypeContainer {
					continue
				}
			case "component":
				if elem.Type != model.ElementTypeComponent {
					continue
				}
			}
		}

		// Filter by ID if specified
		if cmd.TargetID != "" && elem.ID != cmd.TargetID {
			continue
		}

		filteredElements = append(filteredElements, elem)
	}

	// Filter relations to only include those connecting filtered elements
	elementIDs := make(map[string]bool)
	for _, elem := range filteredElements {
		elementIDs[elem.ID] = true
	}

	var filteredRelations []model.Relation
	for _, rel := range filtered.Architecture.Relations {
		if elementIDs[rel.From] && elementIDs[rel.To] {
			filteredRelations = append(filteredRelations, rel)
		}
	}

	filtered.Architecture.Elements = filteredElements
	filtered.Architecture.Relations = filteredRelations

	return filtered, nil
}

// executeValidationCell runs validations.
func (k *Kernel) executeValidationCell(cellID CellID, source string, result *ExecutionResult) (*ExecutionResult, error) {
	// Parse validation command
	cmd := ParseValidationCommand(source)

	// Check if we have a program to validate
	if k.lastProgram == nil {
		result.Outputs = append(result.Outputs, CellOutput{
			CellID:     cellID,
			OutputType: "text",
			Data:       "No architecture to validate. Define architecture in DSL cells first.",
			Timestamp:  time.Now(),
		})
		result.Success = false
		k.cellHistory[cellID] = *result
		return result, nil
	}

	// Run validation
	validationErrors := k.validator.Validate(k.lastProgram)
	allDiagnostics := k.collectDiagnostics(validationErrors, nil)

	// Filter diagnostics based on command
	var filteredDiagnostics []Diagnostic
	if cmd.TargetType != "" || cmd.TargetID != "" || cmd.RuleName != "" {
		filteredDiagnostics = k.filterDiagnostics(allDiagnostics, cmd)
	} else {
		filteredDiagnostics = allDiagnostics
	}

	result.Diagnostics = filteredDiagnostics

	// Format diagnostics for output
	var summary strings.Builder
	if len(filteredDiagnostics) == 0 {
		summary.WriteString("✅ Validation passed! No errors or warnings found.\n")
	} else {
		errorCount := 0
		warningCount := 0
		for _, diag := range filteredDiagnostics {
			if diag.Severity == "error" {
				errorCount++
			} else if diag.Severity == "warning" {
				warningCount++
			}
		}
		summary.WriteString(fmt.Sprintf("Validation results: %d error(s), %d warning(s)\n\n", errorCount, warningCount))

		for _, diag := range filteredDiagnostics {
			summary.WriteString(fmt.Sprintf("[%s] %s", strings.ToUpper(diag.Severity), diag.Message))
			if diag.Location != nil {
				summary.WriteString(fmt.Sprintf(" (at %s:%d:%d)", diag.Location.File, diag.Location.Line, diag.Location.Column))
			}
			summary.WriteString("\n")
		}
	}

	// Add text output
	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "text",
		Data:       summary.String(),
		Timestamp:  time.Now(),
	})

	// Add JSON diagnostics output
	jsonData, err := json.MarshalIndent(filteredDiagnostics, "", "  ")
	if err == nil {
		result.Outputs = append(result.Outputs, CellOutput{
			CellID:     cellID,
			OutputType: "application/sruja-diagnostics+json",
			Data:       string(jsonData),
			Timestamp:  time.Now(),
		})
	}

	result.Success = len(filteredDiagnostics) == 0 || (len(filteredDiagnostics) > 0 && !hasErrors(filteredDiagnostics))
	k.cellHistory[cellID] = *result
	return result, nil
}

// filterDiagnostics filters diagnostics based on validation command.
func (k *Kernel) filterDiagnostics(diagnostics []Diagnostic, cmd *ValidationCommand) []Diagnostic {
	var filtered []Diagnostic

	for _, diag := range diagnostics {
		// Filter by rule name if specified
		if cmd.RuleName != "" {
			// Note: Current diagnostics don't include rule names, so we skip this filter for now
			// This would require enhancing the Diagnostic type
			// For now, include all diagnostics if rule filtering is requested
			if cmd.TargetType == "" && cmd.TargetID == "" {
				filtered = append(filtered, diag)
			}
			continue
		}

		// Filter by target type and ID
		if cmd.TargetType != "" || cmd.TargetID != "" {
			// Check if diagnostic location matches the target
			if diag.Location != nil {
				// For now, we'll include all diagnostics that might be related
				// In the future, we could enhance this to match specific element IDs
				filtered = append(filtered, diag)
			} else {
				// Include diagnostics without specific location (global issues)
				filtered = append(filtered, diag)
			}
		} else {
			filtered = append(filtered, diag)
		}
	}

	return filtered
}

// hasErrors checks if any diagnostics are errors.
func hasErrors(diagnostics []Diagnostic) bool {
	for _, diag := range diagnostics {
		if diag.Severity == "error" {
			return true
		}
	}
	return false
}

// executeSimulationCell handles event lifecycle simulation.
func (k *Kernel) executeSimulationCell(cellID CellID, source string, result *ExecutionResult) (*ExecutionResult, error) {
	// Parse simulation command
	cmd, err := ParseSimulationCommand(source)
	if err != nil {
		result.Error = err.Error()
		result.Diagnostics = append(result.Diagnostics, Diagnostic{
			Severity: "error",
			Message:  fmt.Sprintf("Simulation command parse error: %v", err),
		})
		k.cellHistory[cellID] = *result
		return result, nil
	}

	// Run simulation
	sim, err := k.simulationEngine.Simulate(cmd.EntityName, cmd.InitialState, cmd.Events)
	if err != nil {
		result.Error = err.Error()
		result.Diagnostics = append(result.Diagnostics, Diagnostic{
			Severity: "error",
			Message:  fmt.Sprintf("Simulation error: %v", err),
		})
		k.cellHistory[cellID] = *result
		return result, nil
	}

	// Format results
	textOutput := FormatSimulationResult(sim)

	// Add text output
	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "text",
		Data:       textOutput,
		Timestamp:  time.Now(),
	})

	// Add JSON output for structured data
	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "application/sruja-simulation+json",
		Data:       sim,
		Timestamp:  time.Now(),
	})

	// Add diagnostics for invalid transitions
	for _, invalid := range sim.InvalidTransitions {
		result.Diagnostics = append(result.Diagnostics, Diagnostic{
			Severity: "error",
			Message:  invalid.Reason,
		})
	}

	// Add warnings
	for _, warning := range sim.Warnings {
		result.Diagnostics = append(result.Diagnostics, Diagnostic{
			Severity: "warning",
			Message:  warning,
		})
	}

	result.Success = len(sim.InvalidTransitions) == 0
	k.cellHistory[cellID] = *result
	return result, nil
}

// executeAICell handles AI-assisted refinement (placeholder).
func (k *Kernel) executeAICell(cellID CellID, source string, result *ExecutionResult) (*ExecutionResult, error) {
	// Note: AI/MCP integration handled via IDE (Cursor/VS Code) rather than kernel cells
	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "text",
		Data:       "AI cell execution not yet implemented",
		Timestamp:  time.Now(),
	})
	result.Success = true
	k.cellHistory[cellID] = *result
	return result, nil
}

// executeMagicCommand executes a magic command.
func (k *Kernel) executeMagicCommand(cellID CellID, source string, result *ExecutionResult) (*ExecutionResult, error) {
	cmd := ParseMagicCommand(source)
	if cmd == nil {
		result.Error = "Invalid magic command"
		result.Success = false
		k.cellHistory[cellID] = *result
		return result, nil
	}

	switch cmd.Command {
	case "ir":
		return k.executeIRCommand(cellID, result)
	case "snapshot":
		return k.executeSnapshotCommand(cellID, cmd, result)
	case "variant":
		return k.executeVariantCommand(cellID, cmd, result)
	case "validate":
		// Delegate to validation cell execution
		return k.executeValidationCell(cellID, strings.TrimPrefix(source, "%"), result)
	case "reset":
		return k.executeResetCommand(cellID, result)
	default:
		result.Error = fmt.Sprintf("Unknown magic command: %%%s", cmd.Command)
		result.Success = false
		result.Outputs = append(result.Outputs, CellOutput{
			CellID:     cellID,
			OutputType: "text",
			Data:       fmt.Sprintf("Unknown magic command: %%%s\n\nAvailable commands:\n  %%ir\n  %%snapshot <name|list|load|delete>\n  %%variant <list|create|apply>\n  %%validate\n  %%reset", cmd.Command),
			Timestamp:  time.Now(),
		})
		k.cellHistory[cellID] = *result
		return result, nil
	}
}

// executeIRCommand shows the current IR.
func (k *Kernel) executeIRCommand(cellID CellID, result *ExecutionResult) (*ExecutionResult, error) {
	irJSON, err := k.ExportIR()
	if err != nil {
		result.Error = err.Error()
		result.Success = false
		k.cellHistory[cellID] = *result
		return result, nil
	}

	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "application/sruja-ir+json",
		Data:       string(irJSON),
		Timestamp:  time.Now(),
	})

	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "text",
		Data:       fmt.Sprintf("Current Architecture IR (JSON):\n\n%s", string(irJSON)),
		Timestamp:  time.Now(),
	})

	result.Success = true
	k.cellHistory[cellID] = *result
	return result, nil
}

// executeSnapshotCommand handles snapshot magic commands.
func (k *Kernel) executeSnapshotCommand(cellID CellID, cmd *MagicCommand, result *ExecutionResult) (*ExecutionResult, error) {
	var output strings.Builder

	switch cmd.SubCommand {
	case "list":
		snapshots := k.ListSnapshots()
		if len(snapshots) == 0 {
			output.WriteString("No snapshots available.\n")
		} else {
			output.WriteString(fmt.Sprintf("Snapshots (%d):\n\n", len(snapshots)))
			for _, snap := range snapshots {
				output.WriteString(fmt.Sprintf("  - %s", snap.Name))
				if snap.Description != "" {
					output.WriteString(fmt.Sprintf(": %s", snap.Description))
				}
				output.WriteString(fmt.Sprintf(" (created: %s)\n", snap.Timestamp.Format("2006-01-02 15:04:05")))
			}
		}
	case "load":
		if len(cmd.Args) == 0 {
			result.Error = "Usage: %snapshot load <name>"
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		snapshotName := cmd.Args[0]
		err := k.LoadSnapshot(snapshotName)
		if err != nil {
			result.Error = err.Error()
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		output.WriteString(fmt.Sprintf("✅ Loaded snapshot: %s\n", snapshotName))
		// Update query engine with new model
		k.queryEngine.SetModel(k.store.GetModel())
	case "delete":
		if len(cmd.Args) == 0 {
			result.Error = "Usage: %snapshot delete <name>"
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		snapshotName := cmd.Args[0]
		err := k.DeleteSnapshot(snapshotName)
		if err != nil {
			result.Error = err.Error()
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		output.WriteString(fmt.Sprintf("✅ Deleted snapshot: %s\n", snapshotName))
	default:
		// Create snapshot
		if len(cmd.Args) == 0 {
			result.Error = "Usage: %snapshot <name> [description]"
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		snapshotName := cmd.Args[0]
		description := ""
		if len(cmd.Args) > 1 {
			description = strings.Join(cmd.Args[1:], " ")
		}
		snap, err := k.CreateSnapshot(snapshotName, description)
		if err != nil {
			result.Error = err.Error()
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		output.WriteString(fmt.Sprintf("✅ Created snapshot: %s\n", snap.Name))
		if snap.Description != "" {
			output.WriteString(fmt.Sprintf("   Description: %s\n", snap.Description))
		}
	}

	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "text",
		Data:       output.String(),
		Timestamp:  time.Now(),
	})

	result.Success = true
	k.cellHistory[cellID] = *result
	return result, nil
}

// executeVariantCommand handles variant magic commands.
func (k *Kernel) executeVariantCommand(cellID CellID, cmd *MagicCommand, result *ExecutionResult) (*ExecutionResult, error) {
	var output strings.Builder

	switch cmd.SubCommand {
	case "list":
		variants := k.ListVariants()
		if len(variants) == 0 {
			output.WriteString("No variants available.\n")
		} else {
			output.WriteString(fmt.Sprintf("Variants (%d):\n\n", len(variants)))
			for _, variant := range variants {
				output.WriteString(fmt.Sprintf("  - %s", variant.Name))
				if variant.Description != "" {
					output.WriteString(fmt.Sprintf(": %s", variant.Description))
				}
				output.WriteString(fmt.Sprintf(" (base: %s, created: %s)\n", variant.Base, variant.CreatedAt.Format("2006-01-02 15:04:05")))
			}
		}
	case "create":
		if len(cmd.Args) < 1 {
			result.Error = "Usage: %variant create <name> [base-snapshot] [description]"
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		variantName := cmd.Args[0]
		baseSnapshot := ""
		if len(cmd.Args) > 1 {
			baseSnapshot = cmd.Args[1]
		}
		description := ""
		if len(cmd.Args) > 2 {
			description = strings.Join(cmd.Args[2:], " ")
		}
		variant, err := k.CreateVariant(variantName, baseSnapshot, description)
		if err != nil {
			result.Error = err.Error()
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		output.WriteString(fmt.Sprintf("✅ Created variant: %s\n", variant.Name))
		if variant.Description != "" {
			output.WriteString(fmt.Sprintf("   Description: %s\n", variant.Description))
		}
		output.WriteString(fmt.Sprintf("   Base snapshot: %s\n", variant.Base))
	case "apply":
		if len(cmd.Args) == 0 {
			result.Error = "Usage: %variant apply <name>"
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		variantName := cmd.Args[0]
		err := k.ApplyVariant(variantName)
		if err != nil {
			result.Error = err.Error()
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		output.WriteString(fmt.Sprintf("✅ Applied variant: %s\n", variantName))
		// Update query engine with new model
		k.queryEngine.SetModel(k.store.GetModel())
	case "merge":
		if len(cmd.Args) == 0 {
			result.Error = "Usage: %variant merge <name>"
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}
		variantName := cmd.Args[0]
		mergeResult, err := k.MergeVariant(variantName)
		if err != nil {
			result.Error = err.Error()
			result.Success = false
			k.cellHistory[cellID] = *result
			return result, nil
		}

		// Write merge explanation
		output.WriteString(mergeResult.Explanation)

		// Add JSON output for structured data
		jsonData, _ := json.MarshalIndent(mergeResult, "", "  ")
		result.Outputs = append(result.Outputs, CellOutput{
			CellID:     cellID,
			OutputType: "application/sruja-merge-result+json",
			Data:       string(jsonData),
			Timestamp:  time.Now(),
		})

		if !mergeResult.Success {
			result.Diagnostics = append(result.Diagnostics, Diagnostic{
				Severity: "error",
				Message:  fmt.Sprintf("Merge completed with %d conflict(s). Manual resolution required.", len(mergeResult.Conflicts)),
			})
		}

		// Update query engine with new model if merge was successful
		if mergeResult.Success {
			k.queryEngine.SetModel(k.store.GetModel())
		}
	default:
		result.Error = fmt.Sprintf("Unknown variant sub-command: %s\nUsage: %%variant <list|create|apply|merge>", cmd.SubCommand)
		result.Success = false
		k.cellHistory[cellID] = *result
		return result, nil
	}

	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "text",
		Data:       output.String(),
		Timestamp:  time.Now(),
	})

	result.Success = true
	k.cellHistory[cellID] = *result
	return result, nil
}

// executeResetCommand resets the kernel state.
func (k *Kernel) executeResetCommand(cellID CellID, result *ExecutionResult) (*ExecutionResult, error) {
	k.Reset()
	// Reinitialize query engine
	k.queryEngine = query.NewEngineFromModel(k.store.GetModel())

	result.Outputs = append(result.Outputs, CellOutput{
		CellID:     cellID,
		OutputType: "text",
		Data:       "✅ Kernel reset. Architecture model cleared.\n",
		Timestamp:  time.Now(),
	})

	result.Success = true
	k.cellHistory[cellID] = *result
	return result, nil
}

// updateSymbolTable updates the symbol table after parsing DSL.
func (k *Kernel) updateSymbolTable(cellID CellID, program *language.Program) {
	// Remove previous symbols from this cell
	k.symbols.RemoveSymbolsByFile(string(cellID))

	// Extract and add new symbols
	k.extractSymbolsFromProgram(cellID, program)
}

// buildSimulationModels builds FSMs from entities and registers event effects.
func (k *Kernel) buildSimulationModels(program *language.Program) {
	if program == nil || program.Architecture == nil {
		return
	}

	arch := program.Architecture

	// Build FSMs from entities at architecture level
	for _, entity := range arch.Entities {
		if entity != nil && entity.Body != nil && entity.Body.Lifecycle != nil {
			_, err := k.simulationEngine.BuildFSMFromEntity(entity)
			if err != nil {
				// Log error but continue
				continue
			}
		}
	}

	// Build FSMs from entities in domains
	for _, item := range arch.Items {
		if item.Domain != nil && item.Domain.EntitiesBlock != nil {
			for _, entity := range item.Domain.EntitiesBlock.Entities {
				if entity != nil && entity.Body != nil && entity.Body.Lifecycle != nil {
					_, err := k.simulationEngine.BuildFSMFromEntity(entity)
					if err != nil {
						// Log error but continue
						continue
					}
				}
			}
		}
	}

	// Register event effects from events at architecture level
	for _, event := range arch.Events {
		if event != nil {
			k.simulationEngine.RegisterEventEffect(event)
		}
	}

	// Register event effects from events in domains
	for _, item := range arch.Items {
		if item.Domain != nil && item.Domain.EventsBlock != nil {
			for _, event := range item.Domain.EventsBlock.Events {
				if event != nil {
					k.simulationEngine.RegisterEventEffect(event)
				}
			}
		}
	}
}

// GetModel returns the current architecture model (IR).
func (k *Kernel) GetModel() *model.Model {
	return k.store.GetModel()
}

// ExportIR exports the current architecture IR as JSON.
func (k *Kernel) ExportIR() ([]byte, error) {
	return k.store.ToJSON()
}

// ImportIR imports architecture IR from JSON.
func (k *Kernel) ImportIR(data []byte) error {
	return k.store.FromJSON(data)
}

// Reset clears the kernel state.
func (k *Kernel) Reset() {
	k.store.Reset()
	k.symbols = NewSymbolTable()
	k.cellHistory = make(map[CellID]ExecutionResult)
	// Snapshots and variants are preserved across reset
}

// Snapshot operations

// CreateSnapshot creates a snapshot of the current architecture state.
func (k *Kernel) CreateSnapshot(name, description string) (*Snapshot, error) {
	return k.snapshots.CreateSnapshot(name, description)
}

// GetSnapshot retrieves a snapshot by name.
func (k *Kernel) GetSnapshot(name string) (*Snapshot, bool) {
	return k.snapshots.GetSnapshot(name)
}

// ListSnapshots returns all snapshots.
func (k *Kernel) ListSnapshots() []*Snapshot {
	return k.snapshots.ListSnapshots()
}

// LoadSnapshot loads a snapshot into the architecture store.
func (k *Kernel) LoadSnapshot(name string) error {
	return k.snapshots.LoadSnapshot(name)
}

// DeleteSnapshot removes a snapshot.
func (k *Kernel) DeleteSnapshot(name string) error {
	return k.snapshots.DeleteSnapshot(name)
}

// Variant operations

// CreateVariant creates a new variant from a base snapshot.
func (k *Kernel) CreateVariant(name, baseSnapshot, description string) (*Variant, error) {
	return k.variants.CreateVariant(name, baseSnapshot, description)
}

// GetVariant retrieves a variant by name.
func (k *Kernel) GetVariant(name string) (*Variant, bool) {
	return k.variants.GetVariant(name)
}

// ListVariants returns all variants.
func (k *Kernel) ListVariants() []*Variant {
	return k.variants.ListVariants()
}

// ApplyVariant loads a variant's state into the main store.
func (k *Kernel) ApplyVariant(name string) error {
	return k.variants.ApplyVariant(name)
}

// MergeVariant merges a variant into the main architecture.
func (k *Kernel) MergeVariant(name string) (*MergeResult, error) {
	return k.variants.MergeVariant(name)
}

// GetVariantDiff computes the differences between a variant and its base.
func (k *Kernel) GetVariantDiff(name string) ([]ModelPatch, error) {
	return k.variants.ComputeVariantDiff(name)
}

// GetCellHistory returns the execution history for a cell.
func (k *Kernel) GetCellHistory(cellID CellID) (ExecutionResult, bool) {
	result, ok := k.cellHistory[cellID]
	return result, ok
}

// formatQueryResult formats query results as human-readable text.
func formatQueryResult(qr query.QueryResult) string {
	var b strings.Builder

	if len(qr.Elements) > 0 {
		b.WriteString("Elements:\n")
		for _, elem := range qr.Elements {
			b.WriteString(fmt.Sprintf("  - %s (%s): %s\n", elem.Type, elem.ID, elem.Label))
		}
	}

	if len(qr.Relations) > 0 {
		if len(qr.Elements) > 0 {
			b.WriteString("\n")
		}
		b.WriteString("Relations:\n")
		for _, rel := range qr.Relations {
			verb := rel.Verb
			if verb == "" {
				verb = "→"
			}
			b.WriteString(fmt.Sprintf("  - %s %s %s", rel.From, verb, rel.To))
			if rel.Label != "" {
				b.WriteString(fmt.Sprintf(" (%s)", rel.Label))
			}
			b.WriteString("\n")
		}
	}

	if len(qr.Elements) == 0 && len(qr.Relations) == 0 {
		return "No results found."
	}

	return b.String()
}

// pkg/import/json/json.go
// Package json provides JSON to AST conversion functionality.
package json

import (
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
	"strings"

	jsonexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/language"
)

// OutputFormat specifies how DSL files should be organized
type OutputFormat string

const (
	OutputFormatSingleFile    OutputFormat = "single"   // All in one file
	OutputFormatMultipleFiles OutputFormat = "multiple" // Concept-based files
)

// FileOutput represents a generated DSL file
type FileOutput struct {
	Path    string
	Content string
}

// stringToQualifiedIdent converts a string to a QualifiedIdent
func stringToQualifiedIdent(s string) language.QualifiedIdent {
	// Count dots to estimate capacity
	dotCount := 0
	for i := 0; i < len(s); i++ {
		if s[i] == '.' {
			dotCount++
		}
	}
	parts := make([]string, 0, dotCount+1)
	start := 0
	for i := 0; i < len(s); i++ {
		if s[i] == '.' {
			if i > start {
				parts = append(parts, s[start:i])
			}
			start = i + 1
		}
	}
	if start < len(s) {
		parts = append(parts, s[start:])
	} else if len(parts) == 0 {
		// Empty string case
		parts = append(parts, "")
	}
	return language.QualifiedIdent{Parts: parts}
}

// Converter converts JSON to AST
type Converter struct{}

// NewConverter creates a new JSON to AST converter
func NewConverter() *Converter {
	return &Converter{}
}

// ToArchitecture converts JSON to an Architecture AST
//
//nolint:funlen,gocyclo // Import logic is long and complex
func (c *Converter) ToArchitecture(jsonData []byte) (*language.Architecture, error) {
	if len(jsonData) == 0 {
		return nil, fmt.Errorf("empty JSON data")
	}

	var archJSON jsonexport.ArchitectureJSON
	if err := json.Unmarshal(jsonData, &archJSON); err != nil {
		// Provide more context in error message
		var jsonErr *json.SyntaxError
		if errors.As(err, &jsonErr) {
			return nil, fmt.Errorf("JSON syntax error at offset %d: %w", jsonErr.Offset, err)
		}
		return nil, fmt.Errorf("failed to unmarshal JSON: %w", err)
	}

	// Validate required fields
	if archJSON.Metadata.Name == "" {
		return nil, fmt.Errorf("missing required field: metadata.name (JSON must include metadata.name)")
	}

	arch := &language.Architecture{
		Name: archJSON.Metadata.Name,
	}

	// Import feature removed - no longer converting imports

	// Estimate total items for pre-allocation
	estimatedItems := len(archJSON.Architecture.Persons) +
		len(archJSON.Architecture.Systems) +
		len(archJSON.Architecture.Relations) +
		len(archJSON.Architecture.Requirements) +
		len(archJSON.Architecture.ADRs) +
		len(archJSON.Architecture.Scenarios) +
		len(archJSON.Architecture.Policies) +
		len(archJSON.Architecture.Flows) +
		len(archJSON.Architecture.Deployment)
	if len(archJSON.Architecture.Contracts) > 0 {
		estimatedItems++ // ContractsBlock counts as one item
	}
	arch.Items = make([]language.ArchitectureItem, 0, estimatedItems)

	// Convert persons
	if len(archJSON.Architecture.Persons) > 0 {
		for _, pJSON := range archJSON.Architecture.Persons {
			person := &language.Person{
				ID:          pJSON.ID,
				Label:       pJSON.Label,
				Description: pJSON.Description,
			}
			arch.Items = append(arch.Items, language.ArchitectureItem{Person: person})
		}
	}

	// Convert systems
	if len(archJSON.Architecture.Systems) > 0 {
		for i := range archJSON.Architecture.Systems {
			sys := convertSystemJSONToAST(&archJSON.Architecture.Systems[i])
			arch.Items = append(arch.Items, language.ArchitectureItem{System: sys})
		}
	}

	// Convert top-level relations
	if len(archJSON.Architecture.Relations) > 0 {
		for i, rJSON := range archJSON.Architecture.Relations {
			if rJSON.From == "" || rJSON.To == "" {
				return nil, fmt.Errorf("relation at index %d: missing required field 'from' or 'to'", i)
			}
			var verb *string
			if rJSON.Verb != nil {
				verb = rJSON.Verb
			}
			rel := &language.Relation{
				From:  stringToQualifiedIdent(rJSON.From),
				Arrow: "->",
				To:    stringToQualifiedIdent(rJSON.To),
				Verb:  verb,
				Label: rJSON.Label,
			}
			arch.Items = append(arch.Items, language.ArchitectureItem{Relation: rel})
		}
	}

	// Convert requirements
	if len(archJSON.Architecture.Requirements) > 0 {
		for _, reqJSON := range archJSON.Architecture.Requirements {
			req := convertRequirementJSONToAST(&reqJSON)
			arch.Items = append(arch.Items, language.ArchitectureItem{Requirement: req})
		}
	}

	// Convert ADRs
	if len(archJSON.Architecture.ADRs) > 0 {
		for _, adrJSON := range archJSON.Architecture.ADRs {
			adr := convertADRJSONToAST(&adrJSON)
			arch.Items = append(arch.Items, language.ArchitectureItem{ADR: adr})
		}
	}

	// Convert scenarios
	if len(archJSON.Architecture.Scenarios) > 0 {
		for _, scenJSON := range archJSON.Architecture.Scenarios {
			scen := convertScenarioJSONToAST(&scenJSON)
			arch.Items = append(arch.Items, language.ArchitectureItem{Scenario: scen})
		}
	}

	// Convert contracts
	if len(archJSON.Architecture.Contracts) > 0 {
		// Contracts are in a block, so we need to create a ContractsBlock
		contractsBlock := &language.ContractsBlock{
			Contracts: make([]*language.Contract, 0, len(archJSON.Architecture.Contracts)),
		}
		for _, contractJSON := range archJSON.Architecture.Contracts {
			contract := convertContractJSONToAST(&contractJSON)
			contractsBlock.Contracts = append(contractsBlock.Contracts, contract)
		}
		arch.Items = append(arch.Items, language.ArchitectureItem{ContractsBlock: contractsBlock})
	}

	// Convert policies
	if len(archJSON.Architecture.Policies) > 0 {
		for _, policyJSON := range archJSON.Architecture.Policies {
			policy := convertPolicyJSONToAST(&policyJSON)
			arch.Items = append(arch.Items, language.ArchitectureItem{Policy: policy})
		}
	}

	// Convert flows
	if len(archJSON.Architecture.Flows) > 0 {
		for _, flowJSON := range archJSON.Architecture.Flows {
			flow := convertFlowJSONToAST(&flowJSON)
			arch.Items = append(arch.Items, language.ArchitectureItem{Flow: flow})
		}
	}

	// Convert deployment nodes
	if len(archJSON.Architecture.Deployment) > 0 {
		for _, depJSON := range archJSON.Architecture.Deployment {
			dep := convertDeploymentNodeJSONToAST(&depJSON)
			arch.Items = append(arch.Items, language.ArchitectureItem{DeploymentNode: dep})
		}
	}

	// Convert shared artifacts
	if len(archJSON.Architecture.SharedArtifacts) > 0 {
		for _, saJSON := range archJSON.Architecture.SharedArtifacts {
			sa := convertSharedArtifactJSONToAST(&saJSON)
			arch.Items = append(arch.Items, language.ArchitectureItem{SharedArtifact: sa})
		}
	}

	// Convert libraries
	if len(archJSON.Architecture.Libraries) > 0 {
		for i := range archJSON.Architecture.Libraries {
			lib := convertLibraryJSONToAST(&archJSON.Architecture.Libraries[i])
			arch.Items = append(arch.Items, language.ArchitectureItem{Library: lib})
		}
	}

	// Convert constraints
	if len(archJSON.Architecture.Constraints) > 0 {
		constraintsBlock := &language.ConstraintsBlock{}
		for _, constraintJSON := range archJSON.Architecture.Constraints {
			constraint := &language.ConstraintEntry{
				Key:   constraintJSON.Key,
				Value: constraintJSON.Value,
			}
			constraintsBlock.Entries = append(constraintsBlock.Entries, constraint)
		}
		arch.Items = append(arch.Items, language.ArchitectureItem{ConstraintsBlock: constraintsBlock})
	}

	// Convert conventions
	if len(archJSON.Architecture.Conventions) > 0 {
		conventionsBlock := &language.ConventionsBlock{}
		for _, conventionJSON := range archJSON.Architecture.Conventions {
			convention := &language.ConventionEntry{
				Key:   conventionJSON.Key,
				Value: conventionJSON.Value,
			}
			conventionsBlock.Entries = append(conventionsBlock.Entries, convention)
		}
		arch.Items = append(arch.Items, language.ArchitectureItem{ConventionsBlock: conventionsBlock})
	}

	// Post-process to populate convenience fields
	arch.PostProcess()

	return arch, nil
}

// ToDSL converts JSON to DSL files based on output format
func (c *Converter) ToDSL(jsonData []byte, format OutputFormat) ([]FileOutput, error) {
	arch, err := c.ToArchitecture(jsonData)
	if err != nil {
		return nil, err
	}

	return generateDSLFiles(arch, format)
}

// generateDSLFiles generates DSL files based on output format
func generateDSLFiles(arch *language.Architecture, format OutputFormat) ([]FileOutput, error) {
	var files []FileOutput
	printer := language.NewPrinter()

	switch format {
	case OutputFormatSingleFile:
		// Single file with all sections
		program := &language.Program{Architecture: arch}
		content := printer.Print(program)
		files = append(files, FileOutput{
			Path:    sanitizeFileName(arch.Name) + ".sruja",
			Content: content,
		})

	case OutputFormatMultipleFiles:
		// Concept-based files with standard names
		// Architecture file (systems, containers, persons, relations)
		archOnly := extractArchitectureOnly(arch)
		archProgram := &language.Program{Architecture: archOnly}
		archContent := printer.Print(archProgram)
		files = append(files, FileOutput{
			Path:    "architecture.sruja",
			Content: archContent,
		})

		// Requirements file
		if len(arch.Requirements) > 0 {
			reqOnly := extractRequirementsOnly(arch)
			reqProgram := &language.Program{Architecture: reqOnly}
			reqContent := printer.Print(reqProgram)
			files = append(files, FileOutput{
				Path:    "requirements.sruja",
				Content: reqContent,
			})
		}

		// ADRs file
		if len(arch.ADRs) > 0 {
			adrOnly := extractADRsOnly(arch)
			adrProgram := &language.Program{Architecture: adrOnly}
			adrContent := printer.Print(adrProgram)
			files = append(files, FileOutput{
				Path:    "decisions.sruja",
				Content: adrContent,
			})
		}

		// Scenarios file
		if len(arch.Scenarios) > 0 {
			scenOnly := extractScenariosOnly(arch)
			scenProgram := &language.Program{Architecture: scenOnly}
			scenContent := printer.Print(scenProgram)
			files = append(files, FileOutput{
				Path:    "scenarios.sruja",
				Content: scenContent,
			})
		}
	}

	return files, nil
}

// sanitizeFileName converts a name to a valid filename
func sanitizeFileName(name string) string {
	// Trim whitespace first
	name = strings.TrimSpace(name)
	if name == "" {
		return "architecture"
	}

	// Replace spaces and special characters with underscores
	result := ""
	for _, r := range name {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || r == '-' || r == '_' {
			result += string(r)
		} else if r == ' ' {
			result += "_"
		}
	}
	if result == "" {
		result = "architecture"
	}
	return result
}

// extractArchitectureOnly creates a new Architecture with only structural elements
func extractArchitectureOnly(arch *language.Architecture) *language.Architecture {
	result := &language.Architecture{Name: arch.Name}
	for i := range arch.Items {
		item := arch.Items[i]
		if item.System != nil || item.Person != nil || item.Relation != nil ||
			item.Container != nil || item.Component != nil || item.DataStore != nil ||
			item.Queue != nil || item.DeploymentNode != nil {
			result.Items = append(result.Items, item)
		}
	}
	result.PostProcess()
	return result
}

// extractRequirementsOnly creates a new Architecture with only requirements
func extractRequirementsOnly(arch *language.Architecture) *language.Architecture {
	result := &language.Architecture{Name: arch.Name + " - Requirements"}
	for i := range arch.Items {
		item := arch.Items[i]
		if item.Requirement != nil {
			result.Items = append(result.Items, item)
		}
	}
	result.PostProcess()
	return result
}

// extractADRsOnly creates a new Architecture with only ADRs
func extractADRsOnly(arch *language.Architecture) *language.Architecture {
	result := &language.Architecture{Name: arch.Name + " - Decisions"}
	for i := range arch.Items {
		item := arch.Items[i]
		if item.ADR != nil {
			result.Items = append(result.Items, item)
		}
	}
	result.PostProcess()
	return result
}

// extractScenariosOnly creates a new Architecture with only scenarios
func extractScenariosOnly(arch *language.Architecture) *language.Architecture {
	result := &language.Architecture{Name: arch.Name + " - Scenarios"}
	for i := range arch.Items {
		item := arch.Items[i]
		if item.Scenario != nil {
			result.Items = append(result.Items, item)
		}
	}
	result.PostProcess()
	return result
}

// convertSystemJSONToAST converts a SystemJSON to a System AST
func convertSystemJSONToAST(sJSON *jsonexport.SystemJSON) *language.System {
	if sJSON.ID == "" {
		// Use label as fallback if ID is missing
		sJSON.ID = sJSON.Label
	}

	sys := &language.System{
		ID:          sJSON.ID,
		Label:       sJSON.Label,
		Description: sJSON.Description,
	}

	// Convert containers
	if len(sJSON.Containers) > 0 {
		for i := range sJSON.Containers {
			sys.Items = append(sys.Items, language.SystemItem{Container: convertContainerJSONToAST(&sJSON.Containers[i])})
		}
	}

	// Convert components
	// Note: Components at system level are not supported in SystemItem union
	// They would need to be added directly to sys.Components after PostProcess()
	// For now, skip - components are typically at container level
	if len(sJSON.Components) > 0 {
		// TODO: Handle system-level components when JSON exporter supports them
		_ = sJSON.Components
	}

	// Convert data stores
	if len(sJSON.DataStores) > 0 {
		for _, dsJSON := range sJSON.DataStores {
			ds := &language.DataStore{
				ID:    dsJSON.ID,
				Label: dsJSON.Label,
			}
			sys.Items = append(sys.Items, language.SystemItem{DataStore: ds})
		}
	}

	// Convert queues
	if len(sJSON.Queues) > 0 {
		for _, qJSON := range sJSON.Queues {
			q := &language.Queue{
				ID:    qJSON.ID,
				Label: qJSON.Label,
			}
			sys.Items = append(sys.Items, language.SystemItem{Queue: q})
		}
	}

	// Convert relations
	if len(sJSON.Relations) > 0 {
		for _, rJSON := range sJSON.Relations {
			if rJSON.From == "" || rJSON.To == "" {
				// Skip invalid relations - don't fail entire conversion
				continue
			}
			var verb *string
			if rJSON.Verb != nil {
				verb = rJSON.Verb
			}
			rel := &language.Relation{
				From:  stringToQualifiedIdent(rJSON.From),
				Arrow: "->",
				To:    stringToQualifiedIdent(rJSON.To),
				Verb:  verb,
				Label: rJSON.Label,
			}
			sys.Items = append(sys.Items, language.SystemItem{Relation: rel})
		}
	}

	return sys
}

// convertContainerJSONToAST converts a ContainerJSON to a Container AST
func convertContainerJSONToAST(cJSON *jsonexport.ContainerJSON) *language.Container {
	if cJSON.ID == "" {
		cJSON.ID = cJSON.Label
	}

	cont := &language.Container{
		ID:          cJSON.ID,
		Label:       cJSON.Label,
		Description: cJSON.Description,
	}

	// Convert components
	if len(cJSON.Components) > 0 {
		for i := range cJSON.Components {
			comp := convertComponentJSONToAST(&cJSON.Components[i])
			cont.Items = append(cont.Items, language.ContainerItem{Component: comp})
		}
	}

	// Convert data stores
	if len(cJSON.DataStores) > 0 {
		for _, dsJSON := range cJSON.DataStores {
			ds := &language.DataStore{
				ID:    dsJSON.ID,
				Label: dsJSON.Label,
			}
			cont.Items = append(cont.Items, language.ContainerItem{DataStore: ds})
		}
	}

	// Convert queues
	if len(cJSON.Queues) > 0 {
		for _, qJSON := range cJSON.Queues {
			q := &language.Queue{
				ID:    qJSON.ID,
				Label: qJSON.Label,
			}
			cont.Items = append(cont.Items, language.ContainerItem{Queue: q})
		}
	}

	// Convert relations
	if len(cJSON.Relations) > 0 {
		for _, rJSON := range cJSON.Relations {
			if rJSON.From == "" || rJSON.To == "" {
				continue // Skip invalid relations
			}
			var verb *string
			if rJSON.Verb != nil {
				verb = rJSON.Verb
			}
			rel := &language.Relation{
				From:  stringToQualifiedIdent(rJSON.From),
				Arrow: "->",
				To:    stringToQualifiedIdent(rJSON.To),
				Verb:  verb,
				Label: rJSON.Label,
			}
			cont.Items = append(cont.Items, language.ContainerItem{Relation: rel})
		}
	}

	return cont
}

// convertComponentJSONToAST converts a ComponentJSON to a Component AST
func convertComponentJSONToAST(compJSON *jsonexport.ComponentJSON) *language.Component {
	if compJSON.ID == "" {
		compJSON.ID = compJSON.Label
	}

	comp := &language.Component{
		ID:          compJSON.ID,
		Label:       compJSON.Label,
		Description: compJSON.Description,
	}

	// Convert relations
	if len(compJSON.Relations) > 0 {
		for _, rJSON := range compJSON.Relations {
			if rJSON.From == "" || rJSON.To == "" {
				continue // Skip invalid relations
			}
			var verb *string
			if rJSON.Verb != nil {
				verb = rJSON.Verb
			}
			rel := &language.Relation{
				From:  stringToQualifiedIdent(rJSON.From),
				Arrow: "->",
				To:    stringToQualifiedIdent(rJSON.To),
				Verb:  verb,
				Label: rJSON.Label,
			}
			comp.Items = append(comp.Items, language.ComponentItem{Relation: rel})
		}
	}

	return comp
}

// convertRequirementJSONToAST converts a RequirementJSON to a Requirement AST
func convertRequirementJSONToAST(reqJSON *jsonexport.RequirementJSON) *language.Requirement {
	if reqJSON.ID == "" {
		// Generate ID from title if missing
		reqJSON.ID = reqJSON.Title
	}

	req := &language.Requirement{
		ID:          reqJSON.ID,
		Type:        strPtr("functional"), // Default type when not specified
		Description: strPtr(reqJSON.Title),
	}
	return req
}

// convertADRJSONToAST converts an ADRJSON to an ADR AST
func convertADRJSONToAST(adrJSON *jsonexport.ADRJSON) *language.ADR {
	if adrJSON.ID == "" {
		adrJSON.ID = adrJSON.Title
	}

	var title *string
	if adrJSON.Title != "" {
		title = &adrJSON.Title
	}

	adr := &language.ADR{
		ID:    adrJSON.ID,
		Title: title,
	}
	return adr
}

// convertScenarioJSONToAST converts a ScenarioJSON to a Scenario AST
func convertScenarioJSONToAST(scenJSON *jsonexport.ScenarioJSON) *language.Scenario {
	if scenJSON.ID == "" {
		scenJSON.ID = scenJSON.Label
	}

	scen := &language.Scenario{
		ID:     scenJSON.ID,
		Title:  scenJSON.Label,
		LBrace: "{",
		RBrace: "}",
		Items:  []*language.ScenarioItem{}, // Empty steps - would need full JSON structure
	}
	return scen
}

// convertContractJSONToAST converts a ContractJSON to a Contract AST
func convertContractJSONToAST(contractJSON *jsonexport.ContractJSON) *language.Contract {
	// ContractJSON is a placeholder - need to check actual structure
	contract := &language.Contract{
		Kind: "api", // Default kind
		ID:   contractJSON.ID,
		L:    "{",
		R:    "}",
		Body: &language.ContractBody{},
	}
	return contract
}

// convertPolicyJSONToAST converts a PolicyJSON to a Policy AST
func convertPolicyJSONToAST(policyJSON *jsonexport.PolicyJSON) *language.Policy {
	// PolicyJSON is a placeholder - need to check actual structure
	policy := &language.Policy{
		ID:          policyJSON.ID,
		Description: policyJSON.Label,
	}
	return policy
}

// convertFlowJSONToAST converts a FlowJSON to a Flow AST
func convertFlowJSONToAST(flowJSON *jsonexport.FlowJSON) *language.Flow {
	flow := &language.Flow{
		ID:          flowJSON.ID,
		Title:       flowJSON.Title,
		Description: flowJSON.Description,
		LBrace:      "{",
		RBrace:      "}",
	}

	// Convert steps (Flow is alias to Scenario - uses ScenarioStep)
	if len(flowJSON.Steps) > 0 {
		for _, stepJSON := range flowJSON.Steps {
			// Parse qualified identifiers from strings
			step := &language.ScenarioStep{
				From:        stringToQualifiedIdent(stepJSON.From),
				To:          stringToQualifiedIdent(stepJSON.To),
				Description: stepJSON.Description,
				Tags:        stepJSON.Tags,
				Order:       intPtrToStringPtr(stepJSON.Order),
			}
			flow.Items = append(flow.Items, &language.ScenarioItem{Step: step})
		}
	}

	return flow
}

func intPtrToStringPtr(i *int) *string {
	if i == nil {
		return nil
	}
	s := strconv.Itoa(*i)
	return &s
}

// convertDeploymentNodeJSONToAST converts a DeploymentNodeJSON to a DeploymentNode AST
func convertDeploymentNodeJSONToAST(depJSON *jsonexport.DeploymentNodeJSON) *language.DeploymentNode {
	// DeploymentNodeJSON is a placeholder - need to check actual structure
	dep := &language.DeploymentNode{
		Type:  "deployment",
		ID:    depJSON.ID,
		Label: depJSON.Label,
	}
	return dep
}

// convertSharedArtifactJSONToAST converts a SharedArtifactJSON to a SharedArtifact AST
func convertSharedArtifactJSONToAST(saJSON *jsonexport.SharedArtifactJSON) *language.SharedArtifact {
	sa := &language.SharedArtifact{
		ID:    saJSON.ID,
		Label: saJSON.Label,
	}
	return sa
}

// convertLibraryJSONToAST converts a LibraryJSON to a Library AST
func convertLibraryJSONToAST(libJSON *jsonexport.LibraryJSON) *language.Library {
	lib := &language.Library{
		ID:    libJSON.ID,
		Label: libJSON.Label,
	}
	return lib
}

func strPtr(s string) *string {
	return &s
}

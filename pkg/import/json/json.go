// pkg/import/json/json.go
// Package json provides JSON to AST conversion functionality.
package json

import (
	"encoding/json"
	"errors"
	"fmt"

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

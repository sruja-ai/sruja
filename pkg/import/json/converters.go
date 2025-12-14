// pkg/import/json/converters.go
// Type-specific JSON to AST conversion functions
package json

import (
	"strconv"

	jsonexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/language"
)

// convertSystemJSONToAST converts a SystemJSON to a System AST
func convertSystemJSONToAST(sJSON *jsonexport.SystemJSON) *language.System {
	if sJSON.ID == "" {
		sJSON.ID = sJSON.Label
	}

	sys := &language.System{
		ID:          sJSON.ID,
		Label:       sJSON.Label,
		Description: sJSON.Description,
	}

	if len(sJSON.Containers) > 0 {
		for i := range sJSON.Containers {
			sys.Items = append(sys.Items, language.SystemItem{Container: convertContainerJSONToAST(&sJSON.Containers[i])})
		}
	}

	if len(sJSON.Components) > 0 {
		_ = sJSON.Components
	}

	if len(sJSON.DataStores) > 0 {
		for _, dsJSON := range sJSON.DataStores {
			ds := &language.DataStore{
				ID:    dsJSON.ID,
				Label: dsJSON.Label,
			}
			sys.Items = append(sys.Items, language.SystemItem{DataStore: ds})
		}
	}

	if len(sJSON.Queues) > 0 {
		for _, qJSON := range sJSON.Queues {
			q := &language.Queue{
				ID:    qJSON.ID,
				Label: qJSON.Label,
			}
			sys.Items = append(sys.Items, language.SystemItem{Queue: q})
		}
	}

	if len(sJSON.Relations) > 0 {
		for _, rJSON := range sJSON.Relations {
			if rJSON.From == "" || rJSON.To == "" {
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

	if len(cJSON.Components) > 0 {
		for i := range cJSON.Components {
			comp := convertComponentJSONToAST(&cJSON.Components[i])
			cont.Items = append(cont.Items, language.ContainerItem{Component: comp})
		}
	}

	if len(cJSON.DataStores) > 0 {
		for _, dsJSON := range cJSON.DataStores {
			ds := &language.DataStore{
				ID:    dsJSON.ID,
				Label: dsJSON.Label,
			}
			cont.Items = append(cont.Items, language.ContainerItem{DataStore: ds})
		}
	}

	if len(cJSON.Queues) > 0 {
		for _, qJSON := range cJSON.Queues {
			q := &language.Queue{
				ID:    qJSON.ID,
				Label: qJSON.Label,
			}
			cont.Items = append(cont.Items, language.ContainerItem{Queue: q})
		}
	}

	if len(cJSON.Relations) > 0 {
		for _, rJSON := range cJSON.Relations {
			if rJSON.From == "" || rJSON.To == "" {
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

	if len(compJSON.Relations) > 0 {
		for _, rJSON := range compJSON.Relations {
			if rJSON.From == "" || rJSON.To == "" {
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
			comp.Items = append(comp.Items, language.ComponentItem{Relation: rel})
		}
	}

	return comp
}

// convertRequirementJSONToAST converts a RequirementJSON to a Requirement AST
func convertRequirementJSONToAST(reqJSON *jsonexport.RequirementJSON) *language.Requirement {
	if reqJSON.ID == "" {
		reqJSON.ID = reqJSON.Title
	}

	req := &language.Requirement{
		ID:          reqJSON.ID,
		Type:        strPtr("functional"),
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
		Items:  []*language.ScenarioItem{},
	}
	return scen
}

// convertContractJSONToAST converts a ContractJSON to a Contract AST
func convertContractJSONToAST(contractJSON *jsonexport.ContractJSON) *language.Contract {
	contract := &language.Contract{
		Kind: "api",
		ID:   contractJSON.ID,
		L:    "{",
		R:    "}",
		Body: &language.ContractBody{},
	}
	return contract
}

// convertPolicyJSONToAST converts a PolicyJSON to a Policy AST
func convertPolicyJSONToAST(policyJSON *jsonexport.PolicyJSON) *language.Policy {
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

	if len(flowJSON.Steps) > 0 {
		for _, stepJSON := range flowJSON.Steps {
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

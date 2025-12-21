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

	totalItems := len(sJSON.Containers) + len(sJSON.DataStores) + len(sJSON.Queues) + len(sJSON.Relations)
	sys := &language.System{
		ID:          sJSON.ID,
		Label:       sJSON.Label,
		Description: sJSON.Description,
		Items:       make([]language.SystemItem, 0, totalItems),
	}

	for i := range sJSON.Containers {
		sys.Items = append(sys.Items, language.SystemItem{Container: convertContainerJSONToAST(&sJSON.Containers[i])})
	}

	for _, dsJSON := range sJSON.DataStores {
		sys.Items = append(sys.Items, language.SystemItem{DataStore: &language.DataStore{
			ID:    dsJSON.ID,
			Label: dsJSON.Label,
		}})
	}

	for _, qJSON := range sJSON.Queues {
		sys.Items = append(sys.Items, language.SystemItem{Queue: &language.Queue{
			ID:    qJSON.ID,
			Label: qJSON.Label,
		}})
	}

	for _, rJSON := range sJSON.Relations {
		if rel := convertRelationJSONToAST(&rJSON); rel != nil {
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

	totalItems := len(cJSON.Components) + len(cJSON.DataStores) + len(cJSON.Queues) + len(cJSON.Relations)
	cont := &language.Container{
		ID:          cJSON.ID,
		Label:       cJSON.Label,
		Description: cJSON.Description,
		Items:       make([]language.ContainerItem, 0, totalItems),
	}

	for i := range cJSON.Components {
		cont.Items = append(cont.Items, language.ContainerItem{Component: convertComponentJSONToAST(&cJSON.Components[i])})
	}

	for _, dsJSON := range cJSON.DataStores {
		cont.Items = append(cont.Items, language.ContainerItem{DataStore: &language.DataStore{
			ID:    dsJSON.ID,
			Label: dsJSON.Label,
		}})
	}

	for _, qJSON := range cJSON.Queues {
		cont.Items = append(cont.Items, language.ContainerItem{Queue: &language.Queue{
			ID:    qJSON.ID,
			Label: qJSON.Label,
		}})
	}

	for _, rJSON := range cJSON.Relations {
		if rel := convertRelationJSONToAST(&rJSON); rel != nil {
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
		Items:       make([]language.ComponentItem, 0, len(compJSON.Relations)),
	}

	for _, rJSON := range compJSON.Relations {
		if rel := convertRelationJSONToAST(&rJSON); rel != nil {
			comp.Items = append(comp.Items, language.ComponentItem{Relation: rel})
		}
	}

	return comp
}

func convertRelationJSONToAST(rJSON *jsonexport.RelationJSON) *language.Relation {
	if rJSON.From == "" || rJSON.To == "" {
		return nil
	}
	return &language.Relation{
		From:  stringToQualifiedIdent(rJSON.From),
		Arrow: "->",
		To:    stringToQualifiedIdent(rJSON.To),
		Verb:  rJSON.Verb,
		Label: rJSON.Label,
	}
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

	title := scenJSON.Label
	scen := &language.Scenario{
		ID:    scenJSON.ID,
		Title: &title,
		Items: []*language.ScenarioItem{},
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
	title := flowJSON.Title
	flow := &language.Flow{
		ID:          flowJSON.ID,
		Title:       &title,
		Description: flowJSON.Description,
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

func strPtr(s string) *string {
	return &s
}

package json

import (
	"testing"

	jsonexport "github.com/sruja-ai/sruja/pkg/export/json"
)

func TestConvertComponentJSONToAST_AllFields(t *testing.T) {
	compJSON := &jsonexport.ComponentJSON{
		ID:          "comp1",
		Label:       "Component",
		Description: mkStr("Component description"),
		Technology:  mkStr("Go"),
		Relations: []jsonexport.RelationJSON{
			{From: "A", To: "B", Verb: mkStr("calls"), Label: mkStr("API call")},
		},
	}

	result := convertComponentJSONToAST(compJSON)

	if result.ID != "comp1" {
		t.Errorf("expected ID comp1, got %s", result.ID)
	}
	if result.Label != "Component" {
		t.Errorf("expected Label Component, got %s", result.Label)
	}
	if result.Description == nil || *result.Description != "Component description" {
		t.Errorf("expected Description Component description, got %v", result.Description)
	}
	if len(result.Items) != 1 {
		t.Errorf("expected 1 item (relation), got %d", len(result.Items))
	}
}

func TestConvertSystemJSONToAST_AllFields(t *testing.T) {
	sysJSON := &jsonexport.SystemJSON{
		ID:    "sys1",
		Label: "System",
		Containers: []jsonexport.ContainerJSON{
			{ID: "cont1", Label: "Cont1"},
		},
		DataStores: []jsonexport.DataStoreJSON{
			{ID: "ds1", Label: "DS1"},
		},
		Queues: []jsonexport.QueueJSON{
			{ID: "q1", Label: "Q1"},
		},
		Relations: []jsonexport.RelationJSON{
			{From: "A", To: "B", Verb: mkStr("calls")},
		},
	}

	result := convertSystemJSONToAST(sysJSON)
	if result.ID != "sys1" {
		t.Errorf("expected sys1, got %s", result.ID)
	}
	// SysItems (Container + DataStore + Queue + Relation)
	if len(result.Items) != 4 {
		t.Errorf("expected 4 items, got %d", len(result.Items))
	}
}

func TestConvertContainerJSONToAST_AllFields(t *testing.T) {
	contJSON := &jsonexport.ContainerJSON{
		ID:    "cont1",
		Label: "Container",
		Components: []jsonexport.ComponentJSON{
			{ID: "comp1", Label: "Comp1"},
		},
		DataStores: []jsonexport.DataStoreJSON{
			{ID: "ds1", Label: "DS1"},
		},
		Queues: []jsonexport.QueueJSON{
			{ID: "q1", Label: "Q1"},
		},
		Relations: []jsonexport.RelationJSON{
			{From: "A", To: "B"},
		},
	}

	result := convertContainerJSONToAST(contJSON)
	if result.ID != "cont1" {
		t.Errorf("expected cont1, got %s", result.ID)
	}
	if len(result.Items) != 4 {
		t.Errorf("expected 4 items, got %d", len(result.Items))
	}
}

func TestConvertRequirementJSONToAST(t *testing.T) {
	reqJSON := &jsonexport.RequirementJSON{ID: "R1", Title: "Req1"}
	result := convertRequirementJSONToAST(reqJSON)
	if result.ID != "R1" {
		t.Error("expected R1")
	}
}

func TestConvertADRJSONToAST(t *testing.T) {
	adrJSON := &jsonexport.ADRJSON{ID: "A1", Title: "ADR1"}
	result := convertADRJSONToAST(adrJSON)
	if result.ID != "A1" {
		t.Error("expected A1")
	}
}

func TestConvertScenarioJSONToAST(t *testing.T) {
	scenJSON := &jsonexport.ScenarioJSON{ID: "S1", Label: "Scen1"}
	result := convertScenarioJSONToAST(scenJSON)
	if result.ID != "S1" {
		t.Error("expected S1")
	}
}

func TestConvertContractJSONToAST(t *testing.T) {
	cJSON := &jsonexport.ContractJSON{ID: "C1"}
	result := convertContractJSONToAST(cJSON)
	if result.ID != "C1" {
		t.Error("expected C1")
	}
}

func TestConvertPolicyJSONToAST(t *testing.T) {
	pJSON := &jsonexport.PolicyJSON{ID: "P1", Label: "Pol1"}
	result := convertPolicyJSONToAST(pJSON)
	if result.ID != "P1" {
		t.Error("expected P1")
	}
}

func TestConvertFlowJSONToAST(t *testing.T) {
	fJSON := &jsonexport.FlowJSON{
		ID:    "F1",
		Title: "Flow1",
		Steps: []jsonexport.ScenarioStepJSON{
			{From: "A", To: "B", Description: mkStr("step")},
		},
	}
	result := convertFlowJSONToAST(fJSON)
	if result.ID != "F1" {
		t.Error("expected F1")
	}
	if len(result.Items) != 1 {
		t.Error("expected 1 step")
	}
}

func TestConvertDeploymentNodeJSONToAST(t *testing.T) {
	dJSON := &jsonexport.DeploymentNodeJSON{ID: "D1", Label: "Dep1"}
	result := convertDeploymentNodeJSONToAST(dJSON)
	if result.ID != "D1" {
		t.Error("expected D1")
	}
}

func TestStringToQualifiedIdent(t *testing.T) {
	qid := stringToQualifiedIdent("A.B.C")
	if len(qid.Parts) != 3 || qid.String() != "A.B.C" {
		t.Errorf("expected A.B.C, got %v", qid.Parts)
	}

	qid = stringToQualifiedIdent("")
	if len(qid.Parts) != 1 || qid.Parts[0] != "" {
		t.Errorf("expected empty part, got %v", qid.Parts)
	}
}

func TestIntPtrToStringPtr(t *testing.T) {
	i := 10
	s := intPtrToStringPtr(&i)
	if s == nil || *s != "10" {
		t.Errorf("expected 10, got %v", s)
	}
	if intPtrToStringPtr(nil) != nil {
		t.Error("expected nil")
	}
}

func TestConverter_Methods(t *testing.T) {
	c := NewConverter()
	_, err := c.ToArchitecture(nil)
	if err == nil {
		t.Error("expected error for ToArchitecture")
	}
	_, err = c.ToDSL(nil, OutputFormatSingleFile)
	if err == nil {
		t.Error("expected error for ToDSL")
	}
}

package json

import (
    "testing"
    "github.com/sruja-ai/sruja/pkg/language"
)

func TestConvertArchitecture_RichFeatures(t *testing.T) {
    // Overview via entries with PostProcess
    ov := &language.OverviewBlock{Entries: []*language.MetaEntry{{Key: "summary", Value: mkStr2("Summary")}, {Key: "goals", Array: []string{"g1", "g2"}}}}
    ov.PostProcess()

    // Policies
    pol := &language.Policy{ID: "P1", Description: "Desc", Category: mkStr2("cat"), Enforcement: mkStr2("must")}

    // Constraints and Conventions
    cons := []*language.ConstraintEntry{{Key: "latency", Value: "100ms"}}
    conv := []*language.ConventionEntry{{Key: "naming", Value: "snake_case"}}

    // Shared artifact and library with items
    sa := &language.SharedArtifact{ID: "SA", Label: "Shared"}
    lib := &language.Library{ID: "L", Label: "Lib", Items: []*language.LibraryItem{{Description: mkStr2("Use")}}}

    // Contracts
    reqType := &language.TypeSpec{Name: "string"}
    respType := &language.TypeSpec{Name: "int", Optional: "?"}
    req := &language.SchemaBlock{Entries: []*language.SchemaEntry{{Key: "customerId", Type: reqType}}}
    resp := &language.SchemaBlock{Entries: []*language.SchemaEntry{{Key: "status", Type: respType}}}
    cb := &language.ContractBody{Version: mkStr2("1.0"), Endpoint: mkStr2("/x"), Method: mkStr2("POST"), Request: req, Response: resp, Errors: []string{"E1"}, Retention: mkStr2("30d")}
    ctr := &language.Contract{Kind: "api", ID: "Create", Body: cb}

    // Scenario
    step := &language.ScenarioStep{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}, Description: mkStr2("d")}
    sc := &language.Scenario{ID: "Sc1", Title: "Scenario", Steps: []*language.ScenarioStep{step}}

    // Relations
    verb := "calls"
    rel := &language.Relation{From: language.QualifiedIdent{Parts: []string{"A"}}, To: language.QualifiedIdent{Parts: []string{"B"}}, Verb: &verb}

    // Person, datastore, queue
    person := &language.Person{ID: "U", Label: "User"}
    ds := &language.DataStore{ID: "DB", Label: "Database"}
    q := &language.Queue{ID: "Q", Label: "Queue"}
    arch := &language.Architecture{
        Name:         "Ar",
        Overview:     ov,
        Policies:     []*language.Policy{pol},
        Constraints:  cons,
        Conventions:  conv,
        SharedArtifacts: []*language.SharedArtifact{sa},
        Libraries:    []*language.Library{lib},
        Contracts:    []*language.Contract{ctr},
        Scenarios:     []*language.Scenario{sc},
        Relations:    []*language.Relation{rel},
        Properties:   map[string]string{"qps": "100"},
        Style:        map[string]string{"color": "#fff"},
        Persons:      []*language.Person{person},
        DataStores:   []*language.DataStore{ds},
        Queues:       []*language.Queue{q},
    }

    body := convertArchitectureToJSON(arch)
    if body.Overview == nil || body.Overview.Summary != "Summary" || len(body.Overview.Goals) != 2 { t.Fatalf("overview conversion failed: %+v", body.Overview) }
    if len(body.Policies) != 1 || body.Policies[0].Category != "cat" || body.Policies[0].Enforcement != "must" { t.Fatalf("policy conversion failed: %+v", body.Policies) }
    if len(body.Contracts) != 1 || body.Contracts[0].Body == nil || body.Contracts[0].Body.Request == nil || body.Contracts[0].Body.Response == nil { t.Fatalf("contracts conversion failed: %+v", body.Contracts) }
    if len(body.Scenarios) != 1 || len(body.Scenarios[0].Steps) != 1 { t.Fatalf("scenario conversion failed: %+v", body.Scenarios) }
    if len(body.Relations) != 1 || (body.Relations[0].Verb == nil || *body.Relations[0].Verb != "calls") { t.Fatalf("relation conversion failed: %+v", body.Relations) }
    if body.Properties["qps"] != "100" || body.Style["color"] != "#fff" { t.Fatalf("properties/style conversion failed: %+v %+v", body.Properties, body.Style) }
    if len(body.SharedArtifacts) != 1 || len(body.Libraries) != 1 { t.Fatalf("shared/library conversion failed: %d %d", len(body.SharedArtifacts), len(body.Libraries)) }
    // Basic presence checks (non-fatal for optional top-level items)
}

func mkStr2(s string) *string { return &s }

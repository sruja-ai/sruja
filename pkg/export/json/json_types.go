package json

import "time"

type ArchitectureJSON struct {
    Metadata    MetadataJSON     `json:"metadata"`
    Architecture ArchitectureBody `json:"architecture"`
    Navigation  NavigationJSON   `json:"navigation"`
}

type MetadataJSON struct {
    Name      string `json:"name"`
    Version   string `json:"version"`
    Generated string `json:"generated"`
}

type ArchitectureBody struct {
    Imports        []ImportJSON    `json:"imports,omitempty"`
    Systems        []SystemJSON    `json:"systems,omitempty"`
    Persons        []PersonJSON    `json:"persons,omitempty"`
    Relations      []RelationJSON  `json:"relations,omitempty"`
    Containers     []ContainerJSON `json:"containers,omitempty"`
    Components     []ComponentJSON `json:"components,omitempty"`
    DataStores     []DataStoreJSON `json:"datastores,omitempty"`
    Queues         []QueueJSON     `json:"queues,omitempty"`
    Domains        []DomainJSON    `json:"domains,omitempty"`
    Contexts       []ContextJSON   `json:"contexts,omitempty"`
    Aggregates     []AggregateJSON `json:"aggregates,omitempty"`
    Scenarios      []ScenarioJSON  `json:"scenarios,omitempty"`
    Flows          []FlowJSON      `json:"flows,omitempty"`
    Requirements   []RequirementJSON `json:"requirements,omitempty"`
    ADRs           []ADRJSON         `json:"adrs,omitempty"`
    Deployment     []DeploymentNodeJSON `json:"deployment,omitempty"`
    Contracts      []ContractJSON      `json:"contracts,omitempty"`
    SharedArtifacts []SharedArtifactJSON `json:"sharedArtifacts,omitempty"`
    Libraries      []LibraryJSON       `json:"libraries,omitempty"`
    Policies       []PolicyJSON        `json:"policies,omitempty"`
    Constraints    []ConstraintJSON    `json:"constraints,omitempty"`
    Conventions    []ConventionJSON    `json:"conventions,omitempty"`
    Views          []ViewJSON          `json:"views,omitempty"`
}

type NavigationJSON struct {
    Levels   []string      `json:"levels,omitempty"`
    Scenarios []ScenarioNav `json:"scenarios,omitempty"`
    Flows     []FlowNav     `json:"flows,omitempty"`
    Domains   []DomainNav   `json:"domains,omitempty"`
}

type ImportJSON struct {
    Path  string  `json:"path"`
    Alias *string `json:"alias,omitempty"`
}

type SystemJSON struct {
    ID          string           `json:"id"`
    Label       string           `json:"label,omitempty"`
    Description *string          `json:"description,omitempty"`
    Containers  []ContainerJSON  `json:"containers,omitempty"`
    Components  []ComponentJSON  `json:"components,omitempty"`
    DataStores  []DataStoreJSON  `json:"datastores,omitempty"`
    Queues      []QueueJSON      `json:"queues,omitempty"`
    Relations   []RelationJSON   `json:"relations,omitempty"`
}

type ContainerJSON struct {
    ID          string          `json:"id"`
    Label       string          `json:"label,omitempty"`
    Description *string         `json:"description,omitempty"`
    Components  []ComponentJSON `json:"components,omitempty"`
    DataStores  []DataStoreJSON `json:"datastores,omitempty"`
    Queues      []QueueJSON     `json:"queues,omitempty"`
    Relations   []RelationJSON  `json:"relations,omitempty"`
}

type ComponentJSON struct {
    ID          string         `json:"id"`
    Label       string         `json:"label,omitempty"`
    Description *string        `json:"description,omitempty"`
    Relations   []RelationJSON `json:"relations,omitempty"`
}

type DataStoreJSON struct {
    ID    string `json:"id"`
    Label string `json:"label,omitempty"`
}

type QueueJSON struct {
    ID    string `json:"id"`
    Label string `json:"label,omitempty"`
}

type PersonJSON struct {
    ID          string  `json:"id"`
    Label       string  `json:"label,omitempty"`
    Description *string `json:"description,omitempty"`
}

type RelationJSON struct {
    From  string  `json:"from"`
    To    string  `json:"to"`
    Verb  *string `json:"verb,omitempty"`
    Label *string `json:"label,omitempty"`
}

// Placeholder types for extended sections; filled incrementally as needed
type DomainJSON struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type ContextJSON struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type AggregateJSON struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type ScenarioJSON struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type FlowJSON struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type RequirementJSON struct{ ID string `json:"id"`; Title string `json:"title,omitempty"` }
type ADRJSON struct{ ID string `json:"id"`; Title string `json:"title,omitempty"` }
type DeploymentNodeJSON struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type ContractJSON struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type SharedArtifactJSON struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type LibraryJSON struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type PolicyJSON struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type ConstraintJSON struct{ Key string `json:"key"`; Value string `json:"value"` }
type ConventionJSON struct{ Key string `json:"key"`; Value string `json:"value"` }
type ViewJSON struct{ Name string `json:"name"`; Type string `json:"type"` }

type ScenarioNav struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type FlowNav struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }
type DomainNav struct{ ID string `json:"id"`; Label string `json:"label,omitempty"` }

func NewMetadata(name string) MetadataJSON {
    return MetadataJSON{ Name: name, Version: "1.0.0", Generated: time.Now().Format(time.RFC3339) }
}


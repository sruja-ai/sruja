//nolint:gocritic // rangeValCopy, ifElseChain acceptable here
package converter

// JSON Structures matching viewer expectations

type ArchitectureJSON struct {
	Metadata     MetadataJSON `json:"metadata"`
	Architecture Architecture `json:"architecture"`
}

type MetadataJSON struct {
	Name      string `json:"name"`
	Version   string `json:"version"`
	Generated string `json:"generated"`
}

type MetadataEntryJSON struct {
	Key   string   `json:"key"`
	Value *string  `json:"value,omitempty"`
	Array []string `json:"array,omitempty"`
}

type Architecture struct {
	Name         string              `json:"name,omitempty"`
	ArchMetadata []MetadataEntryJSON `json:"archMetadata,omitempty"`
	Systems      []System            `json:"systems,omitempty"`
	Persons      []Person            `json:"persons,omitempty"`
	Containers   []Container         `json:"containers,omitempty"` // Top-level containers
	Relations    []Relation          `json:"relations,omitempty"`
	ADRs         []ADR               `json:"adrs,omitempty"`
	Requirements []Requirement       `json:"requirements,omitempty"`
	Flows        []Flow              `json:"flows,omitempty"`
	Policies     []Policy            `json:"policies,omitempty"`
}

type System struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Containers  []Container         `json:"containers,omitempty"`
	DataStores  []DataStore         `json:"datastores,omitempty"`
	Queues      []Queue             `json:"queues,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type Container struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Technology  string              `json:"technology,omitempty"`
	Components  []Component         `json:"components,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type Component struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Technology  string              `json:"technology,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type DataStore struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Technology  string              `json:"technology,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type Queue struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Technology  string              `json:"technology,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type Person struct {
	ID          string              `json:"id"`
	Label       string              `json:"label"`
	Description string              `json:"description,omitempty"`
	Metadata    []MetadataEntryJSON `json:"metadata,omitempty"`
}

type Relation struct {
	From  string   `json:"from"`
	To    string   `json:"to"`
	Tags  []string `json:"tags,omitempty"`
	Verb  string   `json:"verb,omitempty"`
	Label string   `json:"label,omitempty"`
}

type ADR struct {
	ID           string `json:"id"`
	Title        string `json:"title"`
	Status       string `json:"status,omitempty"`
	Context      string `json:"context,omitempty"`
	Decision     string `json:"decision,omitempty"`
	Consequences string `json:"consequences,omitempty"`
}

type Requirement struct {
	ID          string   `json:"id"`
	Type        string   `json:"type,omitempty"`
	Title       string   `json:"title,omitempty"`
	Description string   `json:"description,omitempty"`
	Tags        []string `json:"tags,omitempty"`
}

type Flow struct {
	ID          string     `json:"id"`
	Title       string     `json:"title,omitempty"`
	Label       string     `json:"label,omitempty"`
	Description string     `json:"description,omitempty"`
	Steps       []FlowStep `json:"steps,omitempty"`
}

type FlowStep struct {
	From        string `json:"from"`
	To          string `json:"to"`
	Description string `json:"description,omitempty"`
}

type Policy struct {
	ID          string   `json:"id"`
	Description string   `json:"description,omitempty"`
	Category    string   `json:"category,omitempty"`
	Enforcement string   `json:"enforcement,omitempty"`
	Tags        []string `json:"tags,omitempty"`
}

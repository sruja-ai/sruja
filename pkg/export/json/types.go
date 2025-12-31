package json

// JSON model format with Sruja extensions

// SrujaModelDump is the root JSON structure for Sruja model data
type SrujaModelDump struct {
	// Required fields
	Stage         string                   `json:"_stage,omitempty"` // "parsed" | "computed" | "layouted", defaults to "layouted"
	ProjectID     string                   `json:"projectId,omitempty"`
	Project       *ProjectDump             `json:"project,omitempty"`
	Globals       *GlobalsDump             `json:"globals,omitempty"`
	Imports       map[string][]ElementDump `json:"imports,omitempty"`
	Deployments   *DeploymentsDump         `json:"deployments,omitempty"`
	Specification SpecificationDump        `json:"specification"`
	Elements      map[string]ElementDump   `json:"elements"`
	Relations     []RelationDump           `json:"relations,omitempty"`
	Views         map[string]ViewDump      `json:"views,omitempty"`

	// Sruja governance extensions
	Sruja *SrujaExtensions `json:"sruja,omitempty"`

	// Metadata (Sruja-specific)
	Metadata ModelMetadata `json:"_metadata,omitempty"`
}

type ProjectDump struct {
	ID   string `json:"id"`
	Name string `json:"name"`
}

type GlobalsDump struct {
	Predicates        map[string]interface{} `json:"predicates,omitempty"`
	DynamicPredicates map[string]interface{} `json:"dynamicPredicates,omitempty"`
	Styles            map[string]interface{} `json:"styles,omitempty"`
}

type DeploymentsDump struct {
	Elements  map[string]interface{} `json:"elements,omitempty"`
	Relations map[string]interface{} `json:"relations,omitempty"`
}

type ModelMetadata struct {
	Name       string                `json:"name"`
	Version    string                `json:"version"`
	Generated  string                `json:"generated"`
	SrujaVer   string                `json:"srujaVersion"`
	LayoutData map[string]LayoutData `json:"layout,omitempty"`
}

// Note: LayoutData is defined in json_types.go

// SpecificationDump defines element kinds, relationship kinds, tags
type SpecificationDump struct {
	Elements      map[string]ElementKindDump      `json:"elements"`
	Relationships map[string]RelationshipKindDump `json:"relationships,omitempty"`
	Tags          map[string]TagDump              `json:"tags,omitempty"`
	Colors        map[string]string               `json:"customColors,omitempty"`
	Project       *ProjectDump                    `json:"project,omitempty"` // Project info for import reference checking
}

type ElementKindDump struct {
	Title       string     `json:"title,omitempty"`
	Description string     `json:"description,omitempty"`
	Technology  string     `json:"technology,omitempty"`
	Style       *StyleDump `json:"style,omitempty"`
}

type RelationshipKindDump struct {
	Title      string `json:"title,omitempty"`
	Technology string `json:"technology,omitempty"`
}

type TagDump struct {
	Color string `json:"color,omitempty"`
}

// ElementDump represents a model element
type ElementDump struct {
	ID          string            `json:"id"`   // FQN: "System.Container.Component"
	Kind        string            `json:"kind"` // "person", "system", "container", etc.
	Title       string            `json:"title"`
	Description string            `json:"description,omitempty"`
	Technology  string            `json:"technology,omitempty"`
	Tags        []string          `json:"tags,omitempty"`
	Links       []LinkDump        `json:"links,omitempty"`
	Metadata    map[string]string `json:"metadata,omitempty"`
	Style       *StyleDump        `json:"style,omitempty"`
	Parent      string            `json:"parent,omitempty"` // Parent FQN
}

type LinkDump struct {
	URL   string `json:"url"`
	Title string `json:"title,omitempty"`
}

type StyleDump struct {
	Shape   string `json:"shape,omitempty"`
	Color   string `json:"color,omitempty"`
	Icon    string `json:"icon,omitempty"`
	Border  string `json:"border,omitempty"`
	Opacity int    `json:"opacity,omitempty"`
}

// FqnRefDump represents a Fully Qualified Name Reference
// References are objects with a 'model' property, not plain strings
type FqnRefDump struct {
	Model string `json:"model"` // The FQN string (e.g., "system.container.component")
}

// NewFqnRef creates a FqnRefDump from an FQN string
func NewFqnRef(fqn string) FqnRefDump {
	return FqnRefDump{Model: fqn}
}

// RelationDump represents a relationship
// Source and Target must be FqnRef objects, not plain strings
type RelationDump struct {
	ID          string            `json:"id"`
	Source      FqnRefDump        `json:"source"` // FqnRef with 'model' field
	Target      FqnRefDump        `json:"target"` // FqnRef with 'model' field
	Title       string            `json:"title,omitempty"`
	Description string            `json:"description,omitempty"`
	Technology  string            `json:"technology,omitempty"`
	Kind        string            `json:"kind,omitempty"`
	Tags        []string          `json:"tags,omitempty"`
	Metadata    map[string]string `json:"metadata,omitempty"`
	// Styling
	Color string `json:"color,omitempty"`
	Line  string `json:"line,omitempty"` // "solid", "dashed", "dotted"
	Head  string `json:"head,omitempty"` // arrow type
	Tail  string `json:"tail,omitempty"` // arrow type
}

// ViewDump represents a view
type ViewDump struct {
	ID          string     `json:"id"`
	Title       string     `json:"title,omitempty"`
	Description string     `json:"description,omitempty"`
	ViewOf      string     `json:"viewOf,omitempty"` // FQN of scoped element
	Tags        []string   `json:"tags,omitempty"`
	Rules       []ViewRule `json:"rules,omitempty"`
	// Computed nodes and edges (for rendering)
	Nodes []NodeDump `json:"nodes"`
	Edges []EdgeDump `json:"edges"`
	// Layout positions for manual editing (Sruja extension)
	Layout *ViewLayoutDump `json:"layout,omitempty"`
}

// ViewLayoutDump represents layout configuration for a view
type ViewLayoutDump struct {
	Positions map[string]ViewPositionDump `json:"positions,omitempty"`
}

// ViewPositionDump represents a position hint for an element
type ViewPositionDump struct {
	X float64 `json:"x"`
	Y float64 `json:"y"`
}

type ViewRule struct {
	Include *ViewRuleExpr `json:"include,omitempty"`
	Exclude *ViewRuleExpr `json:"exclude,omitempty"`
}

type ViewRuleExpr struct {
	Wildcard  bool     `json:"wildcard,omitempty"`
	Recursive bool     `json:"recursive,omitempty"`
	Elements  []string `json:"elements,omitempty"`
}

type NodeDump struct {
	ID      string  `json:"id"`
	Element string  `json:"element"` // FQN reference
	Parent  string  `json:"parent,omitempty"`
	Title   string  `json:"title,omitempty"`
	X       float64 `json:"x"`
	Y       float64 `json:"y"`
	Width   float64 `json:"width"`
	Height  float64 `json:"height"`
}

type EdgeDump struct {
	ID       string `json:"id"`
	Source   string `json:"source"`
	Target   string `json:"target"`
	Relation string `json:"relation,omitempty"` // Relation ID reference
	Title    string `json:"title,omitempty"`
}

// ===========================================================================
// Sruja Extensions (Governance Layer)
// ===========================================================================

type SrujaExtensions struct {
	Requirements []RequirementDump `json:"requirements,omitempty"`
	ADRs         []ADRDump         `json:"adrs,omitempty"`
	Policies     []PolicyDump      `json:"policies,omitempty"`
	Constraints  []ConstraintDump  `json:"constraints,omitempty"`
	Conventions  []ConventionDump  `json:"conventions,omitempty"`
	Scenarios    []ScenarioDump    `json:"scenarios,omitempty"`
	Flows        []FlowDump        `json:"flows,omitempty"`
	Deployments  []DeploymentDump  `json:"deployments,omitempty"`
	Imports      []ImportDump      `json:"imports,omitempty"`
	SLOs         []SLODump         `json:"slos,omitempty"`
}

type RequirementDump struct {
	ID          string   `json:"id"`
	Title       string   `json:"title"`
	Type        string   `json:"type,omitempty"`
	Description string   `json:"description,omitempty"`
	Priority    string   `json:"priority,omitempty"`
	Status      string   `json:"status,omitempty"`
	Elements    []string `json:"elements,omitempty"` // FQN references
}

type ADRDump struct {
	ID           string `json:"id"`
	Title        string `json:"title"`
	Status       string `json:"status,omitempty"`
	Context      string `json:"context,omitempty"`
	Decision     string `json:"decision,omitempty"`
	Consequences string `json:"consequences,omitempty"`
	Date         string `json:"date,omitempty"`
	Author       string `json:"author,omitempty"`
}

type PolicyDump struct {
	ID          string   `json:"id"`
	Title       string   `json:"title"`
	Category    string   `json:"category,omitempty"`
	Enforcement string   `json:"enforcement,omitempty"`
	Description string   `json:"description,omitempty"`
	Elements    []string `json:"elements,omitempty"`
}

type ConstraintDump struct {
	ID          string `json:"id"`
	Description string `json:"description"`
	Type        string `json:"type,omitempty"`
}

type ConventionDump struct {
	ID          string `json:"id"`
	Description string `json:"description"`
}

type ScenarioDump struct {
	ID          string     `json:"id"`
	Title       string     `json:"title"`
	Description string     `json:"description,omitempty"`
	Steps       []StepDump `json:"steps,omitempty"`
}

type StepDump struct {
	ID          string `json:"id,omitempty"`
	Description string `json:"description"`
	From        string `json:"from,omitempty"`
	To          string `json:"to,omitempty"`
}

type FlowDump struct {
	ID          string     `json:"id"`
	Title       string     `json:"title"`
	Description string     `json:"description,omitempty"`
	Steps       []StepDump `json:"steps,omitempty"`
}

type DeploymentDump struct {
	ID          string           `json:"id"`
	Kind        string           `json:"kind,omitempty"` // "node", "region", "zone", etc.
	Title       string           `json:"title,omitempty"`
	Description string           `json:"description,omitempty"`
	Technology  string           `json:"technology,omitempty"`
	Children    []DeploymentDump `json:"children,omitempty"`
	Instances   []string         `json:"instances,omitempty"` // Element FQNs deployed here
}

type ImportDump struct {
	Elements []string `json:"elements"`
	From     string   `json:"from"`
}

type SLODump struct {
	ID                string  `json:"id"`
	Target            float64 `json:"target,omitempty"`
	Availability      float64 `json:"availability,omitempty"`
	Latency           string  `json:"latency,omitempty"`
	Throughput        string  `json:"throughput,omitempty"`
	ErrorBudget       float64 `json:"errorBudget,omitempty"`
	RecoveryObjective string  `json:"recoveryObjective,omitempty"`
	RecoveryPoint     string  `json:"recoveryPoint,omitempty"`
}

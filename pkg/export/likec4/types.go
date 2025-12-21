// Package likec4 exports Sruja architecture to LikeC4 JSON format.
//
// LikeC4 is a DSL for software architecture diagrams that provides
// interactive visualization. This exporter converts Sruja models to
// LikeC4's JSON format for rendering with @likec4/diagram.
//
// See: https://likec4.dev/tooling/model-api/
package likec4

// Model represents a LikeC4 model with elements, relations, and views.
type Model struct {
	Elements  []Element  `json:"elements"`
	Relations []Relation `json:"relations"`
	Views     []View     `json:"views,omitempty"`
}

// Element represents a node in the LikeC4 model.
type Element struct {
	// ID is the unique identifier (e.g., "system.container")
	ID string `json:"id"`

	// Kind is the element type (e.g., "system", "container", "component", "person")
	Kind string `json:"kind"`

	// Title is the display name
	Title string `json:"title,omitempty"`

	// Description is a longer explanation (supports Markdown)
	Description string `json:"description,omitempty"`

	// Technology specifies the tech stack (e.g., "Go", "PostgreSQL")
	Technology string `json:"technology,omitempty"`

	// Tags for categorization
	Tags []string `json:"tags,omitempty"`

	// Metadata is custom key-value data (policies, SLOs, etc.)
	Metadata map[string]interface{} `json:"metadata,omitempty"`

	// Parent is the parent element ID for nested elements
	Parent string `json:"parent,omitempty"`
}

// Relation represents a connection between elements.
type Relation struct {
	// Source is the origin element ID
	Source string `json:"source"`

	// Target is the destination element ID
	Target string `json:"target"`

	// Title is the relationship label (e.g., "sends data to")
	Title string `json:"title,omitempty"`

	// Kind is an optional relationship type (e.g., "async", "uses")
	Kind string `json:"kind,omitempty"`

	// Technology specifies the protocol (e.g., "gRPC", "REST")
	Technology string `json:"technology,omitempty"`

	// Tags for categorization
	Tags []string `json:"tags,omitempty"`

	// Metadata for custom data
	Metadata map[string]interface{} `json:"metadata,omitempty"`
}

// View represents a diagram perspective.
type View struct {
	// ID is the unique view identifier
	ID string `json:"id"`

	// Title is the display name
	Title string `json:"title,omitempty"`

	// Description explains what the view shows
	Description string `json:"description,omitempty"`

	// Scope is the root element for scoped views
	Scope string `json:"scope,omitempty"`

	// Include contains predicates for included elements
	Include []string `json:"include,omitempty"`

	// Exclude contains predicates for excluded elements
	Exclude []string `json:"exclude,omitempty"`
}

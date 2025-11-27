// Package model provides the architecture model structure and JSON serialization.
//
// The model package defines the canonical representation of architecture that is:
//   - Language-agnostic (not tied to DSL syntax)
//   - JSON-serializable (for storage, MCP, etc.)
//   - Extensible (can add new fields without breaking compatibility)
//
// The model is the intermediate representation between:
//   - AST (language-specific parsed structure)
//   - Diagram formats (Mermaid, D2, PlantUML)
//
// Example usage:
//
//	// Transform AST to Model
//	transformer := compiler.NewTransformer()
//	model, _ := transformer.Transform(program)
//
//	// Serialize to JSON
//	jsonData, _ := model.ToJSON()
//
//	// Compile to Mermaid
//	mermaidCompiler := compiler.NewMermaidCompiler()
//	mermaid, _ := mermaidCompiler.Compile(program)
package model

import (
	"encoding/json"
	"time"
)

// Model represents the complete architecture model.
//
// This is the canonical representation of architecture that can be:
//   - Serialized to JSON (for storage, APIs, MCP)
//   - Compiled to diagrams (Mermaid, D2, PlantUML)
//   - Validated (semantic rules, architecture rules)
//   - Queried (find elements, relationships, etc.)
//
// The model is versioned to support evolution over time.
type Model struct {
	Version      string        `json:"version"`                // Model version (e.g., "1.0")
	GeneratedAt  time.Time     `json:"generatedAt"`            // When this model was generated
	Architecture *Architecture `json:"architecture,omitempty"` // Preferred root container name
}

// Workspace represents the workspace in the model.
//
// A workspace contains all architecture definitions for a project:
//   - Elements: All architecture elements (systems, containers, components, etc.)
//   - Relations: All relationships between elements
//   - Requirements: Functional and non-functional requirements
//   - ADRs: Architecture Decision Records
//   - Journeys: User journeys describing flows across architecture elements
//
// Elements and Relations are flattened (not hierarchical) for easier querying and validation.
// Architecture represents the architecture in the model (preferred name)
type Architecture struct {
	Name         string        `json:"name,omitempty"`
	Description  string        `json:"description,omitempty"`
	Elements     []Element     `json:"elements"`               // All architecture elements (flattened)
	Relations    []Relation    `json:"relations"`              // All relationships between elements
	Requirements []Requirement `json:"requirements,omitempty"` // Requirements (optional)
	ADRs         []ADR         `json:"adrs,omitempty"`         // ADRs (optional)
	Journeys     []Journey     `json:"journeys,omitempty"`     // User journeys (optional)
}

// Removed legacy Workspace alias; Architecture is the canonical root.

// ElementType represents the type of an architecture element.
//
// These types correspond to the C4 model hierarchy:
//   - Person: Human users of the system
//   - System: High-level software systems
//   - Container: Applications or data stores within a system
//   - Component: Logical modules within a container
//   - DataStore: Databases, file systems, etc.
//   - Queue: Message queues, event streams, etc.
//   - ExternalService: External systems (third-party APIs, etc.)
type ElementType string

const (
	ElementTypePerson          ElementType = "Person"          // Human users
	ElementTypeSystem          ElementType = "System"          // Software systems
	ElementTypeContainer       ElementType = "Container"       // Applications, databases
	ElementTypeComponent       ElementType = "Component"       // Logical modules
	ElementTypeDataStore       ElementType = "DataStore"       // Databases, file systems
	ElementTypeQueue           ElementType = "Queue"           // Message queues, event streams
	ElementTypeExternalService ElementType = "ExternalService" // External systems
)

// Element represents an architecture element.
//
// Elements are flattened in the model (not hierarchical) for easier:
//   - Querying (find all elements of a type)
//   - Validation (check all elements for rules)
//   - Diagram generation (iterate over all elements)
//
// The hierarchical structure (system contains containers, containers contain components)
// is preserved through relationships and metadata if needed.
type Element struct {
	Type        ElementType `json:"type"`                  // Element type (System, Container, etc.)
	ID          string      `json:"id"`                    // Unique identifier
	Name        string      `json:"name"`                  // Human-readable name
	Description string      `json:"description,omitempty"` // Optional description
	Technology  string      `json:"technology,omitempty"`  // Technology stack (optional)
	Tags        []string    `json:"tags,omitempty"`        // Tags for categorization (optional)
	Metadata    Metadata    `json:"metadata,omitempty"`    // Additional metadata (optional)
	Location    Location    `json:"location,omitempty"`    // Source location (optional, for error reporting)
}

// Location represents the source location of an element.
//
// Used for error reporting and IDE features (go-to-definition, hover, etc.).
// All positions are 1-based (line 1, column 1 is the first character).
type Location struct {
	File   string `json:"file"`   // File path
	Line   int    `json:"line"`   // 1-based line number
	Column int    `json:"column"` // 1-based column number
}

// Metadata represents additional metadata for an element.
//
// This is a flexible key-value store for extensibility. Can be used for:
//   - Custom properties
//   - Extension-specific data (resilience, performance, security, etc.)
//   - Tool-specific annotations
type Metadata map[string]interface{}

// RelationType represents the type of a relationship.
//
// Common relationship types:
//   - Uses: Element uses another element (general dependency)
//   - DependsOn: Element depends on another (stronger than Uses)
//   - Publishes: Element publishes events to another
//   - Subscribes: Element subscribes to events from another
//   - Reads: Element reads from another (e.g., database reads)
//   - Writes: Element writes to another (e.g., database writes)
type RelationType string

const (
	RelationTypeUses       RelationType = "Uses"       // General usage
	RelationTypeDepends    RelationType = "DependsOn"  // Dependency
	RelationTypePublishes  RelationType = "Publishes"  // Event publishing
	RelationTypeSubscribes RelationType = "Subscribes" // Event subscription
	RelationTypeReads      RelationType = "Reads"      // Read access
	RelationTypeWrites     RelationType = "Writes"     // Write access
)

// Relation represents a relationship between elements.
//
// Relations are directed (from → to) and describe how elements interact.
// All relations are flattened in the model for easier querying and validation.
type Relation struct {
	From        string       `json:"from"`                  // Source element ID
	To          string       `json:"to"`                    // Target element ID
	Type        RelationType `json:"type"`                  // Relationship type
	Description string       `json:"description,omitempty"` // Optional description
	Location    Location     `json:"location,omitempty"`    // Source location (optional)
}

// RequirementType represents the type of a requirement.
//
// Types:
//   - functional: What the system must do
//   - constraint: Constraints on the system
//   - performance: Performance requirements
//   - security: Security requirements
type RequirementType string

const (
	RequirementTypeFunctional  RequirementType = "functional"  // Functional requirements
	RequirementTypeConstraint  RequirementType = "constraint"  // Constraints
	RequirementTypePerformance RequirementType = "performance" // Performance requirements
	RequirementTypeSecurity    RequirementType = "security"    // Security requirements
)

// Requirement represents a requirement.
//
// Requirements describe what the system must do or constraints it must satisfy.
// The Implements field links requirements to architecture elements for traceability.
type Requirement struct {
	ID          string          `json:"id"`                   // Requirement ID (e.g., "R1", "R2")
	Type        RequirementType `json:"type"`                 // Requirement type
	Description string          `json:"description"`          // Requirement description
	Implements  []string        `json:"implements,omitempty"` // Element IDs that implement this requirement
	Location    Location        `json:"location,omitempty"`   // Source location (optional)
}

// ADRStatus represents the status of an ADR.
//
// Status lifecycle:
//   - proposed: Decision is proposed but not yet accepted
//   - accepted: Decision has been accepted and is in effect
//   - rejected: Decision was considered but rejected
//   - deprecated: Decision was accepted but is now deprecated (superseded)
type ADRStatus string

const (
	ADRStatusProposed   ADRStatus = "proposed"   // Proposed decision
	ADRStatusAccepted   ADRStatus = "accepted"   // Accepted decision
	ADRStatusRejected   ADRStatus = "rejected"   // Rejected decision
	ADRStatusDeprecated ADRStatus = "deprecated" // Deprecated decision
)

// ADR represents an Architecture Decision Record.
//
// ADRs document important architectural decisions, including context, decision,
// and consequences. This helps teams understand why certain choices were made.
type ADR struct {
	ID           string    `json:"id"`                     // ADR ID (e.g., "ADR001")
	Title        string    `json:"title,omitempty"`        // Optional title
	Context      string    `json:"context,omitempty"`      // Why this decision was needed
	Decision     string    `json:"decision"`               // What was decided
	Consequences []string  `json:"consequences,omitempty"` // Implications of the decision
	Status       ADRStatus `json:"status,omitempty"`       // Current status
	Location     Location  `json:"location,omitempty"`     // Source location (optional)
}

// ToJSON serializes the model to JSON with indentation.
//
// This is useful for:
//   - Storing models in files (.architecture/model.json)
//   - Sending models via APIs (MCP, REST, etc.)
//   - Version control (readable diffs)
//
// Example:
//
//	model, _ := transformer.Transform(program)
//	jsonData, _ := model.ToJSON()
//	os.WriteFile("model.json", jsonData, 0644)
func (m *Model) ToJSON() ([]byte, error) {
	return json.MarshalIndent(m, "", "  ")
}

// Journey represents a user journey.
//
// A journey describes a flow across architecture elements, showing how users
// or systems interact with the architecture through a sequence of steps.
// Each step is a relation between elements.
type Journey struct {
	ID          string     `json:"id"`                    // Journey ID (e.g., "login", "checkout")
	Title       string     `json:"title,omitempty"`       // Optional title
	Description string     `json:"description,omitempty"` // Optional description
	Steps       []Relation `json:"steps"`                 // Sequence of relations representing the journey steps
	Location    Location   `json:"location,omitempty"`    // Source location (optional)
}

// FromJSON deserializes a model from JSON.
//
// This is useful for:
//   - Loading saved models
//   - Receiving models via APIs
//   - Round-trip testing (DSL → Model → JSON → Model → DSL)
//
// Example:
//
//	jsonData, _ := os.ReadFile("model.json")
//	model, _ := model.FromJSON(jsonData)
func FromJSON(data []byte) (*Model, error) {
	var m Model
	if err := json.Unmarshal(data, &m); err != nil {
		return nil, err
	}
	return &m, nil
}

// Clone creates a deep copy of the model.
//
// This is useful for:
//   - Returning safe copies from stateful stores (kernel)
//   - Creating snapshots/variants
//   - Testing
func (m *Model) Clone() *Model {
	if m == nil {
		return nil
	}

	// Use JSON round-trip for deep copy
	data, err := json.Marshal(m)
	if err != nil {
		// Fallback to shallow copy if JSON fails
		return &Model{
			Version:      m.Version,
			GeneratedAt:  m.GeneratedAt,
			Architecture: m.Architecture, // Shallow copy
		}
	}

	cloned, err := FromJSON(data)
	if err != nil {
		// Fallback to shallow copy
		return &Model{
			Version:      m.Version,
			GeneratedAt:  m.GeneratedAt,
			Architecture: m.Architecture,
		}
	}

	return cloned
}

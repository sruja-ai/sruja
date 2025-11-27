// pkg/language/ast.go
// Package language provides DSL parsing and AST (Abstract Syntax Tree) structures.
//
// The AST represents the parsed structure of a Sruja DSL file. Each node in the AST
// corresponds to a construct in the DSL (architecture, system, container, etc.) and
// includes source location information for error reporting and IDE features.
//
// Example usage:
//
//	parser, _ := language.NewParser()
//	program, _ := parser.Parse("example.sruja", dslText)
//	architecture := program.Architecture
//	system := architecture.Systems[0]
package language

import (
	"fmt"
)

// ============================================================================
// Shared Base Types
// ============================================================================

// MetaEntry represents a single metadata key-value pair.
//
// Metadata is freeform key-value pairs that allow infinite extension
// without modifying the core DSL grammar. Plugins can consume metadata
// for validation, code generation, diagram customization, etc.
//
// Example DSL:
//
//	metadata {
//	    team: "Payments"
//	    tier: "critical"
//	    rate_limit: "100/s"
//	}
type MetaEntry struct {
	Key   string `parser:"@Ident ':'"`
	Value string `parser:"@String"`
}

func (m *MetaEntry) Location() SourceLocation { return SourceLocation{} }

// SourceLocation represents the position of a node in the source file.
//
// This is used for:
//   - Error reporting (showing where errors occurred)
//   - IDE features (go-to-definition, hover, etc.)
//   - Validation (reporting which element has issues)
//
// All positions are 1-based (line 1, column 1 is the first character).
type SourceLocation struct {
	File   string // File path where this node appears
	Line   int    // 1-based line number
	Column int    // 1-based column number
	Offset int    // Byte offset in file (0-based)
	Length int    // Length of the node in bytes
}

// String returns a human-readable location string in the format "file:line:column".
//
// Example: "example.sruja:5:12"
func (l SourceLocation) String() string {
	return fmt.Sprintf("%s:%d:%d", l.File, l.Line, l.Column)
}

// ASTNode is the base interface for all AST nodes.
//
// Every node in the AST implements this interface, which provides:
//   - Source location information (for error reporting)
//
// All concrete AST types (Architecture, System, Container, etc.) implement this interface.
type ASTNode interface {
	Location() SourceLocation
}

// ============================================================================
// Architecture (Top-Level)
// ============================================================================

// Architecture is the top-level root of a document.
//
// An architecture contains all the elements that make up the system architecture:
// systems, persons, relations, requirements, ADRs, shared artifacts, libraries, and journeys.
//
// Example DSL:
//
//	architecture "My System" {
//	  import "shared.sruja"
//	  system API "API Service" { ... }
//	  person User "End User"
//	}
type Architecture struct {
	Name   string             `parser:"'architecture' @String"`
	LBrace string             `parser:"'{'"`
	Items  []ArchitectureItem `parser:"@@*"`
	RBrace string             `parser:"'}'"`

	// Post-processed fields
	Imports         []*ImportSpec
	ResolvedImports []*ImportedArchitecture
	Systems         []*System
	Persons         []*Person
	Relations       []*Relation
	Requirements    []*Requirement
	ADRs            []*ADR
	SharedArtifacts []*SharedArtifact
	Libraries       []*Library
	Journeys        []*Journey
	Metadata        []*MetaEntry // Metadata from metadata blocks
	Contracts       []*Contract
	Constraints     []*ConstraintEntry
	Conventions     []*ConventionEntry
	Entities        []*Entity
	Events          []*DomainEvent
	DeploymentNodes []*DeploymentNode
	DynamicViews    []*DynamicView
}

func (a *Architecture) Location() SourceLocation { return SourceLocation{} }

// ArchitectureItem is a union type for items that can appear at the architecture level.
type ArchitectureItem struct {
	Import           *ImportSpec       `parser:"@@"`
	System           *System           `parser:"| @@"`
	Person           *Person           `parser:"| @@"`
	Relation         *Relation         `parser:"| @@"`
	Requirement      *Requirement      `parser:"| @@"`
	ADR              *ADR              `parser:"| @@"`
	SharedArtifact   *SharedArtifact   `parser:"| @@"`
	Library          *Library          `parser:"| @@"`
	Journey          *Journey          `parser:"| @@"`
	Metadata         *MetadataBlock    `parser:"| @@"`
	ContractsBlock   *ContractsBlock   `parser:"| 'contracts' '{' @@ '}'"`
	ConstraintsBlock *ConstraintsBlock `parser:"| 'constraints' '{' @@* '}'"`
	ConventionsBlock *ConventionsBlock `parser:"| 'conventions' '{' @@* '}'"`
	EntitiesBlock    *EntitiesBlock    `parser:"| 'entities' '{' @@ '}'"`
	Domain           *DomainBlock      `parser:"| @@"`
	DeploymentNode   *DeploymentNode   `parser:"| @@"`
	DynamicView      *DynamicView      `parser:"| @@"`
}

// MetadataBlock represents a metadata block.
//
// Example DSL:
//
//	metadata {
//	    team: "Payments"
//	    tier: "critical"
//	}
type MetadataBlock struct {
	LBrace  string       `parser:"'metadata' '{'"`
	Entries []*MetaEntry `parser:"@@*"`
	RBrace  string       `parser:"'}'"`
}

func (m *MetadataBlock) Location() SourceLocation { return SourceLocation{} }

// ============================================================================
// Contracts, Constraints, Conventions
// ============================================================================

type ContractsBlock struct {
	Contracts []*Contract `parser:"@@*"`
}

type Contract struct {
	Kind string        `parser:"@( 'api' | 'event' | 'data' )"`
	ID   string        `parser:"@Ident"`
	L    string        `parser:"'{'"`
	Body *ContractBody `parser:"@@"`
	R    string        `parser:"'}'"`
}

func (c *Contract) Location() SourceLocation { return SourceLocation{} }

type ContractBody struct {
	Version       *string             `parser:"( 'version' @String )?"`
	Status        *string             `parser:"( 'status' @String )?"`
	Endpoint      *string             `parser:"( 'endpoint' @String )?"`
	Method        *string             `parser:"( 'method' @String )?"`
	Request       *SchemaBlock        `parser:"( 'request' '{' @@ '}' )?"`
	Response      *SchemaBlock        `parser:"( 'response' '{' @@ '}' )?"`
	Errors        []string            `parser:"( 'errors' '[' @String ( ',' @String )* ']' )?"`
	Schema        *SchemaBlock        `parser:"( 'schema' '{' @@ '}' )?"`
	Retention     *string             `parser:"( 'retention' @String )?"`
	RequestMap    *string             `parser:"( 'request_map' @String )?"`
	ResponseMap   *string             `parser:"( 'response_map' @String )?"`
	ErrorMap      []string            `parser:"( 'error_map' '[' @String ( ',' @String )* ']' )?"`
	EmitsSchema   *string             `parser:"( 'emits_schema' @String )?"`
	WritesSchema  *string             `parser:"( 'writes_schema' @String )?"`
	Deprecation   *DeprecationBlock   `parser:"( 'deprecation' '{' @@ '}' )?"`
	Compatibility *CompatibilityBlock `parser:"( 'compatibility' '{' @@ '}' )?"`
	Guarantees    *GuaranteesBlock    `parser:"( 'guarantees' '{' @@* '}' )?"`
}

type SchemaBlock struct {
	Entries []*SchemaEntry `parser:"@@*"`
}

type SchemaEntry struct {
	Key  string    `parser:"@Ident ':'"`
	Type *TypeSpec `parser:"@@"`
}

func (s *SchemaBlock) Location() SourceLocation { return SourceLocation{} }

type ConstraintsBlock struct {
	Entries []*ConstraintEntry `parser:"@@*"`
}

type ConstraintEntry struct {
	Key   string `parser:"@Ident"`
	Value string `parser:"@String"`
}

func (c *ConstraintEntry) Location() SourceLocation { return SourceLocation{} }

type ConventionsBlock struct {
	Entries []*ConventionEntry `parser:"@@*"`
}

type ConventionEntry struct {
	Key   string `parser:"@Ident"`
	Value string `parser:"@String"`
}

func (c *ConventionEntry) Location() SourceLocation { return SourceLocation{} }

// ============================================================================
// Imports & Multi-Architecture Composition
// ============================================================================

// ImportSpec represents an import statement.
//
// Example DSL:
//
//	import "shared.sruja"
//	import "common.sruja" as common
type ImportSpec struct {
	Path  string  `parser:"'import' @String"`
	Alias *string `parser:"( 'as' @Ident )?"`
}

func (i *ImportSpec) Location() SourceLocation { return SourceLocation{} }

// ImportedArchitecture represents a resolved imported architecture.
type ImportedArchitecture struct {
	Alias        string
	Architecture *Architecture
}

// ============================================================================
// Systems
// ============================================================================

// System represents a system declaration.
//
// A system is a high-level software system that delivers value to users.
// It can contain containers, data stores, queues, persons, components, and relationships.
//
// Example DSL:
//
//	system API "API Service" {
//	  container WebApp "Web Application" { ... }
//	  datastore DB "PostgreSQL Database"
//	}
type System struct {
	ID          string       `parser:"'system' @Ident"`
	Label       string       `parser:"@String"`
	Description *string      `parser:"( @String )?"`
	LBrace      *string      `parser:"( '{' )?"`
	Items       []SystemItem `parser:"@@*"`
	RBrace      *string      `parser:"( '}' )?"`

	// Post-processed fields
	Containers   []*Container
	DataStores   []*DataStore
	Queues       []*Queue
	Persons      []*Person
	Components   []*Component
	Requirements []*Requirement
	ADRs         []*ADR
	Relations    []*Relation
	Metadata     []*MetaEntry // Metadata from metadata blocks
	Contracts    []*Contract
	Constraints  []*ConstraintEntry
	Conventions  []*ConventionEntry
	Entities     []*Entity
	Events       []*DomainEvent
}

func (s *System) Location() SourceLocation { return SourceLocation{} }

// SystemItem is a union type for items that can appear in a system.
type SystemItem struct {
	Container        *Container        `parser:"@@"`
	DataStore        *DataStore        `parser:"| @@"`
	Queue            *Queue            `parser:"| @@"`
	Person           *Person           `parser:"| @@"`
	Relation         *Relation         `parser:"| @@"`
	Requirement      *Requirement      `parser:"| @@"`
	ADR              *ADR              `parser:"| @@"`
	Metadata         *MetadataBlock    `parser:"| @@"`
	ContractsBlock   *ContractsBlock   `parser:"| 'contracts' '{' @@ '}'"`
	ConstraintsBlock *ConstraintsBlock `parser:"| 'constraints' '{' @@* '}'"`
	ConventionsBlock *ConventionsBlock `parser:"| 'conventions' '{' @@* '}'"`
	EntitiesBlock    *EntitiesBlock    `parser:"| 'entities' '{' @@ '}'"`
	EventsBlock      *EventsBlock      `parser:"| 'events' '{' @@ '}'"`
}

// ============================================================================
// Containers
// ============================================================================

// Container represents a container declaration.
//
// A container is an application within a system. Examples:
//   - Web applications (React, Angular, etc.)
//   - API services
//   - Background jobs
//
// Containers can contain components, data stores, queues, and have relationships.
//
// Example DSL:
//
//	container WebApp "Web Application" {
//	  technology "React"
//	  tags ["frontend", "spa"]
//	  component ShoppingCart { ... }
//	}
type Container struct {
	ID          string          `parser:"'container' @Ident"`
	Label       string          `parser:"@String"`
	Description *string         `parser:"( @String )?"`
	LBrace      *string         `parser:"( '{' )?"`
	Items       []ContainerItem `parser:"@@*"`
	RBrace      *string         `parser:"( '}' )?"`

	// Post-processed fields
	Components   []*Component
	DataStores   []*DataStore
	Queues       []*Queue
	Requirements []*Requirement
	ADRs         []*ADR
	Relations    []*Relation
	Metadata     []*MetaEntry // Metadata from metadata blocks
	Contracts    []*Contract
	Constraints  []*ConstraintEntry
	Conventions  []*ConventionEntry
	Version      *string
	Entities     []*Entity
	Events       []*DomainEvent
}

func (c *Container) Location() SourceLocation { return SourceLocation{} }

// ContainerItem is a union type for items that can appear in a container.
type ContainerItem struct {
	Technology       *string           `parser:"'technology' @String"`
	Tags             []string          `parser:"| 'tags' '[' @String ( ',' @String )* ']'"`
	Version          *string           `parser:"| 'version' @String"`
	Component        *Component        `parser:"| @@"`
	DataStore        *DataStore        `parser:"| @@"`
	Queue            *Queue            `parser:"| @@"`
	Relation         *Relation         `parser:"| @@"`
	Requirement      *Requirement      `parser:"| @@"`
	ADR              *ADR              `parser:"| @@"`
	Metadata         *MetadataBlock    `parser:"| @@"`
	ContractsBlock   *ContractsBlock   `parser:"| 'contracts' '{' @@ '}'"`
	ConstraintsBlock *ConstraintsBlock `parser:"| 'constraints' '{' @@* '}'"`
	ConventionsBlock *ConventionsBlock `parser:"| 'conventions' '{' @@* '}'"`
	EntitiesBlock    *EntitiesBlock    `parser:"| 'entities' '{' @@ '}'"`
	EventsBlock      *EventsBlock      `parser:"| 'events' '{' @@ '}'"`
}

// ============================================================================
// Components
// ============================================================================

// Component represents a component declaration.
//
// A component is a logical module within a container. Examples:
//   - ShoppingCart component in a WebApp container
//   - PaymentProcessor component in a PaymentService container
//
// Components are the lowest level of detail in the C4 model hierarchy.
//
// Example DSL:
//
//	component ShoppingCart "Shopping Cart" {
//	  technology "React"
//	}
type Component struct {
	ID          string          `parser:"'component' @Ident"`
	Label       string          `parser:"@String"`
	Description *string         `parser:"( @String )?"`
	Technology  *string         `parser:"( 'technology' @String )?"`
	LBrace      *string         `parser:"( '{' )?"`
	Items       []ComponentItem `parser:"@@*"`
	RBrace      *string         `parser:"( '}' )?"`

	// Post-processed fields
	Requirements []*Requirement
	ADRs         []*ADR
	Relations    []*Relation
	Metadata     []*MetaEntry // Metadata from metadata blocks
}

func (c *Component) Location() SourceLocation { return SourceLocation{} }

// ComponentItem is a union type for items that can appear in a component.
type ComponentItem struct {
	Requirement      *Requirement      `parser:"@@"`
	ADR              *ADR              `parser:"| @@"`
	Relation         *Relation         `parser:"| @@"`
	Metadata         *MetadataBlock    `parser:"| @@"`
	Behavior         *BehaviorBlock    `parser:"| 'behavior' '{' @@* '}'"`
	ContractsBlock   *ContractsBlock   `parser:"| 'contracts' '{' @@ '}'"`
	ConstraintsBlock *ConstraintsBlock `parser:"| 'constraints' '{' @@* '}'"`
	ConventionsBlock *ConventionsBlock `parser:"| 'conventions' '{' @@* '}'"`
	DependsOn        *string           `parser:"| 'depends_on' @Ident"`
}

// ============================================================================
// DataStore & Queue (specialized containers)
// ============================================================================

// DataStore represents a data store declaration.
//
// A data store is where data is persisted. Examples:
//   - PostgreSQL database
//   - MongoDB database
//   - Redis cache
//
// Example DSL:
//
//	datastore DB "PostgreSQL Database" "Main application database"
type DataStore struct {
	ID          string          `parser:"'datastore' @Ident"`
	Label       string          `parser:"@String"`
	Description *string         `parser:"( @String )?"`
	LBrace      *string         `parser:"( '{' )?"`
	Items       []DataStoreItem `parser:"@@*"`
	RBrace      *string         `parser:"( '}' )?"`

	Metadata []*MetaEntry
}

func (d *DataStore) Location() SourceLocation { return SourceLocation{} }

// Queue represents a queue declaration.
//
// A queue is a message queue or event stream. Examples:
//   - RabbitMQ queue
//   - Kafka topic
//   - SQS queue
//
// Example DSL:
//
//	queue Events "Event Queue" "Handles async events"
type Queue struct {
	ID          string      `parser:"'queue' @Ident"`
	Label       string      `parser:"@String"`
	Description *string     `parser:"( @String )?"`
	LBrace      *string     `parser:"( '{' )?"`
	Items       []QueueItem `parser:"@@*"`
	RBrace      *string     `parser:"( '}' )?"`

	Metadata []*MetaEntry
}

func (q *Queue) Location() SourceLocation { return SourceLocation{} }

// ============================================================================
// Person
// ============================================================================

// Person represents a person (user) declaration.
//
// A person is an external user or actor that interacts with the system.
//
// Example DSL:
//
//	person User "End User"
type Person struct {
	ID     string       `parser:"'person' @Ident"`
	Label  string       `parser:"@String"`
	LBrace *string      `parser:"( '{' )?"`
	Items  []PersonItem `parser:"@@*"`
	RBrace *string      `parser:"( '}' )?"`

	Metadata []*MetaEntry
}

func (p *Person) Location() SourceLocation { return SourceLocation{} }

// ============================================================================
// Relations + Qualified References
// ============================================================================

// Relation represents a relationship between elements.
//
// A relation describes how one element interacts with another.
// The From and To fields can be qualified references (e.g., "System.Container").
//
// Example DSL:
//
//	User -> WebApp "Uses"
//	WebApp -> Database "Reads/Writes"
//	API -> UserService "calls" "Makes HTTP requests"
type Relation struct {
	From  string  `parser:"@Ident"` // possibly qualified
	Arrow string  `parser:"'->'"`   // explicit arrow
	To    string  `parser:"@Ident"` // possibly qualified
	Verb  *string `parser:"( @Ident | @String )?"`
	Label *string `parser:"( @String )?"`

	// Post-processed: resolved refs
	ResolvedFrom Element
	ResolvedTo   Element
}

func (r *Relation) Location() SourceLocation { return SourceLocation{} }

// Element is implemented by System, Container, Component, Person, DataStore, Queue
type Element interface {
	ASTNode
	GetID() string
	GetLabel() string
}

func (s *System) GetID() string    { return s.ID }
func (c *Container) GetID() string { return c.ID }
func (c *Component) GetID() string { return c.ID }
func (p *Person) GetID() string    { return p.ID }
func (d *DataStore) GetID() string { return d.ID }
func (q *Queue) GetID() string     { return q.ID }

func (s *System) GetLabel() string    { return s.Label }
func (c *Container) GetLabel() string { return c.Label }
func (c *Component) GetLabel() string { return c.Label }
func (p *Person) GetLabel() string    { return p.Label }
func (d *DataStore) GetLabel() string { return d.Label }
func (q *Queue) GetLabel() string     { return q.Label }

// ============================================================================
// Requirements & ADRs
// ============================================================================

// Requirement represents a requirement declaration.
//
// A requirement describes what the system must do or constraints it must satisfy.
// Types:
//   - "functional": What the system must do
//   - "constraint": Constraints on the system
//   - "performance": Performance requirements
//   - "security": Security requirements
//
// Example DSL:
//
//	requirement R1 functional "Must handle 10k concurrent users"
type Requirement struct {
	ID          string `parser:"'requirement' @Ident"`
	Type        string `parser:"@Ident"` // functional|performance|security|constraint
	Description string `parser:"@String"`
}

func (r *Requirement) Location() SourceLocation { return SourceLocation{} }

// ADR represents an Architecture Decision Record.
//
// An ADR documents an important architectural decision.
//
// Example DSL:
//
//	adr ADR001 "Use microservices architecture for scalability"
type ADR struct {
	ID    string `parser:"'adr' @Ident"`
	Title string `parser:"@String"`
}

func (a *ADR) Location() SourceLocation { return SourceLocation{} }

// ============================================================================
// Shared Artifacts & Libraries
// ============================================================================

// SharedArtifact represents a shared artifact declaration.
//
// A shared artifact is a reusable component or service shared across systems.
//
// Example DSL:
//
//	sharedArtifact Auth "Authentication Service" version "1.0.0" owner "platform-team"
type SharedArtifact struct {
	ID      string  `parser:"'sharedArtifact' @Ident"`
	Label   string  `parser:"@String"`
	Version *string `parser:"( 'version' @String )?"`
	Owner   *string `parser:"( 'owner' @String )?"`
}

func (a *SharedArtifact) Location() SourceLocation { return SourceLocation{} }

// Library represents a library declaration.
//
// A library is a reusable code library or framework.
//
// Example DSL:
//
//	library React "React Framework" version "18.0.0" owner "facebook"
type Library struct {
	ID      string  `parser:"'library' @Ident"`
	Label   string  `parser:"@String"`
	Version *string `parser:"( 'version' @String )?"`
	Owner   *string `parser:"( 'owner' @String )?"`
}

func (l *Library) Location() SourceLocation { return SourceLocation{} }

// ============================================================================
// Journeys & Journey Steps
// ============================================================================

// Journey represents a user journey declaration.
//
// A journey describes a flow across architecture elements, showing how users
// or systems interact with the architecture through a sequence of steps.
//
// Example DSL:
//
//	journey login {
//	  title "User logs into the system"
//	  steps {
//	    user -> ui "enters username/password"
//	    ui -> api "POST /login"
//	  }
//	}
type Journey struct {
	ID     string        `parser:"'journey' @Ident"`
	LBrace string        `parser:"'{'"`
	Items  []JourneyItem `parser:"@@*"`
	RBrace string        `parser:"'}'"`

	// Post-processed fields
	Title string
	Steps []*JourneyStep
}

func (j *Journey) Location() SourceLocation { return SourceLocation{} }

// JourneyItem is a union type for items that can appear in a journey.
type JourneyItem struct {
	Title *string       `parser:"'title' @String"`
	Steps *JourneySteps `parser:"| 'steps' '{' @@* '}'"`
}

// JourneySteps represents a collection of journey steps.
type JourneySteps struct {
	Items []*JourneyStep `parser:"@@*"`
}

// JourneyStep represents a single step in a journey.
//
// Steps can use different arrow directions:
//   - "->": Forward direction
//   - "<-": Reverse direction
//   - "<->": Bidirectional
type JourneyStep struct {
	From  string  `parser:"@Ident"`
	Arrow string  `parser:"@( '->' | '<-' | '<->' )"`
	To    string  `parser:"@Ident"`
	Label *string `parser:"( @String )?"`

	// Resolved references after PostProcess
	ResolvedFrom Element
	ResolvedTo   Element
}

func (s *JourneyStep) Location() SourceLocation { return SourceLocation{} }

// ============================================================================
// Deployment View
// ============================================================================

// DeploymentNode represents a deployment node (e.g., "Production", "AWS", "Server").
//
// Example DSL:
//
//	deployment "Production" {
//	  node "AWS" {
//	    node "US-East-1" {
//	       containerInstance WebApp
//	    }
//	  }
//	}
type DeploymentNode struct {
	Type        string               `parser:"@( 'deployment' | 'node' )"`
	ID          string               `parser:"@Ident"`
	Label       string               `parser:"@String"`
	Description *string              `parser:"( @String )?"`
	LBrace      string               `parser:"'{'"`
	Items       []DeploymentNodeItem `parser:"@@*"`
	RBrace      string               `parser:"'}'"`

	// Post-processed
	Children           []*DeploymentNode
	ContainerInstances []*ContainerInstance
	Infrastructure     []*InfrastructureNode
}

func (d *DeploymentNode) Location() SourceLocation { return SourceLocation{} }

type DeploymentNodeItem struct {
	Node              *DeploymentNode     `parser:"@@"`
	ContainerInstance *ContainerInstance  `parser:"| @@"`
	Infrastructure    *InfrastructureNode `parser:"| @@"`
}

// ContainerInstance represents an instance of a container in a deployment node.
type ContainerInstance struct {
	ContainerID string  `parser:"'containerInstance' @Ident"`
	InstanceID  *string `parser:"( 'instanceId' @Number )?"` // Optional instance ID
}

func (c *ContainerInstance) Location() SourceLocation { return SourceLocation{} }

// InfrastructureNode represents an infrastructure node (e.g., Load Balancer, DNS).
type InfrastructureNode struct {
	ID          string  `parser:"'infrastructure' @Ident"`
	Label       string  `parser:"@String"`
	Description *string `parser:"( @String )?"`
}

func (i *InfrastructureNode) Location() SourceLocation { return SourceLocation{} }

func (d *DeploymentNode) PostProcess() {
	for _, item := range d.Items {
		if item.Node != nil {
			item.Node.PostProcess()
			d.Children = append(d.Children, item.Node)
		}
		if item.ContainerInstance != nil {
			d.ContainerInstances = append(d.ContainerInstances, item.ContainerInstance)
		}
		if item.Infrastructure != nil {
			d.Infrastructure = append(d.Infrastructure, item.Infrastructure)
		}
	}
}

// ============================================================================
// Dynamic View
// ============================================================================

// DynamicView represents a dynamic view (runtime flow).
//
// Example DSL:
//
//	dynamic "Login Flow" {
//	   User -> WebApp "Credentials"
//	   WebApp -> DB "Verify"
//	}
type DynamicView struct {
	ID          string            `parser:"'dynamic' @Ident"` // Optional ID? Usually dynamic views have titles. Let's use ID as title/name for now.
	Title       string            `parser:"@String"`
	Description *string           `parser:"( @String )?"`
	LBrace      string            `parser:"'{'"`
	Items       []DynamicViewItem `parser:"@@*"`
	RBrace      string            `parser:"'}'"`

	// Post-processed
	Steps []*DynamicViewStep
}

func (d *DynamicView) Location() SourceLocation { return SourceLocation{} }

type DynamicViewItem struct {
	Step *DynamicViewStep `parser:"@@"`
}

type DynamicViewStep struct {
	From        string  `parser:"@Ident"`
	Arrow       string  `parser:"'->'"`
	To          string  `parser:"@Ident"`
	Description *string `parser:"( @String )?"`         // Description of the interaction in this specific context
	Order       *string `parser:"( 'order' @String )?"` // Explicit ordering if needed
}

func (s *DynamicViewStep) Location() SourceLocation { return SourceLocation{} }

func (d *DynamicView) PostProcess() {
	for _, item := range d.Items {
		if item.Step != nil {
			d.Steps = append(d.Steps, item.Step)
		}
	}
}

// Program represents the complete parsed program
type Program struct {
	Architecture *Architecture
}

// ============================================================================
// Post-Processing Methods
// ============================================================================

// PostProcess populates convenience fields from parsed items.
func (a *Architecture) PostProcess() {
	for _, item := range a.Items {
		if item.Import != nil {
			a.Imports = append(a.Imports, item.Import)
		}
		if item.System != nil {
			item.System.PostProcess()
			a.Systems = append(a.Systems, item.System)
		}
		if item.Person != nil {
			item.Person.PostProcess()
			a.Persons = append(a.Persons, item.Person)
		}
		if item.Relation != nil {
			a.Relations = append(a.Relations, item.Relation)
		}
		if item.Requirement != nil {
			a.Requirements = append(a.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			a.ADRs = append(a.ADRs, item.ADR)
		}
		if item.SharedArtifact != nil {
			a.SharedArtifacts = append(a.SharedArtifacts, item.SharedArtifact)
		}
		if item.Library != nil {
			a.Libraries = append(a.Libraries, item.Library)
		}
		if item.Journey != nil {
			item.Journey.PostProcess()
			a.Journeys = append(a.Journeys, item.Journey)
		}
		if item.Metadata != nil {
			a.Metadata = append(a.Metadata, item.Metadata.Entries...)
		}
		if item.ContractsBlock != nil {
			a.Contracts = append(a.Contracts, item.ContractsBlock.Contracts...)
		}
		if item.ConstraintsBlock != nil {
			a.Constraints = append(a.Constraints, item.ConstraintsBlock.Entries...)
		}
		if item.ConventionsBlock != nil {
			a.Conventions = append(a.Conventions, item.ConventionsBlock.Entries...)
		}
		if item.EntitiesBlock != nil {
			a.Entities = append(a.Entities, item.EntitiesBlock.Entities...)
		}
		if item.Domain != nil && item.Domain.EntitiesBlock != nil {
			a.Entities = append(a.Entities, item.Domain.EntitiesBlock.Entities...)
		}
		if item.Domain != nil && item.Domain.EventsBlock != nil {
			a.Events = append(a.Events, item.Domain.EventsBlock.Events...)
		}
		if item.DeploymentNode != nil {
			item.DeploymentNode.PostProcess()
			a.DeploymentNodes = append(a.DeploymentNodes, item.DeploymentNode)
		}
		if item.DynamicView != nil {
			item.DynamicView.PostProcess()
			a.DynamicViews = append(a.DynamicViews, item.DynamicView)
		}
	}
}

// PostProcess populates convenience fields from system items.
func (s *System) PostProcess() {
	for _, item := range s.Items {
		if item.Container != nil {
			item.Container.PostProcess()
			s.Containers = append(s.Containers, item.Container)
			// Collect components from containers
			s.Components = append(s.Components, item.Container.Components...)
		}
		if item.DataStore != nil {
			item.DataStore.PostProcess()
			s.DataStores = append(s.DataStores, item.DataStore)
		}
		if item.Queue != nil {
			item.Queue.PostProcess()
			s.Queues = append(s.Queues, item.Queue)
		}
		if item.Person != nil {
			item.Person.PostProcess()
			s.Persons = append(s.Persons, item.Person)
		}
		if item.Requirement != nil {
			s.Requirements = append(s.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			s.ADRs = append(s.ADRs, item.ADR)
		}
		if item.Relation != nil {
			s.Relations = append(s.Relations, item.Relation)
		}
		if item.Metadata != nil {
			s.Metadata = append(s.Metadata, item.Metadata.Entries...)
		}
		if item.ContractsBlock != nil {
			s.Contracts = append(s.Contracts, item.ContractsBlock.Contracts...)
		}
		if item.ConstraintsBlock != nil {
			s.Constraints = append(s.Constraints, item.ConstraintsBlock.Entries...)
		}
		if item.ConventionsBlock != nil {
			s.Conventions = append(s.Conventions, item.ConventionsBlock.Entries...)
		}
		if item.EntitiesBlock != nil {
			s.Entities = append(s.Entities, item.EntitiesBlock.Entities...)
		}
		if item.EventsBlock != nil {
			s.Events = append(s.Events, item.EventsBlock.Events...)
		}
	}
}

// PostProcess populates convenience fields from container items.
func (c *Container) PostProcess() {
	for _, item := range c.Items {
		if item.Component != nil {
			item.Component.PostProcess()
			c.Components = append(c.Components, item.Component)
		}
		if item.DataStore != nil {
			item.DataStore.PostProcess()
			c.DataStores = append(c.DataStores, item.DataStore)
		}
		if item.Queue != nil {
			item.Queue.PostProcess()
			c.Queues = append(c.Queues, item.Queue)
		}
		if item.Requirement != nil {
			c.Requirements = append(c.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			c.ADRs = append(c.ADRs, item.ADR)
		}
		if item.Relation != nil {
			c.Relations = append(c.Relations, item.Relation)
		}
		if item.Metadata != nil {
			c.Metadata = append(c.Metadata, item.Metadata.Entries...)
		}
		if item.ContractsBlock != nil {
			c.Contracts = append(c.Contracts, item.ContractsBlock.Contracts...)
		}
		if item.ConstraintsBlock != nil {
			c.Constraints = append(c.Constraints, item.ConstraintsBlock.Entries...)
		}
		if item.ConventionsBlock != nil {
			c.Conventions = append(c.Conventions, item.ConventionsBlock.Entries...)
		}
		if item.EntitiesBlock != nil {
			c.Entities = append(c.Entities, item.EntitiesBlock.Entities...)
		}
		if item.EventsBlock != nil {
			c.Events = append(c.Events, item.EventsBlock.Events...)
		}
	}
}

// PostProcess populates convenience fields from component items.
func (c *Component) PostProcess() {
	for _, item := range c.Items {
		if item.Requirement != nil {
			c.Requirements = append(c.Requirements, item.Requirement)
		}
		if item.ADR != nil {
			c.ADRs = append(c.ADRs, item.ADR)
		}
		if item.Relation != nil {
			c.Relations = append(c.Relations, item.Relation)
		}
		if item.Metadata != nil {
			c.Metadata = append(c.Metadata, item.Metadata.Entries...)
		}
		if item.Behavior != nil {
			// Behavior could be used by validators; collected via metadata-like map? omitted
		}
		if item.ContractsBlock != nil {
			// Attach component-level contracts (behavior/api/event/data)
		}
		if item.ConstraintsBlock != nil {
			// Extendable: collect constraints
		}
		if item.ConventionsBlock != nil {
			// Extendable: collect conventions
		}
	}
}

// PostProcess populates convenience fields from journey items.
func (j *Journey) PostProcess() {
	for _, item := range j.Items {
		if item.Title != nil {
			j.Title = *item.Title
		}
		if item.Steps != nil {
			j.Steps = append(j.Steps, item.Steps.Items...)
		}
	}
}

// Element item unions for metadata blocks
type DataStoreItem struct {
	Metadata *MetadataBlock `parser:"@@"`
}

type QueueItem struct {
	Metadata *MetadataBlock `parser:"@@"`
}

type PersonItem struct {
	Metadata *MetadataBlock `parser:"@@"`
}

// PostProcess populates metadata from inline blocks for DataStore.
func (d *DataStore) PostProcess() {
	for _, it := range d.Items {
		if it.Metadata != nil {
			d.Metadata = append(d.Metadata, it.Metadata.Entries...)
		}
	}
}

// PostProcess populates metadata from inline blocks for Queue.
func (q *Queue) PostProcess() {
	for _, it := range q.Items {
		if it.Metadata != nil {
			q.Metadata = append(q.Metadata, it.Metadata.Entries...)
		}
	}
}

// PostProcess populates metadata from inline blocks for Person.
func (p *Person) PostProcess() {
	for _, it := range p.Items {
		if it.Metadata != nil {
			p.Metadata = append(p.Metadata, it.Metadata.Entries...)
		}
	}
}

// ============================================================================
// Metadata Helper Methods
// ============================================================================

// MetaString returns a metadata value as a string.
// Returns the value and true if the key exists, empty string and false otherwise.
func (s *System) MetaString(key string) (string, bool) {
	for _, meta := range s.Metadata {
		if meta.Key == key {
			return meta.Value, true
		}
	}
	return "", false
}

// MetaString returns a metadata value as a string.
func (c *Container) MetaString(key string) (string, bool) {
	for _, meta := range c.Metadata {
		if meta.Key == key {
			return meta.Value, true
		}
	}
	return "", false
}

// MetaString returns a metadata value as a string.
func (c *Component) MetaString(key string) (string, bool) {
	for _, meta := range c.Metadata {
		if meta.Key == key {
			return meta.Value, true
		}
	}
	return "", false
}

// MetaString returns a metadata value as a string.
func (d *DataStore) MetaString(key string) (string, bool) {
	for _, meta := range d.Metadata {
		if meta.Key == key {
			return meta.Value, true
		}
	}
	return "", false
}

// MetaString returns a metadata value as a string.
func (q *Queue) MetaString(key string) (string, bool) {
	for _, meta := range q.Metadata {
		if meta.Key == key {
			return meta.Value, true
		}
	}
	return "", false
}

// MetaString returns a metadata value as a string.
func (p *Person) MetaString(key string) (string, bool) {
	for _, meta := range p.Metadata {
		if meta.Key == key {
			return meta.Value, true
		}
	}
	return "", false
}

// MetaString returns a metadata value as a string.
func (a *Architecture) MetaString(key string) (string, bool) {
	for _, meta := range a.Metadata {
		if meta.Key == key {
			return meta.Value, true
		}
	}
	return "", false
}

// HasMeta checks if a metadata key exists.
func (s *System) HasMeta(key string) bool {
	_, ok := s.MetaString(key)
	return ok
}

// HasMeta checks if a metadata key exists.
func (c *Container) HasMeta(key string) bool {
	_, ok := c.MetaString(key)
	return ok
}

// HasMeta checks if a metadata key exists.
func (c *Component) HasMeta(key string) bool {
	_, ok := c.MetaString(key)
	return ok
}

// HasMeta checks if a metadata key exists.
func (d *DataStore) HasMeta(key string) bool {
	_, ok := d.MetaString(key)
	return ok
}

// HasMeta checks if a metadata key exists.
func (q *Queue) HasMeta(key string) bool {
	_, ok := q.MetaString(key)
	return ok
}

// HasMeta checks if a metadata key exists.
func (p *Person) HasMeta(key string) bool {
	_, ok := p.MetaString(key)
	return ok
}

// HasMeta checks if a metadata key exists.
func (a *Architecture) HasMeta(key string) bool {
	_, ok := a.MetaString(key)
	return ok
}

// AllMetadata returns all metadata as a map.
func (s *System) AllMetadata() map[string]string {
	result := make(map[string]string)
	for _, meta := range s.Metadata {
		result[meta.Key] = meta.Value
	}
	return result
}

// AllMetadata returns all metadata as a map.
func (c *Container) AllMetadata() map[string]string {
	result := make(map[string]string)
	for _, meta := range c.Metadata {
		result[meta.Key] = meta.Value
	}
	return result
}

// AllMetadata returns all metadata as a map.
func (c *Component) AllMetadata() map[string]string {
	result := make(map[string]string)
	for _, meta := range c.Metadata {
		result[meta.Key] = meta.Value
	}
	return result
}

// AllMetadata returns all metadata as a map.
func (d *DataStore) AllMetadata() map[string]string {
	result := make(map[string]string)
	for _, meta := range d.Metadata {
		result[meta.Key] = meta.Value
	}
	return result
}

// AllMetadata returns all metadata as a map.
func (q *Queue) AllMetadata() map[string]string {
	result := make(map[string]string)
	for _, meta := range q.Metadata {
		result[meta.Key] = meta.Value
	}
	return result
}

// AllMetadata returns all metadata as a map.
func (p *Person) AllMetadata() map[string]string {
	result := make(map[string]string)
	for _, meta := range p.Metadata {
		result[meta.Key] = meta.Value
	}
	return result
}

// AllMetadata returns all metadata as a map.
func (a *Architecture) AllMetadata() map[string]string {
	result := make(map[string]string)
	for _, meta := range a.Metadata {
		result[meta.Key] = meta.Value
	}
	return result
}

// MetaMap returns all metadata entries with a given prefix.
func (s *System) MetaMap(prefix string) map[string]string {
	result := make(map[string]string)
	for _, meta := range s.Metadata {
		if len(meta.Key) >= len(prefix) && meta.Key[:len(prefix)] == prefix {
			result[meta.Key] = meta.Value
		}
	}
	return result
}

// MetaMap returns all metadata entries with a given prefix.
func (c *Container) MetaMap(prefix string) map[string]string {
	result := make(map[string]string)
	for _, meta := range c.Metadata {
		if len(meta.Key) >= len(prefix) && meta.Key[:len(prefix)] == prefix {
			result[meta.Key] = meta.Value
		}
	}
	return result
}

// MetaMap returns all metadata entries with a given prefix.
func (c *Component) MetaMap(prefix string) map[string]string {
	result := make(map[string]string)
	for _, meta := range c.Metadata {
		if len(meta.Key) >= len(prefix) && meta.Key[:len(prefix)] == prefix {
			result[meta.Key] = meta.Value
		}
	}
	return result
}

type TypeSpec struct {
	Name     string   `parser:"@Ident"`
	Generics []string `parser:"( '<' @Ident ( ',' @Ident )* '>' )?"`
	Optional string   `parser:"( @'?' )?"`
}

type DeprecationBlock struct {
	Reason      *string `parser:"( 'reason' @String )?"`
	Sunset      *string `parser:"( 'sunset' @String )?"`
	Replacement *string `parser:"( 'replacement' @String )?"`
}

type CompatibilityBlock struct {
	BackwardsWith     *string `parser:"( 'backwards_with' @String )?"`
	ForwardsWith      *string `parser:"( 'forwards_with' @String )?"`
	BreakingChange    *string `parser:"( 'breaking_change' @String )?"`
	RequiresMigration *string `parser:"( 'requires_migration' @String )?"`
	Deprecates        *string `parser:"( 'deprecates' @String )?"`
}

type GuaranteesBlock struct {
	Entries []*GuaranteeEntry `parser:"@@*"`
}

type GuaranteeEntry struct {
	Key   string `parser:"@Ident"`
	Value string `parser:"@String"`
}

type BehaviorBlock struct {
	Entries []*BehaviorEntry `parser:"@@*"`
}

type BehaviorEntry struct {
	Key   string `parser:"@Ident"`
	Value string `parser:"@String"`
}

// ============================================================================
// Entities (Domain Model)
// ============================================================================

type EntitiesBlock struct {
	Entities []*Entity `parser:"@@*"`
}

type Entity struct {
	Kw   string      `parser:"'entity'"`
	Name string      `parser:"@Ident"`
	L    string      `parser:"'{'"`
	Body *EntityBody `parser:"@@"`
	R    string      `parser:"'}'"`
}

func (e *Entity) Location() SourceLocation { return SourceLocation{} }

type EntityBody struct {
	Description *string           `parser:"( 'description' @String )?"`
	Fields      *FieldBlock       `parser:"( 'fields' '{' @@* '}' )?"`
	Relations   *EntityRelBlock   `parser:"( 'relations' '{' @@* '}' )?"`
	Invariants  *InvariantBlock   `parser:"( 'invariants' '{' @@* '}' )?"`
	Lifecycle   *LifecycleBlock   `parser:"( 'lifecycle' '{' @@* '}' )?"`
	Versioning  *VersionBlock     `parser:"( 'versioning' '{' @@ '}' )?"`
	Constraints *ConstraintsBlock `parser:"( 'constraints' '{' @@* '}' )?"`
}

type FieldBlock struct {
	Entries []*SchemaEntry `parser:"@@*"`
}

type EntityRelBlock struct {
	Entries []*EntityRelation `parser:"@@*"`
}

type EntityRelation struct {
	Name   string `parser:"@Ident"`
	Arrow  string `parser:"'->'"`
	Target string `parser:"@Ident"`
}

type InvariantBlock struct {
	Items []string `parser:"@String*"`
}

type LifecycleBlock struct {
	Transitions []*LifecycleTransition `parser:"@@*"`
}

type LifecycleTransition struct {
	From  string `parser:"@Ident"`
	Arrow string `parser:"'->'"`
	To    string `parser:"@Ident"`
}

type VersionBlock struct {
	Current       *string `parser:"( 'current' @String )?"`
	BackwardsWith *string `parser:"( 'backwards_with' @String )?"`
}

// Domain wrapper
type DomainBlock struct {
	Kw            string         `parser:"'domain'"`
	Name          string         `parser:"@Ident"`
	L             string         `parser:"'{'"`
	EntitiesBlock *EntitiesBlock `parser:"( 'entities' '{' @@ '}' )?"`
	EventsBlock   *EventsBlock   `parser:"( 'events' '{' @@ '}' )?"`
	R             string         `parser:"'}'"`
}

// ============================================================================
// Domain-Driven Event DSL
// ============================================================================

type EventsBlock struct {
	Events []*DomainEvent `parser:"@@*"`
}

type DomainEvent struct {
	Kw   string     `parser:"'event'"`
	Name string     `parser:"@Ident"`
	L    string     `parser:"'{'"`
	Body *EventBody `parser:"@@"`
	R    string     `parser:"'}'"`
}

func (e *DomainEvent) Location() SourceLocation { return SourceLocation{} }

type EventBody struct {
	Version         *string               `parser:"( 'version' @String )?"`
	Entity          *string               `parser:"( 'entity' @Ident )?"`
	Category        *string               `parser:"( 'category' @String )?"`
	Description     *string               `parser:"( 'description' @String )?"`
	Schema          *SchemaBlock          `parser:"( 'schema' '{' @@ '}' )?"`
	Metadata        *EventMetaBlock       `parser:"( 'metadata' '{' @@* '}' )?"`
	Guarantees      *GuaranteesBlock      `parser:"( 'guarantees' '{' @@* '}' )?"`
	LifecycleEffect *EventLifecycleEffect `parser:"( 'lifecycle_effect' '{' @@ '}' )?"`
	Causes          *EventList            `parser:"( 'causes' '{' @@ '}' )?"`
	Publishers      *QualifiedList        `parser:"( 'publishers' '{' @@ '}' )?"`
	Consumers       *QualifiedList        `parser:"( 'consumers' '{' @@ '}' )?"`
	Versioning      *VersionBlock         `parser:"( 'versioning' '{' @@ '}' )?"`
}

type EventMetaBlock struct {
	Entries []*ConstraintEntry `parser:"@@*"`
}

type EventLifecycleEffect struct {
	From  *QualifiedState `parser:"@@"`
	Arrow string          `parser:"'->'"`
	To    *QualifiedState `parser:"@@"`
}

type QualifiedState struct {
	Entity string `parser:"@Ident"`
	Dot    string `parser:"'.'"`
	State  string `parser:"@Ident"`
}

type EventList struct {
	Items []string `parser:"@Ident*"`
}

type QualifiedList struct {
	Items []*QualifiedIdent `parser:"@@*"`
}

type QualifiedIdent struct {
	Parts []string `parser:"@Ident ( '.' @Ident )*"`
}

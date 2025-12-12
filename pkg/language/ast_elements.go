package language

import (
	"github.com/alecthomas/participle/v2/lexer"
)

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
	Pos         lexer.Position
	ID          string       `parser:"'system' @Ident"`
	Label       string       `parser:"( @String )?"`
	Description *string      `parser:"( @String )?"`
	Items       []SystemItem `parser:"( '{' @@* '}' )?"`

	// Post-processed fields
	Containers   []*Container
	DataStores   []*DataStore
	Queues       []*Queue
	Persons      []*Person
	Components   []*Component
	Relations    []*Relation
	Metadata     []*MetaEntry // Metadata from metadata blocks
	Contracts    []*Contract
	Constraints  []*ConstraintEntry
	Conventions  []*ConventionEntry
	Properties   map[string]string
	Style        map[string]string
	SLO          *SLOBlock
}

func (s *System) Location() SourceLocation {
	return SourceLocation{File: s.Pos.Filename, Line: s.Pos.Line, Column: s.Pos.Column, Offset: s.Pos.Offset}
}

// SystemItem is a union type for items that can appear in a system.
type SystemItem struct {
	Container        *Container        `parser:"@@"`
	DataStore        *DataStore        `parser:"| @@"`
	Queue            *Queue            `parser:"| @@"`
	Person           *Person           `parser:"| @@"`
	Metadata         *MetadataBlock    `parser:"| @@"`
	ContractsBlock   *ContractsBlock   `parser:"| 'contracts' '{' @@ '}'"`
	ConstraintsBlock *ConstraintsBlock `parser:"| 'constraints' '{' @@ '}'"`
	ConventionsBlock *ConventionsBlock `parser:"| 'conventions' '{' @@ '}'"`
	Properties       *PropertiesBlock  `parser:"| @@"`
	Style            *StyleBlock       `parser:"| @@"`
	SLO              *SLOBlock         `parser:"| @@"`
	Relation         *Relation         `parser:"| @@"`
	Description      *string           `parser:"| 'description' @String"`
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
	Pos         lexer.Position
	ID          string          `parser:"'container' @Ident"`
	Label       string          `parser:"( @String )?"`
	Description *string         `parser:"( @String )?"`
	Items       []ContainerItem `parser:"( '{' @@* '}' )?"`

	// Post-processed fields
	Components   []*Component
	DataStores   []*DataStore
	Queues       []*Queue
	Relations    []*Relation
	Metadata     []*MetaEntry // Metadata from metadata blocks
	Contracts    []*Contract
	Constraints  []*ConstraintEntry
	Conventions  []*ConventionEntry
	Version      *string
	Properties   map[string]string
	Style        map[string]string
	Scale        *ScaleBlock
	SLO          *SLOBlock
}

func (c *Container) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

// ContainerItem is a union type for items that can appear in a container.
type ContainerItem struct {
	Technology       *string           `parser:"'technology' @String"`
	Tags             []string          `parser:"| 'tags' '[' @String ( ',' @String )* ']'"`
	Version          *string           `parser:"| 'version' @String"`
	Component        *Component        `parser:"| @@"`
	DataStore        *DataStore        `parser:"| @@"`
	Queue            *Queue            `parser:"| @@"`
	Metadata         *MetadataBlock    `parser:"| @@"`
	ContractsBlock   *ContractsBlock   `parser:"| 'contracts' '{' @@ '}'"`
	ConstraintsBlock *ConstraintsBlock `parser:"| 'constraints' '{' @@ '}'"`
	ConventionsBlock *ConventionsBlock `parser:"| 'conventions' '{' @@ '}'"`
	Properties       *PropertiesBlock  `parser:"| @@"`
	Style            *StyleBlock       `parser:"| @@"`
	Scale            *ScaleBlock       `parser:"| @@"`
	SLO              *SLOBlock         `parser:"| @@"`
	Relation         *Relation         `parser:"| @@"`
	Description      *string           `parser:"| 'description' @String"`
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
	Pos         lexer.Position
	ID          string          `parser:"'component' @Ident"`
	Label       string          `parser:"( @String )?"`
	Description *string         `parser:"( @String )?"`
	Items       []ComponentItem `parser:"( '{' @@* '}' )?"`

	// Post-processed fields
	Technology   *string
	Relations    []*Relation
	Metadata     []*MetaEntry // Metadata from metadata blocks
	Properties   map[string]string
	Style        map[string]string
	Scale        *ScaleBlock
}

func (c *Component) Location() SourceLocation {
	return SourceLocation{File: c.Pos.Filename, Line: c.Pos.Line, Column: c.Pos.Column, Offset: c.Pos.Offset}
}

// ComponentItem is a union type for items that can appear in a component.
type ComponentItem struct {
    Technology       *string           `parser:"'technology' @String |"`
	Metadata         *MetadataBlock    `parser:"@@ |"`
	Behavior         *BehaviorBlock    `parser:"'behavior' '{' @@* '}' |"`
	ContractsBlock   *ContractsBlock   `parser:"'contracts' '{' @@ '}' |"`
	ConstraintsBlock *ConstraintsBlock `parser:"'constraints' '{' @@ '}' |"`
	ConventionsBlock *ConventionsBlock `parser:"'conventions' '{' @@ '}' |"`
	DependsOn        *string           `parser:"'depends_on' @Ident |"`
	Properties       *PropertiesBlock  `parser:"@@ |"`
	Style            *StyleBlock       `parser:"@@ |"`
	Scale            *ScaleBlock       `parser:"@@ |"`
    Relation         *Relation         `parser:"@@ |"`
	Description      *string           `parser:"'description' @String"`
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
	Pos         lexer.Position
	ID          string          `parser:"'datastore' @Ident"`
	Label       string          `parser:"( @String )?"`
	Description *string         `parser:"( @String )?"`
	Items       []DataStoreItem `parser:"( '{' @@* '}' )?"`

	// Post-processed fields
	Technology *string
	Relations  []*Relation
	Metadata   []*MetaEntry
	Properties map[string]string
	Style      map[string]string
}

func (d *DataStore) Location() SourceLocation {
	return SourceLocation{File: d.Pos.Filename, Line: d.Pos.Line, Column: d.Pos.Column, Offset: d.Pos.Offset}
}

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
	Pos         lexer.Position
	ID          string      `parser:"'queue' @Ident"`
	Label       string      `parser:"( @String )?"`
	Description *string     `parser:"( @String )?"`
	Items       []QueueItem `parser:"( '{' @@* '}' )?"`

	// Post-processed fields
	Technology *string
	Relations  []*Relation
	Metadata   []*MetaEntry
	Properties map[string]string
	Style      map[string]string
}

func (q *Queue) Location() SourceLocation {
	return SourceLocation{File: q.Pos.Filename, Line: q.Pos.Line, Column: q.Pos.Column, Offset: q.Pos.Offset}
}

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
	Pos   lexer.Position
	ID    string       `parser:"'person' @Ident"`
	Label string       `parser:"( @String )?"`
	Items []PersonItem `parser:"( '{' @@* '}' )?"`

	// Post-processed fields
	Description *string
	Metadata    []*MetaEntry
	Properties  map[string]string
	Style       map[string]string
}

func (p *Person) Location() SourceLocation {
	return SourceLocation{File: p.Pos.Filename, Line: p.Pos.Line, Column: p.Pos.Column, Offset: p.Pos.Offset}
}

// Element item unions for metadata blocks
type DataStoreItem struct {
	Technology  *string          `parser:"'technology' @String |"`
	Description *string          `parser:"'description' @String |"`
	Metadata    *MetadataBlock   `parser:"@@ |"`
	Properties  *PropertiesBlock `parser:"@@ |"`
	Style       *StyleBlock      `parser:"@@"`
}

type QueueItem struct {
	Technology  *string          `parser:"'technology' @String |"`
	Description *string          `parser:"'description' @String |"`
	Metadata    *MetadataBlock   `parser:"@@ |"`
	Properties  *PropertiesBlock `parser:"@@ |"`
	Style       *StyleBlock      `parser:"@@"`
}

type PersonItem struct {
	Description *string          `parser:"'description' @String |"`
	Metadata    *MetadataBlock   `parser:"@@ |"`
	Properties  *PropertiesBlock `parser:"@@ |"`
	Style       *StyleBlock      `parser:"@@"`
}

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

// MetaString returns a metadata value as a string.
func (s *System) MetaString(key string) (string, bool) {
	return metaString(s.Metadata, key)
}

// MetaString returns a metadata value as a string.
func (c *Container) MetaString(key string) (string, bool) {
	return metaString(c.Metadata, key)
}

// MetaString returns a metadata value as a string.
func (c *Component) MetaString(key string) (string, bool) {
	return metaString(c.Metadata, key)
}

// MetaString returns a metadata value as a string.
func (d *DataStore) MetaString(key string) (string, bool) {
	return metaString(d.Metadata, key)
}

// MetaString returns a metadata value as a string.
func (q *Queue) MetaString(key string) (string, bool) {
	return metaString(q.Metadata, key)
}

// MetaString returns a metadata value as a string.
func (p *Person) MetaString(key string) (string, bool) {
	return metaString(p.Metadata, key)
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

// AllMetadata returns all metadata as a map.
func (s *System) AllMetadata() map[string]string {
	return metaAll(s.Metadata)
}

// AllMetadata returns all metadata as a map.
func (c *Container) AllMetadata() map[string]string {
	return metaAll(c.Metadata)
}

// AllMetadata returns all metadata as a map.
func (c *Component) AllMetadata() map[string]string {
	return metaAll(c.Metadata)
}

// AllMetadata returns all metadata as a map.
func (d *DataStore) AllMetadata() map[string]string {
	return metaAll(d.Metadata)
}

// AllMetadata returns all metadata as a map.
func (q *Queue) AllMetadata() map[string]string {
	return metaAll(q.Metadata)
}

// AllMetadata returns all metadata as a map.
func (p *Person) AllMetadata() map[string]string {
	return metaAll(p.Metadata)
}

// MetaMap returns all metadata entries with a given prefix.
func (s *System) MetaMap(prefix string) map[string]string {
	return metaMap(s.Metadata, prefix)
}

// MetaMap returns all metadata entries with a given prefix.
func (c *Container) MetaMap(prefix string) map[string]string {
	return metaMap(c.Metadata, prefix)
}

// MetaMap returns all metadata entries with a given prefix.
func (c *Component) MetaMap(prefix string) map[string]string {
	return metaMap(c.Metadata, prefix)
}

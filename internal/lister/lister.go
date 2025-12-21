// Package lister provides pure functions for listing architecture elements.
package lister

// SystemInfo represents a system for listing.
type SystemInfo struct {
	ID          string
	Label       string
	Description string
}

// ContainerInfo represents a container for listing.
type ContainerInfo struct {
	ID          string
	Label       string
	SystemID    string
	Description string
}

// ComponentInfo represents a component for listing.
type ComponentInfo struct {
	ID          string
	Label       string
	SystemID    string
	ContainerID string
	Description string
}

// PersonInfo represents a person for listing.
type PersonInfo struct {
	ID    string
	Label string
}

// DataStoreInfo represents a datastore for listing.
type DataStoreInfo struct {
	ID       string
	Label    string
	SystemID string
}

// QueueInfo represents a queue for listing.
type QueueInfo struct {
	ID       string
	Label    string
	SystemID string
}

// ScenarioInfo represents a scenario for listing.
type ScenarioInfo struct {
	ID    string
	Title string
}

// ADRInfo represents an ADR for listing.
type ADRInfo struct {
	ID    string
	Title string
}

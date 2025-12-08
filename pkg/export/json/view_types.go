package json

// View types for extended JSON export with pre-computed views

// ViewsJSON contains all pre-computed views for the architecture
type ViewsJSON struct {
	L1 ViewData            `json:"L1"`
	L2 map[string]ViewData `json:"L2,omitempty"` // Key: systemId
	L3 map[string]ViewData `json:"L3,omitempty"` // Key: systemId.containerId
}

// ViewData contains nodes and edges for a single view
type ViewData struct {
	Nodes []ViewNode `json:"nodes"`
	Edges []ViewEdge `json:"edges"`
}

// ViewNode represents a node in a view (simplified C4 element)
type ViewNode struct {
	ID          string `json:"id"`
	Label       string `json:"label"`
	Type        string `json:"type"` // person, system, container, component, datastore, queue
	Technology  string `json:"technology,omitempty"`
	Description string `json:"description,omitempty"`
	IsExternal  bool   `json:"isExternal,omitempty"`
	ParentID    string `json:"parentId,omitempty"` // For hierarchical context
	ChildCount  int    `json:"childCount,omitempty"`
}

// ViewEdge represents an edge/relation in a view
type ViewEdge struct {
	ID     string `json:"id"`
	Source string `json:"source"`
	Target string `json:"target"`
	Label  string `json:"label,omitempty"`
}

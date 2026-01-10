package views

// Element represents a flattened architectural element for export.
// This is used as the common exchange format between the View Engine and specific exporters.
type Element struct {
	ID          string
	Kind        string // person, system, container, component, datastore, queue
	Title       string
	Technology  string
	Description string
	ParentID    string
	Width       int
	Height      int
}

// Relation represents an extracted relation for export.
type Relation struct {
	From  string
	To    string
	Label string
}

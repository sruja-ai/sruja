package json

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

// GenerateViews creates pre-computed views for all C4 levels
func GenerateViews(arch *language.Architecture) *ViewsJSON {
	views := &ViewsJSON{
		L1: generateL1View(arch),
		L2: make(map[string]ViewData),
		L3: make(map[string]ViewData),
	}

	// Generate L2 view for each system
	for _, sys := range arch.Systems {
		views.L2[sys.ID] = generateL2View(arch, sys)

		// Generate L3 view for each container in this system
		for _, cont := range sys.Containers {
			key := fmt.Sprintf("%s.%s", sys.ID, cont.ID)
			views.L3[key] = generateL3View(arch, sys, cont)
		}
	}

	return views
}

// generateL1View creates the system context view (all systems + persons)
func generateL1View(arch *language.Architecture) ViewData {
	var nodes []ViewNode
	var edges []ViewEdge
	nodeSet := make(map[string]bool)

	// Add all persons
	for _, p := range arch.Persons {
		nodes = append(nodes, ViewNode{
			ID:          p.ID,
			Label:       labelOrID(p.Label, p.ID),
			Type:        "person",
			Description: ptrStr(p.Description),
		})
		nodeSet[p.ID] = true
	}

	// Add all systems
	for _, s := range arch.Systems {
		containerCount := len(s.Containers)
		nodes = append(nodes, ViewNode{
			ID:          s.ID,
			Label:       labelOrID(s.Label, s.ID),
			Type:        "system",
			Description: ptrStr(s.Description),
			ChildCount:  containerCount,
		})
		nodeSet[s.ID] = true
	}

	// Add top-level relations (between systems and persons only)
	edgeIndex := 0
	for _, r := range arch.Relations {
		fromID := r.From.String()
		toID := r.To.String()

		// Normalize FQN to just the first part for L1
		fromID = getL1ID(fromID)
		toID = getL1ID(toID)

		if nodeSet[fromID] && nodeSet[toID] {
			edges = append(edges, ViewEdge{
				ID:     fmt.Sprintf("edge-%s-%s-%d", fromID, toID, edgeIndex),
				Source: fromID,
				Target: toID,
				Label:  ptrStr(r.Label),
			})
			edgeIndex++
		}
	}

	return ViewData{Nodes: nodes, Edges: edges}
}

// generateL2View creates container view for a specific system
func generateL2View(arch *language.Architecture, focusedSystem *language.System) ViewData {
	var nodes []ViewNode
	var edges []ViewEdge
	nodeSet := make(map[string]bool)

	// Add containers, datastores, queues from focused system
	for _, c := range focusedSystem.Containers {
		nodes = append(nodes, ViewNode{
			ID:          c.ID,
			Label:       labelOrID(c.Label, c.ID),
			Type:        "container",
			Technology:  getContainerTechnology(c),
			Description: ptrStr(c.Description),
			ParentID:    focusedSystem.ID,
			ChildCount:  len(c.Components),
		})
		nodeSet[c.ID] = true
	}

	for _, ds := range focusedSystem.DataStores {
		nodes = append(nodes, ViewNode{
			ID:       ds.ID,
			Label:    labelOrID(ds.Label, ds.ID),
			Type:     "datastore",
			ParentID: focusedSystem.ID,
		})
		nodeSet[ds.ID] = true
	}

	for _, q := range focusedSystem.Queues {
		nodes = append(nodes, ViewNode{
			ID:       q.ID,
			Label:    labelOrID(q.Label, q.ID),
			Type:     "queue",
			ParentID: focusedSystem.ID,
		})
		nodeSet[q.ID] = true
	}

	// Find connected external entities
	externalNodes := findConnectedExternalL2(arch, focusedSystem, nodeSet)
	nodes = append(nodes, externalNodes...)
	for _, n := range externalNodes {
		nodeSet[n.ID] = true
	}

	// Add relations involving nodes in this view
	edges = collectL2Edges(arch, focusedSystem, nodeSet)

	return ViewData{Nodes: nodes, Edges: edges}
}

// generateL3View creates component view for a specific container
func generateL3View(_ *language.Architecture, _ *language.System, cont *language.Container) ViewData {
	var nodes []ViewNode
	var edges []ViewEdge
	nodeSet := make(map[string]bool)

	// Add components from the container
	for _, comp := range cont.Components {
		nodes = append(nodes, ViewNode{
			ID:          comp.ID,
			Label:       labelOrID(comp.Label, comp.ID),
			Type:        "component",
			Technology:  ptrStr(comp.Technology),
			Description: ptrStr(comp.Description),
			ParentID:    cont.ID,
		})
		nodeSet[comp.ID] = true
	}

	// Add container's internal datastores and queues
	for _, ds := range cont.DataStores {
		nodes = append(nodes, ViewNode{
			ID:       ds.ID,
			Label:    labelOrID(ds.Label, ds.ID),
			Type:     "datastore",
			ParentID: cont.ID,
		})
		nodeSet[ds.ID] = true
	}

	for _, q := range cont.Queues {
		nodes = append(nodes, ViewNode{
			ID:       q.ID,
			Label:    labelOrID(q.Label, q.ID),
			Type:     "queue",
			ParentID: cont.ID,
		})
		nodeSet[q.ID] = true
	}

	// Collect component-level relations within container
	edgeIndex := 0
	for _, r := range cont.Relations {
		fromID := getLastPart(r.From.String())
		toID := getLastPart(r.To.String())

		if nodeSet[fromID] && nodeSet[toID] {
			edges = append(edges, ViewEdge{
				ID:     fmt.Sprintf("edge-%s-%s-%d", fromID, toID, edgeIndex),
				Source: fromID,
				Target: toID,
				Label:  ptrStr(r.Label),
			})
			edgeIndex++
		}
	}

	return ViewData{Nodes: nodes, Edges: edges}
}

// findConnectedExternalL2 finds systems/persons connected to the focused system
func findConnectedExternalL2(arch *language.Architecture, focusedSystem *language.System, internalNodes map[string]bool) []ViewNode {
	var external []ViewNode
	seen := make(map[string]bool)

	// Check all relations in architecture
	for _, r := range arch.Relations {
		fromID := getLastPart(r.From.String())
		toID := getLastPart(r.To.String())

		// If one end is internal and other is external, add external
		if internalNodes[fromID] && !internalNodes[toID] && !seen[toID] {
			if node := findNodeByIDInArch(arch, toID); node != nil {
				node.IsExternal = true
				external = append(external, *node)
				seen[toID] = true
			}
		}
		if internalNodes[toID] && !internalNodes[fromID] && !seen[fromID] {
			if node := findNodeByIDInArch(arch, fromID); node != nil {
				node.IsExternal = true
				external = append(external, *node)
				seen[fromID] = true
			}
		}
	}

	// Also check system-level relations
	for _, r := range focusedSystem.Relations {
		toID := getLastPart(r.To.String())

		if !internalNodes[toID] && !seen[toID] {
			if node := findNodeByIDInArch(arch, toID); node != nil {
				node.IsExternal = true
				external = append(external, *node)
				seen[toID] = true
			}
		}
	}

	return external
}

// collectL2Edges collects edges relevant to a system view
func collectL2Edges(arch *language.Architecture, sys *language.System, nodeSet map[string]bool) []ViewEdge {
	var edges []ViewEdge
	edgeIndex := 0

	// System-level relations
	for _, r := range sys.Relations {
		fromID := getLastPart(r.From.String())
		toID := getLastPart(r.To.String())

		if nodeSet[fromID] && nodeSet[toID] {
			edges = append(edges, ViewEdge{
				ID:     fmt.Sprintf("edge-%s-%s-%d", fromID, toID, edgeIndex),
				Source: fromID,
				Target: toID,
				Label:  ptrStr(r.Label),
			})
			edgeIndex++
		}
	}

	// Architecture-level relations involving this system's nodes
	for _, r := range arch.Relations {
		fromID := getLastPart(r.From.String())
		toID := getLastPart(r.To.String())

		if nodeSet[fromID] && nodeSet[toID] {
			edges = append(edges, ViewEdge{
				ID:     fmt.Sprintf("edge-%s-%s-%d", fromID, toID, edgeIndex),
				Source: fromID,
				Target: toID,
				Label:  ptrStr(r.Label),
			})
			edgeIndex++
		}
	}

	return edges
}

// findNodeByIDInArch looks up a node by ID across the architecture
func findNodeByIDInArch(arch *language.Architecture, id string) *ViewNode {
	// Check persons
	for _, p := range arch.Persons {
		if p.ID == id {
			return &ViewNode{
				ID:          p.ID,
				Label:       labelOrID(p.Label, p.ID),
				Type:        "person",
				Description: ptrStr(p.Description),
			}
		}
	}

	// Check systems
	for _, s := range arch.Systems {
		if s.ID == id {
			return &ViewNode{
				ID:          s.ID,
				Label:       labelOrID(s.Label, s.ID),
				Type:        "system",
				Description: ptrStr(s.Description),
			}
		}

		// Check containers
		for _, c := range s.Containers {
			if c.ID == id {
				return &ViewNode{
					ID:         c.ID,
					Label:      labelOrID(c.Label, c.ID),
					Type:       "container",
					Technology: getContainerTechnology(c),
				}
			}
		}

		// Check datastores
		for _, ds := range s.DataStores {
			if ds.ID == id {
				return &ViewNode{
					ID:    ds.ID,
					Label: labelOrID(ds.Label, ds.ID),
					Type:  "datastore",
				}
			}
		}

		// Check queues
		for _, q := range s.Queues {
			if q.ID == id {
				return &ViewNode{
					ID:    q.ID,
					Label: labelOrID(q.Label, q.ID),
					Type:  "queue",
				}
			}
		}
	}

	return nil
}

// Helper functions

func labelOrID(label string, id string) string {
	if label != "" {
		return label
	}
	return id
}

func ptrStr(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

func getContainerTechnology(c *language.Container) string {
	// Technology is stored in Items
	for _, item := range c.Items {
		if item.Technology != nil {
			return *item.Technology
		}
	}
	return ""
}

func getLastPart(id string) string {
	for i := len(id) - 1; i >= 0; i-- {
		if id[i] == '.' {
			return id[i+1:]
		}
	}
	return id
}

func getL1ID(id string) string {
	// For L1 view, we want the first part of FQN (system ID)
	for i := 0; i < len(id); i++ {
		if id[i] == '.' {
			return id[:i]
		}
	}
	return id
}

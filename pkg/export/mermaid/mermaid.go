package mermaid

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

// Mermaid styles
const (
	PersonStyle    = "fill:#ffcccc,stroke:#333,stroke-width:2px,color:#000"
	SystemStyle    = "fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000"
	ContainerStyle = "fill:#cce5ff,stroke:#333,stroke-width:2px,color:#000"
	DatabaseStyle  = "fill:#ccffcc,stroke:#333,stroke-width:2px,color:#000"
	QueueStyle     = "fill:#ffe5cc,stroke:#333,stroke-width:2px,color:#000"
	ExternalStyle  = "fill:#eeeeee,stroke:#666,stroke-width:2px,color:#000,stroke-dasharray: 3 3"
	ComponentStyle = "fill:#e6f7ff,stroke:#333,stroke-width:2px,color:#000"
)

// Config represents Mermaid diagram configuration.
type Config struct {
	Layout         string
	Theme          string
	Look           string
	Direction      string
	UseFrontmatter bool
}

// DefaultConfig returns the default Mermaid configuration.
func DefaultConfig() Config {
	return Config{
		Layout:    "elk",
		Theme:     "default",
		Direction: "LR",
	}
}

// Exporter handles Mermaid diagram generation.
type Exporter struct {
	Config Config
}

// NewExporter creates a new Mermaid exporter.
func NewExporter(config Config) *Exporter {
	return &Exporter{Config: config}
}

// Export generates a Mermaid diagram from a program.
func (e *Exporter) Export(prog *language.Program) string {
	if prog == nil || prog.Model == nil {
		return ""
	}

	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	e.writeHeader(sb)
	e.writeStyles(sb)

	// Extract elements from Model
	systems := extractSystemsFromModel(prog)
	persons := extractPersonsFromModel(prog)
	containers := extractTopLevelContainers(prog)
	relations := extractRelationsFromModel(prog)

	// Index things for easier lookup
	indexedArch := indexProgram(prog)

	// Render persons
	for _, p := range persons {
		e.writePerson(sb, p)
	}

	// Render systems
	for _, sys := range systems {
		e.writeSystem(sb, sys, indexedArch)
	}

	// Render standalone containers (not in a system)
	for _, cont := range containers {
		e.writeContainer(sb, cont, "", "    ")
	}

	// Render relations
	for _, rel := range relations {
		e.writeRelation(sb, rel, indexedArch)
	}

	return sb.String()
}

// Helper functions to extract elements from Model
func extractSystemsFromModel(prog *language.Program) []*language.System {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var systems []*language.System
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "system" {
			if sys := extractSystemFromElement(item.ElementDef); sys != nil {
				systems = append(systems, sys)
			}
		}
	}
	return systems
}

func extractSystemFromElement(elem *language.LikeC4ElementDef) *language.System {
	if elem == nil || elem.GetKind() != "system" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	sys := &language.System{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
	body := elem.GetBody()
	if body != nil {
		for _, bodyItem := range body.Items {
			if bodyItem.Element != nil {
				if cont := extractContainerFromElement(bodyItem.Element); cont != nil {
					sys.Containers = append(sys.Containers, cont)
				}
				if ds := extractDataStoreFromElement(bodyItem.Element); ds != nil {
					sys.DataStores = append(sys.DataStores, ds)
				}
				if q := extractQueueFromElement(bodyItem.Element); q != nil {
					sys.Queues = append(sys.Queues, q)
				}
			}
		}
	}
	return sys
}

func extractContainerFromElement(elem *language.LikeC4ElementDef) *language.Container {
	if elem == nil || elem.GetKind() != "container" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	cont := &language.Container{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
	body := elem.GetBody()
	if body != nil {
		for _, bodyItem := range body.Items {
			if bodyItem.Element != nil {
				if comp := extractComponentFromElement(bodyItem.Element); comp != nil {
					cont.Components = append(cont.Components, comp)
				}
			}
		}
	}
	return cont
}

func extractComponentFromElement(elem *language.LikeC4ElementDef) *language.Component {
	if elem == nil || elem.GetKind() != "component" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	return &language.Component{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
}

func extractDataStoreFromElement(elem *language.LikeC4ElementDef) *language.DataStore {
	kind := ""
	if elem != nil {
		kind = elem.GetKind()
	}
	if elem == nil || (kind != "database" && kind != "datastore") {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	return &language.DataStore{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
}

func extractQueueFromElement(elem *language.LikeC4ElementDef) *language.Queue {
	if elem == nil || elem.GetKind() != "queue" {
		return nil
	}
	id := elem.GetID()
	if id == "" {
		return nil
	}
	return &language.Queue{
		ID:    id,
		Label: getString(elem.GetTitle()),
	}
}

func extractPersonsFromModel(prog *language.Program) []*language.Person {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var persons []*language.Person
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "person" {
			id := item.ElementDef.GetID()
			if id != "" {
				persons = append(persons, &language.Person{
					ID:    id,
					Label: getString(item.ElementDef.GetTitle()),
				})
			}
		}
	}
	return persons
}

func extractTopLevelContainers(prog *language.Program) []*language.Container {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var containers []*language.Container
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "container" {
			if cont := extractContainerFromElement(item.ElementDef); cont != nil {
				containers = append(containers, cont)
			}
		}
	}
	return containers
}

func extractRelationsFromModel(prog *language.Program) []*language.Relation {
	if prog == nil || prog.Model == nil {
		return nil
	}
	var relations []*language.Relation
	for _, item := range prog.Model.Items {
		if item.Relation != nil {
			relations = append(relations, item.Relation)
		}
	}
	return relations
}

func (e *Exporter) writeHeader(sb *strings.Builder) {
	if e.Config.UseFrontmatter {
		sb.WriteString("---\nconfig:\n")
		if e.Config.Layout != "" {
			fmt.Fprintf(sb, "  layout: %s\n", e.Config.Layout)
		}
		if e.Config.Theme != "" && e.Config.Theme != "default" {
			fmt.Fprintf(sb, "  theme: %s\n", e.Config.Theme)
		}
		if e.Config.Direction != "" {
			fmt.Fprintf(sb, "  direction: %s\n", strings.ToLower(e.Config.Direction))
		}
		sb.WriteString("---\n")
	} else {
		theme := e.Config.Theme
		if theme == "" {
			theme = "default"
		}
		fmt.Fprintf(sb, "%%%%{init: { \"theme\": \"%s\", \"flowchart\": { \"htmlLabels\": true } }}%%%%\n", theme)
	}

	dir := e.Config.Direction
	if dir == "" {
		dir = "LR"
	}
	fmt.Fprintf(sb, "graph %s\n\n", dir)
}

func (e *Exporter) writeStyles(sb *strings.Builder) {
	fmt.Fprintf(sb, "    classDef personStyle %s\n", PersonStyle)
	fmt.Fprintf(sb, "    classDef systemStyle %s\n", SystemStyle)
	fmt.Fprintf(sb, "    classDef containerStyle %s\n", ContainerStyle)
	fmt.Fprintf(sb, "    classDef databaseStyle %s\n", DatabaseStyle)
	fmt.Fprintf(sb, "    classDef queueStyle %s\n", QueueStyle)
	fmt.Fprintf(sb, "    classDef externalStyle %s\n", ExternalStyle)
	fmt.Fprintf(sb, "    classDef componentStyle %s\n\n", ComponentStyle)
}

func (e *Exporter) writePerson(sb *strings.Builder, p *language.Person) {
	id := sanitizeID(p.ID)
	label := escapeQuotes(formatLabel(p.Label, p.ID, getString(p.Description), ""))
	fmt.Fprintf(sb, "    %s[\"%s\"]\n", id, label)
	fmt.Fprintf(sb, "    class %s personStyle\n", id)
}

func (e *Exporter) writeSystem(sb *strings.Builder, sys *language.System, _ *indexedArchitecture) {
	id := sanitizeID(sys.ID)

	// If system has containers, use subgraph
	if len(sys.Containers) > 0 || len(sys.DataStores) > 0 || len(sys.Queues) > 0 {
		label := escapeQuotes(sys.Label)
		if label == "" {
			label = sys.ID
		}
		fmt.Fprintf(sb, "    subgraph %s[\"%s\"]\n", id, label)
		for _, cont := range sys.Containers {
			e.writeContainer(sb, cont, sys.ID, "        ")
		}
		for _, ds := range sys.DataStores {
			e.writeDataStore(sb, ds, sys.ID, "        ")
		}
		for _, q := range sys.Queues {
			e.writeQueue(sb, q, sys.ID, "        ")
		}
		sb.WriteString("    end\n")
	} else {
		label := escapeQuotes(formatLabel(sys.Label, sys.ID, getString(sys.Description), ""))
		fmt.Fprintf(sb, "    %s[\"%s\"]\n", id, label)
		fmt.Fprintf(sb, "    class %s systemStyle\n", id)
	}
}

func (e *Exporter) writeContainer(sb *strings.Builder, cont *language.Container, parentID string, indent string) {
	fullID := cont.ID
	if parentID != "" && !strings.Contains(cont.ID, parentID) {
		fullID = parentID + "." + cont.ID
	}
	id := sanitizeID(fullID)

	// Container does not have direct Technology field in AST element struct, but we can look for it
	tech := ""
	if t, ok := cont.Properties["technology"]; ok {
		tech = t
	}
	label := escapeQuotes(formatLabel(cont.Label, cont.ID, getString(cont.Description), tech))

	// If container has components, use subgraph
	if len(cont.Components) > 0 {
		fmt.Fprintf(sb, "%ssubgraph %s[\"%s\"]\n", indent, id, label)
		for _, comp := range cont.Components {
			e.writeComponent(sb, comp, fullID, indent+"    ")
		}
		fmt.Fprintf(sb, "%send\n", indent)
	} else {
		fmt.Fprintf(sb, "%s%s[\"%s\"]\n", indent, id, label)
		fmt.Fprintf(sb, "%sclass %s containerStyle\n", indent, id)
	}
}

func (e *Exporter) writeDataStore(sb *strings.Builder, ds *language.DataStore, parentID string, indent string) {
	fullID := ds.ID
	if parentID != "" && !strings.Contains(ds.ID, parentID) {
		fullID = parentID + "." + ds.ID
	}
	id := sanitizeID(fullID)
	label := escapeQuotes(formatLabel(ds.Label, ds.ID, getString(ds.Description), getString(ds.Technology)))
	fmt.Fprintf(sb, "%s%s[(\"%s\")]\n", indent, id, label)
	fmt.Fprintf(sb, "%sclass %s databaseStyle\n", indent, id)
}

func (e *Exporter) writeQueue(sb *strings.Builder, q *language.Queue, parentID string, indent string) {
	fullID := q.ID
	if parentID != "" && !strings.Contains(q.ID, parentID) {
		fullID = parentID + "." + q.ID
	}
	id := sanitizeID(fullID)
	label := escapeQuotes(formatLabel(q.Label, q.ID, getString(q.Description), getString(q.Technology)))
	fmt.Fprintf(sb, "%s%s(\"%s\")\n", indent, id, label)
	fmt.Fprintf(sb, "%sclass %s queueStyle\n", indent, id)
}

func (e *Exporter) writeComponent(sb *strings.Builder, comp *language.Component, parentID string, indent string) {
	fullID := comp.ID
	if parentID != "" && !strings.Contains(comp.ID, parentID) {
		fullID = parentID + "." + comp.ID
	}
	id := sanitizeID(fullID)
	label := escapeQuotes(formatLabel(comp.Label, comp.ID, getString(comp.Description), getString(comp.Technology)))
	fmt.Fprintf(sb, "%s%s[\"%s\"]\n", indent, id, label)
	fmt.Fprintf(sb, "%sclass %s componentStyle\n", indent, id)
}

func (e *Exporter) writeRelation(sb *strings.Builder, rel *language.Relation, _ *indexedArchitecture) {
	from := sanitizeID(rel.From.String())
	to := sanitizeID(rel.To.String())

	label := getString(rel.Label)
	if label == "" {
		label = getString(rel.Verb)
	}

	if label != "" {
		fmt.Fprintf(sb, "    %s -->|\"%s\"| %s\n", from, escapeQuotes(label), to)
	} else {
		fmt.Fprintf(sb, "    %s --> %s\n", from, to)
	}
}

// Helpers

func getString(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

func sanitizeID(id string) string {
	return strings.Map(func(r rune) rune {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') {
			return r
		}
		return '_'
	}, id)
}

func escapeQuotes(s string) string {
	return strings.ReplaceAll(s, "\"", "#quot;")
}

func formatLabel(label, id, description, technology string) string {
	main := label
	if main == "" {
		main = id
	}

	res := main
	if technology != "" {
		res += fmt.Sprintf("\n(%s)", technology)
	}
	if description != "" {
		// Truncate description for Mermaid
		desc := description
		if len(desc) > 50 {
			desc = desc[:47] + "..."
		}
		res += fmt.Sprintf("\n%s", desc)
	}
	return res
}

func indexProgram(prog *language.Program) *indexedArchitecture {
	idx := &indexedArchitecture{
		ids: make(map[string]bool),
	}
	// Index all element IDs from the model
	if prog != nil && prog.Model != nil {
		var indexElement func(elem *language.LikeC4ElementDef, prefix string)
		indexElement = func(elem *language.LikeC4ElementDef, prefix string) {
			if elem == nil {
				return
			}
			id := elem.GetID()
			if id != "" {
				fullID := id
				if prefix != "" {
					fullID = prefix + "." + id
				}
				idx.ids[fullID] = true
				body := elem.GetBody()
				if body != nil {
					for _, bodyItem := range body.Items {
						if bodyItem.Element != nil {
							indexElement(bodyItem.Element, fullID)
						}
					}
				}
			}
		}
		for _, item := range prog.Model.Items {
			if item.ElementDef != nil {
				indexElement(item.ElementDef, "")
			}
		}
	}
	return idx
}

type indexedArchitecture struct {
	ids map[string]bool
}

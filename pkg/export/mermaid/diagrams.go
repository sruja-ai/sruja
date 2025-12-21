package mermaid

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

// GenerateL1 generates a System Context Diagram (Level 1).
// Shows Systems and Persons, and relations between them.
func (e *Exporter) GenerateL1(prog *language.Program) string {
	if prog == nil || prog.Model == nil {
		return ""
	}

	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	// Use specific direction for L1 if needed, or default
	e.writeHeader(sb)
	e.writeStyles(sb)

	persons := extractPersonsFromModel(prog)
	systems := extractSystemsFromModel(prog)
	relations := extractRelationsFromModel(prog)
	indexedArch := indexProgram(prog)

	// Write Persons
	for _, p := range persons {
		e.writePerson(sb, p)
	}

	// Write Systems (without internals)
	for _, sys := range systems {
		id := sanitizeID(sys.ID)
		label := escapeQuotes(formatLabel(sys.Label, sys.ID, getString(sys.Description), ""))
		sb.WriteString(strings.Repeat(" ", 4))
		sb.WriteString(id)
		sb.WriteString("[\"")
		sb.WriteString(label)
		sb.WriteString("\"]\n")
		sb.WriteString(strings.Repeat(" ", 4))
		sb.WriteString("class ")
		sb.WriteString(id)
		sb.WriteString(" systemStyle\n")
	}

	// Write Relations (only those between L1 elements)
	// Filter relations where both source and target are in our lists
	l1Ids := make(map[string]bool)
	for _, p := range persons {
		l1Ids[p.ID] = true
	}
	for _, s := range systems {
		l1Ids[s.ID] = true
	}

	for _, rel := range relations {
		from := rel.From.String()
		to := rel.To.String()
		// If directly connected to L1 elements
		if l1Ids[from] && l1Ids[to] {
			e.writeRelation(sb, rel, indexedArch)
		}
	}

	return sb.String()
}

// GenerateL2 generates a Container Diagram (Level 2) for a specific System.
func (e *Exporter) GenerateL2(sys *language.System, prog *language.Program) string {
	if sys == nil {
		return ""
	}

	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	e.writeHeader(sb)
	e.writeStyles(sb)

	indexedArch := indexProgram(prog)

	// Draw the System Boundary
	id := sanitizeID(sys.ID)
	label := escapeQuotes(sys.Label)
	if label == "" {
		label = sys.ID
	}
	sb.WriteString("    subgraph ")
	sb.WriteString(id)
	sb.WriteString("[\"")
	sb.WriteString(label)
	sb.WriteString("\"]\n")
	sb.WriteString("    direction TB\n") // Containers usually look better TB inside system

	// Write Containers
	for _, cont := range sys.Containers {
		e.writeContainer(sb, cont, sys.ID, "        ")
	}
	// Write DataStores
	for _, ds := range sys.DataStores {
		e.writeDataStore(sb, ds, sys.ID, "        ")
	}
	// Write Queues
	for _, q := range sys.Queues {
		e.writeQueue(sb, q, sys.ID, "        ")
	}
	sb.WriteString("    end\n")

	// Filter and Write Relations involving these elements
	// We want relations:
	// 1. Between containers/datastores/queues of THIS system
	// 2. From external persons/systems TO these elements
	// 3. From these elements TO external persons/systems

	// Collect IDs of internal elements
	internalIDs := make(map[string]bool)
	for _, c := range sys.Containers {
		internalIDs[c.ID] = true
	}
	for _, d := range sys.DataStores {
		internalIDs[d.ID] = true
	}
	for _, q := range sys.Queues {
		internalIDs[q.ID] = true
	}

	allRelations := extractRelationsFromModel(prog)
	for _, rel := range allRelations {
		from := rel.From.String()
		to := rel.To.String()

		// Full IDs might be relative or absolute, need to check if they Match
		// The extractors return "simple" IDs inside the struct, but relations use FQN.
		// For verification simplification, we assume IDs in the model are unique or we align them.

		// Adjust: In lister/extractors, we might have stored just the ID, not FQN.
		// But extractRelationsFromModel likely returns FQNs from AST.
		// Let's assume FQN for safety.

		fromSysPrefix := sys.ID + "."
		toSysPrefix := sys.ID + "."

		isFromInternal := from == sys.ID || strings.HasPrefix(from, fromSysPrefix)
		isToInternal := to == sys.ID || strings.HasPrefix(to, toSysPrefix)

		// Specific check for containers: their ID inside struct is usually simple ID, need to prefix
		// Actually the extractor sets ID to simple ID.
		// We need to match against what writeRelation expects.

		// If internal-to-internal
		if isFromInternal && isToInternal {
			// Don't show system-to-system self relations unless meaningful
			if from != sys.ID || to != sys.ID {
				e.writeRelation(sb, rel, indexedArch)
			}
		} else if isFromInternal || isToInternal {
			// Incoming or Outgoing
			// For external elements, we might want to render them if we want a complete C4 Container Diagram context.
			// mermaid/writers.go handles rendering nodes. If we just write a relation to a node that hasn't been drawn, Mermaid might draw a simple box.
			// Ideally we draw external Person/System nodes.

			// Let's draw the external node if it's not drawn.
			// Find external element and draw it (if not already drawn implicitly by styling)
			// But specialized styling won't apply unless we define the class.
			// For simplicty in recovery, we'll let mermaid draw default nodes for externals,
			// OR we could look them up.

			// Find external element and draw it (if not already drawn implicitly by styling)
			// But specialized styling won't apply unless we define the class.
			// For simplicty in recovery, we'll let mermaid draw default nodes for externals,
			// OR we could look them up.

			e.writeRelation(sb, rel, indexedArch)
		}
	}

	return sb.String()
}

// GenerateL3 generates a Component Diagram (Level 3) for a specific Container.
func (e *Exporter) GenerateL3(cont *language.Container, systemID string, prog *language.Program) string {
	if cont == nil {
		return ""
	}

	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	e.writeHeader(sb)
	e.writeStyles(sb)

	indexedArch := indexProgram(prog)

	fullContID := cont.ID
	if systemID != "" && !strings.Contains(cont.ID, systemID) {
		fullContID = systemID + "." + cont.ID
	}

	// Draw Container Boundary
	id := sanitizeID(fullContID)
	label := escapeQuotes(cont.Label)
	if label == "" {
		label = cont.ID
	}
	sb.WriteString("    subgraph ")
	sb.WriteString(id)
	sb.WriteString("[\"")
	sb.WriteString(label)
	sb.WriteString("\"]\n")
	sb.WriteString("    direction TB\n")

	// Write Components
	for _, comp := range cont.Components {
		e.writeComponent(sb, comp, fullContID, "        ")
	}
	sb.WriteString("    end\n")

	// Write Relations (Internal components)
	// Similar logic to L2, filter for components inside this container
	compIDs := make(map[string]bool)
	prefix := fullContID + "."
	for _, c := range cont.Components {
		compIDs[prefix+c.ID] = true // Components usually possess simple IDs in the struct
	}

	allRelations := extractRelationsFromModel(prog)
	for _, rel := range allRelations {
		from := rel.From.String()
		to := rel.To.String()

		if strings.HasPrefix(from, prefix) && strings.HasPrefix(to, prefix) {
			e.writeRelation(sb, rel, indexedArch)
		}
	}

	return sb.String()
}

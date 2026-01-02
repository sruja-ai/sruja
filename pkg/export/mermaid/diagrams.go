package mermaid

import (
	"fmt"
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
		sb.WriteString(Indent4)
		sb.WriteString(id)
		sb.WriteString("[\"")
		sb.WriteString(label)
		sb.WriteString("\"]\n")
		sb.WriteString(Indent4)
		sb.WriteString("class ")
		sb.WriteString(id)
		sb.WriteString(" ")
		sb.WriteString(ClassSystem)
		sb.WriteString("\n")
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

	// indexedArch := indexProgram(prog) // Unused in custom L2 generation
	lookup := buildElementLookup(prog)

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
	sb.WriteString("    direction TB\n")

	// Write Containers
	for _, cont := range sys.Containers {
		// Use manual writing to avoid recursing into components (like writeContainer does)
		fullID := cont.ID
		if sys.ID != "" && !strings.Contains(cont.ID, sys.ID) {
			fullID = sys.ID + "." + cont.ID
		}
		id := sanitizeID(fullID)

		// Determine technology
		tech := ""
		for _, item := range cont.Items {
			if item.Technology != nil {
				tech = *item.Technology
				break
			}
		}

		label := escapeQuotes(formatLabel(cont.Label, cont.ID, getString(cont.Description), tech))

		fmt.Fprintf(sb, "        %s[\"%s\"]\n", id, label)
		fmt.Fprintf(sb, "        class %s %s\n", id, ClassContainer)
	}
	// Write DataStores
	for _, ds := range sys.DataStores {
		e.writeDataStore(sb, ds, sys.ID, Indent8)
	}
	// Write Queues
	for _, q := range sys.Queues {
		e.writeQueue(sb, q, sys.ID, Indent8)
	}
	sb.WriteString("    end\n")

	// Relations & External Context
	// Strategy:
	// 1. Internal -> Internal: Draw detailed
	// 2. Internal -> External: Collapse external to its Root (System/Person)
	// 3. External -> Internal: Collapse external to its Root

	internalPrefix := sys.ID + "."
	isInternal := func(fqn string) bool {
		return fqn == sys.ID || strings.HasPrefix(fqn, internalPrefix)
	}

	externalNodes := make(map[string]*elementInfo)
	renderedRelations := make(map[string]bool)

	allRelations := extractRelationsFromModel(prog)
	for _, rel := range allRelations {
		from := rel.From.String()
		to := rel.To.String()

		fromInternal := isInternal(from)
		toInternal := isInternal(to)

		if !fromInternal && !toInternal {
			continue // Skip completely external relations
		}

		// Resolve effective nodes
		var effectiveFrom, effectiveTo string

		if fromInternal {
			effectiveFrom = from
		} else {
			rootID, _ := lookup.getRoot(from)
			effectiveFrom = rootID
			if info, ok := lookup.elements[rootID]; ok {
				externalNodes[rootID] = info
			}
		}

		if toInternal {
			effectiveTo = to
		} else {
			rootID, _ := lookup.getRoot(to)
			effectiveTo = rootID
			if info, ok := lookup.elements[rootID]; ok {
				externalNodes[rootID] = info
			}
		}

		// Prevent self-loops on the system boundary if they effectively collapse to same thing (unlikely here but possible)
		if effectiveFrom == effectiveTo {
			continue
		}

		// Dedup key
		relKey := effectiveFrom + "->" + effectiveTo + ":" + getString(rel.Label)
		if renderedRelations[relKey] {
			continue
		}
		renderedRelations[relKey] = true

		// Write Relation (using effective IDs)
		// We treat 'rel' as template but use computed IDs
		e.writeCustomRelation(sb, effectiveFrom, effectiveTo, rel)
	}

	// Write External Nodes
	for id, info := range externalNodes {
		// If it's the system itself (shouldn't happen due to isInternal check, but safety)
		if id == sys.ID {
			continue
		}

		sanitized := sanitizeID(id)
		label := escapeQuotes(info.Label)
		if label == "" {
			label = id
		}

		if info.Kind == "person" || info.Kind == "Person" {
			fmt.Fprintf(sb, "    %s[\"%s\"]\n", sanitized, label)
			fmt.Fprintf(sb, "    class %s %s\n", sanitized, ClassPerson)
		} else {
			// System
			fmt.Fprintf(sb, "    %s[\"%s\"]\n", sanitized, label)
			fmt.Fprintf(sb, "    class %s %s\n", sanitized, ClassSystem)
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

	_ = indexProgram(prog) // Keep for compatibility if needed, though we rely on lookup
	lookup := buildElementLookup(prog)

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
		e.writeComponent(sb, comp, fullContID, Indent8)
	}
	sb.WriteString("    end\n")

	// Relations & Context
	// Strategy:
	// 1. Component -> Component (same container): Keep
	// 2. Component -> sibling Container: Collapse to Container
	// 3. Component -> External System: Collapse to System

	internalPrefix := fullContID + "."
	isInternal := func(fqn string) bool {
		return strings.HasPrefix(fqn, internalPrefix)
	}

	externalNodes := make(map[string]*elementInfo)
	renderedRelations := make(map[string]bool)

	allRelations := extractRelationsFromModel(prog)
	for _, rel := range allRelations {
		from := rel.From.String()
		to := rel.To.String()

		fromInternal := isInternal(from)
		toInternal := isInternal(to)

		if !fromInternal && !toInternal {
			continue
		}

		var effectiveFrom, effectiveTo string

		resolveNode := func(fqn string) string {
			if isInternal(fqn) {
				return fqn
			}
			// Is it in the same system?
			if systemID != "" && strings.HasPrefix(fqn, systemID+".") {
				// Yes, finding containing container
				// Try to find container
				contID := lookup.getContainer(fqn)
				if contID != "" {
					return contID
				}
				// Fallback to whatever found
				return fqn
			}
			// External to system -> System/Person Root
			root, _ := lookup.getRoot(fqn)
			return root
		}

		effectiveFrom = resolveNode(from)
		effectiveTo = resolveNode(to)

		// track externals
		if !isInternal(from) {
			if info, ok := lookup.elements[effectiveFrom]; ok {
				externalNodes[effectiveFrom] = info
			}
		}
		if !isInternal(to) {
			if info, ok := lookup.elements[effectiveTo]; ok {
				externalNodes[effectiveTo] = info
			}
		}

		if effectiveFrom == effectiveTo {
			continue
		}

		relKey := effectiveFrom + "->" + effectiveTo + ":" + getString(rel.Label)
		if renderedRelations[relKey] {
			continue
		}
		renderedRelations[relKey] = true

		e.writeCustomRelation(sb, effectiveFrom, effectiveTo, rel)
	}

	// Write External Nodes
	for id, info := range externalNodes {
		if id == fullContID {
			continue
		}

		sanitized := sanitizeID(id)
		label := escapeQuotes(info.Label)
		if label == "" {
			label = id
		}

		kind := info.Kind
		if kind == "container" || kind == "Container" {
			fmt.Fprintf(sb, "    %s[\"%s\"]\n", sanitized, label)
			fmt.Fprintf(sb, "    class %s %s\n", sanitized, ClassContainer)
		} else if kind == "person" || kind == "Person" {
			fmt.Fprintf(sb, "    %s[\"%s\"]\n", sanitized, label)
			fmt.Fprintf(sb, "    class %s %s\n", sanitized, ClassPerson)
		} else {
			// System mainly
			fmt.Fprintf(sb, "    %s[\"%s\"]\n", sanitized, label)
			fmt.Fprintf(sb, "    class %s %s\n", sanitized, ClassSystem)
		}
	}

	return sb.String()
}

func (e *Exporter) writeCustomRelation(sb *strings.Builder, from, to string, rel *language.Relation) {
	sFrom := sanitizeID(from)
	sTo := sanitizeID(to)

	label := getString(rel.Label)
	if label == "" {
		label = getString(rel.Verb)
	}

	if label != "" {
		fmt.Fprintf(sb, "    %s -->|\"%s\"| %s\n", sFrom, escapeQuotes(label), sTo)
	} else {
		fmt.Fprintf(sb, "    %s --> %s\n", sFrom, sTo)
	}
}

package engine

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// DatabaseIsolationRule enforces that databases are not shared between different services/systems
// unless explicitly marked as shared.
// Anti-pattern: Integration via Database.
type DatabaseIsolationRule struct{}

func (r *DatabaseIsolationRule) Name() string {
	return "Database Isolation"
}

func (r *DatabaseIsolationRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic
	if program == nil || program.Model == nil {
		return diags
	}

	// Collect all relations from Model
	_, relations := collectElements(program.Model)

	// 1. Identify all DataStores
	// Map: DataStore ID -> List of Consumers (System/Container/Component IDs)
	dsUsage := make(map[string][]string, 8)

	rels := relations

	for _, rel := range rels {
		// potential datastore targets
		targetName := rel.To.String()

		// Check if target is a datastore
		// We need to resolve the name to a type.
		// Since we don't have a full symbol table index here easily, we iterate to check.
		isDatastore := isTargetDatastore(program, targetName)

		if isDatastore {
			// Who is calling it?
			sourceName := rel.From.String()
			// Get root component of source (e.g. if A.B calls DB, source is A)
			// Use IndexByte for better performance than Split when we only need first part
			firstDot := strings.IndexByte(sourceName, '.')
			var sourceRoot string
			if firstDot == -1 {
				sourceRoot = sourceName
			} else {
				sourceRoot = sourceName[:firstDot]
			}

			// Add to usage
			consumers := dsUsage[targetName]
			found := false
			for _, c := range consumers {
				if c == sourceRoot {
					found = true
					break
				}
			}
			if !found {
				dsUsage[targetName] = append(consumers, sourceRoot)
			}
		}
	}

	// 2. Check for violations
	for dsID, consumers := range dsUsage {
		if len(consumers) > 1 {
			// Check if allowed (e.g. metadata "shared" = "true")
			// We need to find the datastore object again to check metadata
			allowed := isSharedDatastore(program, dsID)

			if !allowed {
				diags = append(diags, diagnostics.Diagnostic{
					Code:     diagnostics.CodeBestPractice, // We might need to define this new code, or use a string
					Severity: diagnostics.SeverityWarning,  // Best practice is usually a warning
					Message:  fmt.Sprintf("Best Practice Violation: DataStore '%s' is accessed by multiple services (%s). Prefer Database-Per-Service pattern.", dsID, strings.Join(consumers, ", ")),
					Location: diagnostics.SourceLocation{File: "architecture"}, // General location if we don't track the definition easily
					Context: []string{
						fmt.Sprintf("Accessed by: %s", strings.Join(consumers, ", ")),
					},
					Suggestions: []string{
						"Consider splitting the DataStore into service-specific databases",
						"Use the Database-Per-Service pattern for better service isolation",
						"If shared data is required, mark with `metadata { shared \"true\" }` and document the rationale",
					},
				})
			}
		}
	}

	return diags
}

func isTargetDatastore(program *language.Program, name string) bool {
	if program == nil || program.Model == nil {
		return false
	}

	// Search for database elements in Model
	var findDatabase func(elem *language.ElementDef, currentFQN string) bool
	findDatabase = func(elem *language.ElementDef, currentFQN string) bool {
		if elem == nil {
			return false
		}

		id := elem.GetID()
		if id == "" {
			return false
		}

		fqn := id
		if currentFQN != "" {
			fqn = buildQualifiedID(currentFQN, id)
		}

		// Check if this is a database and matches the name
		if elem.GetKind() == "database" && (fqn == name || id == name) {
			return true
		}

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					if findDatabase(bodyItem.Element, fqn) {
						return true
					}
				}
			}
		}

		return false
	}

	// Search all top-level elements
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			if findDatabase(item.ElementDef, "") {
				return true
			}
		}
	}

	return false
}

func isSharedDatastore(program *language.Program, name string) bool {
	if program == nil || program.Model == nil {
		return false
	}

	checkMeta := func(metadata []*language.MetaEntry) bool {
		for _, m := range metadata {
			if m.Key == "shared" && m.Value != nil {
				val := *m.Value
				return val == "true" || val == "\"true\""
			}
		}
		return false
	}

	// Search for database element and check its metadata
	var findAndCheckDatabase func(elem *language.ElementDef, currentFQN string) bool
	findAndCheckDatabase = func(elem *language.ElementDef, currentFQN string) bool {
		if elem == nil {
			return false
		}

		id := elem.GetID()
		if id == "" {
			return false
		}

		fqn := id
		if currentFQN != "" {
			fqn = buildQualifiedID(currentFQN, id)
		}

		// Check if this is the database we're looking for
		if elem.GetKind() == "database" && (fqn == name || id == name) {
			// Extract metadata from body
			body := elem.GetBody()
			if body != nil {
				for _, item := range body.Items {
					if item.Metadata != nil {
						return checkMeta(item.Metadata.Entries)
					}
				}
			}
			return false
		}

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					if findAndCheckDatabase(bodyItem.Element, fqn) {
						return true
					}
				}
			}
		}

		return false
	}

	// Search all top-level elements
	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			if findAndCheckDatabase(item.ElementDef, "") {
				return true
			}
		}
	}

	return false
}

// PublicInterfaceDocumentationRule ensures that any System/Container exposed to Persons (users)
// has proper documentation (Description and Technology).
type PublicInterfaceDocumentationRule struct{}

func (r *PublicInterfaceDocumentationRule) Name() string {
	return "Public Interface Documentation"
}

func (r *PublicInterfaceDocumentationRule) Validate(program *language.Program) []diagnostics.Diagnostic {
	var diags []diagnostics.Diagnostic
	if program == nil || program.Model == nil {
		return diags
	}

	// 1. Find all person IDs
	personIDs := make(map[string]bool)
	var collectPersons func(elem *language.ElementDef)
	collectPersons = func(elem *language.ElementDef) {
		if elem == nil {
			return
		}
		if elem.GetKind() == "person" {
			id := elem.GetID()
			if id != "" {
				personIDs[id] = true
			}
		}
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					collectPersons(bodyItem.Element)
				}
			}
		}
	}

	for _, item := range program.Model.Items {
		if item.ElementDef != nil {
			collectPersons(item.ElementDef)
		}
	}

	// 2. Find all items accessed by a Person
	accessedItems := make(map[string]bool)

	// Collect all relations with their scopes
	relationsWithScope := collectAllRelations(program.Model)

	// Check relations for person -> element
	for _, relScope := range relationsWithScope {
		rel := relScope.Relation
		fromName := rel.From.String()
		// Check if FROM matches a Person ID
		if personIDs[fromName] {
			toName := rel.To.String()
			// Resolve to Full Name
			fullToName := toName
			scope := relScope.Scope
			if scope != "" && !strings.Contains(toName, ".") {
				fullToName = buildQualifiedID(scope, toName)
			}
			accessedItems[fullToName] = true
			accessedItems[toName] = true
		}
	}

	// 3. Validate those items have description/technology
	// Helper to find element and check documentation
	var checkElementDoc func(elem *language.ElementDef, currentFQN string, targetName string) bool
	checkElementDoc = func(elem *language.ElementDef, currentFQN string, targetName string) bool {
		if elem == nil {
			return false
		}

		id := elem.GetID()
		if id == "" {
			return false
		}

		fqn := id
		if currentFQN != "" {
			fqn = buildQualifiedID(currentFQN, id)
		}

		// Check if this is the element we're looking for
		if fqn == targetName || id == targetName {
			description := ""
			technology := ""

			body := elem.GetBody()
			if body != nil {
				for _, item := range body.Items {
					if item.Description != nil {
						description = *item.Description
					}
					if item.Technology != nil {
						technology = *item.Technology
					}
				}
			}

			kind := elem.GetKind()
			loc := elem.Location()

			// Check for missing description
			if description == "" {
				var msg string
				switch kind {
				case "system":
					msg = fmt.Sprintf("Public API Documentation: System '%s' is used by humans but lacks a description.", id)
				case "container":
					msg = fmt.Sprintf("Public Interface: Container '%s' is used by humans but lacks a description.", id)
				default:
					return true // Only check systems and containers
				}

				diags = append(diags, diagnostics.Diagnostic{
					Code:     diagnostics.CodeBestPractice,
					Severity: diagnostics.SeverityWarning,
					Message:  msg,
					Location: diagnostics.SourceLocation{
						File:   loc.File,
						Line:   loc.Line,
						Column: loc.Column,
					},
					Suggestions: []string{
						fmt.Sprintf("Add a description to %s '%s'", kind, id),
						"Public interfaces should be well-documented for API consumers",
					},
				})
			}

			// Check for missing technology (containers only)
			if kind == "container" && technology == "" {
				loc := elem.Location()
				diags = append(diags, diagnostics.Diagnostic{
					Code:     diagnostics.CodeBestPractice,
					Severity: diagnostics.SeverityWarning,
					Message:  fmt.Sprintf("Public Interface: Container '%s' is used by humans but lacks technology specification.", id),
					Location: diagnostics.SourceLocation{
						File:   loc.File,
						Line:   loc.Line,
						Column: loc.Column,
					},
					Suggestions: []string{
						fmt.Sprintf("Add technology to Container '%s' (e.g., 'Go', 'React')", id),
						"Technology helps API consumers understand implementation details",
					},
				})
			}

			return true
		}

		// Recurse into nested elements
		body := elem.GetBody()
		if body != nil {
			for _, bodyItem := range body.Items {
				if bodyItem.Element != nil {
					if checkElementDoc(bodyItem.Element, fqn, targetName) {
						return true
					}
				}
			}
		}

		return false
	}

	// Check all accessed items
	for itemName := range accessedItems {
		// Search for the element in the model
		for _, item := range program.Model.Items {
			if item.ElementDef != nil {
				checkElementDoc(item.ElementDef, "", itemName)
			}
		}
	}

	return diags
}

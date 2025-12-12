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
	if program.Architecture == nil {
		return diags
	}

	// 1. Identify all DataStores
	// Map: DataStore ID -> List of Consumers (System/Container/Component IDs)
	// Estimate capacity: typically few datastores per architecture
	estimatedDataStores := 8
	if program.Architecture != nil {
		for _, sys := range program.Architecture.Systems {
			estimatedDataStores += len(sys.DataStores)
		}
		estimatedDataStores += len(program.Architecture.DataStores)
	}
	dsUsage := make(map[string][]string, estimatedDataStores)

	collectRelations := func(program *language.Program) []*language.Relation {
		if program.Architecture == nil {
			return nil
		}

		// Estimate capacity for relations
		estimatedRels := len(program.Architecture.Relations)
		for _, sys := range program.Architecture.Systems {
			estimatedRels += len(sys.Relations)
			for _, cont := range sys.Containers {
				estimatedRels += len(cont.Relations)
				for _, comp := range cont.Components {
					estimatedRels += len(comp.Relations)
				}
			}
		}
		allRels := make([]*language.Relation, 0, estimatedRels)
		allRels = append(allRels, program.Architecture.Relations...)

		for _, sys := range program.Architecture.Systems {
			allRels = append(allRels, sys.Relations...)
			for _, cont := range sys.Containers {
				allRels = append(allRels, cont.Relations...)
				for _, comp := range cont.Components {
					allRels = append(allRels, comp.Relations...)
				}
			}
		}
		return allRels
	}

	rels := collectRelations(program)

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
	// Root level
	for _, ds := range program.Architecture.DataStores {
		if ds.ID == name {
			return true
		}
	}
	// System level
	for _, sys := range program.Architecture.Systems {
		for _, ds := range sys.DataStores {
			// ID inside system is usually just the name, but reference is System.Name
			if sys.ID+"."+ds.ID == name {
				return true
			}
		}
	}
	return false
}

func isSharedDatastore(program *language.Program, name string) bool {
	checkMeta := func(metadata []*language.MetaEntry) bool {
		for _, m := range metadata {
			if m.Key == "shared" && m.Value != nil {
				val := *m.Value
				return val == "true" || val == "\"true\""
			}
		}
		return false
	}

	for _, ds := range program.Architecture.DataStores {
		if ds.ID == name {
			return checkMeta(ds.Metadata)
		}
	}
	for _, sys := range program.Architecture.Systems {
		for _, ds := range sys.DataStores {
			if sys.ID+"."+ds.ID == name {
				return checkMeta(ds.Metadata)
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
	if program.Architecture == nil {
		return diags
	}

	// 1. Find all items accessed by a Person
	accessedItems := make(map[string]bool)

	// Re-implement iteration with context
	checkRel := func(rel *language.Relation, contextPrefix string) {
		fromName := rel.From.String()
		// Check if FROM matches a Person ID (Global)
		isPerson := false
		for _, p := range program.Architecture.Persons {
			if p.ID == fromName {
				isPerson = true
				break
			}
		}
		if isPerson {
			toName := rel.To.String()
			// Resolve to Full Name
			fullToName := toName
			if contextPrefix != "" && !strings.Contains(toName, ".") {
				fullToName = contextPrefix + "." + toName
			}
			accessedItems[fullToName] = true
			// Also track the short name if it's unique? No, explicit full name is safer.
			// But for "System" access, user usually writes `User -> System`.
			// `contextPrefix` is empty.
			accessedItems[toName] = true
		}
	}

	// Iterate with context
	for _, rel := range program.Architecture.Relations {
		checkRel(rel, "")
	}
	for _, sys := range program.Architecture.Systems {
		for _, rel := range sys.Relations {
			// Inside system, resolving depends on what it is.
			// If it points to a container in THIS system, prefix it.
			checkRel(rel, sys.ID)
		}
		for _, cont := range sys.Containers {
			for _, rel := range cont.Relations {
				checkRel(rel, sys.ID+"."+cont.ID)
			}
		}
	}

	// 2. Validate those items have description/technology
	for itemName := range accessedItems {
		// Check Systems
		for _, sys := range program.Architecture.Systems {
			if sys.ID == itemName {
				if sys.Description == nil || *sys.Description == "" {
					diags = append(diags, diagnostics.Diagnostic{
						Code:     diagnostics.CodeBestPractice,
						Severity: diagnostics.SeverityWarning,
						Message:  fmt.Sprintf("Public API Documentation: System '%s' is used by humans but lacks a description.", sys.ID),
						Suggestions: []string{
							fmt.Sprintf("Add a description to System '%s' to document its purpose", sys.ID),
							"Descriptions help users understand the system's role and responsibilities",
						},
					})
				}
			}
			// Check Containers
			for _, cont := range sys.Containers {
				fullName := sys.ID + "." + cont.ID
				if fullName == itemName {
					if cont.Description == nil || *cont.Description == "" {
						diags = append(diags, diagnostics.Diagnostic{
							Code:     diagnostics.CodeBestPractice,
							Severity: diagnostics.SeverityWarning,
							Message:  fmt.Sprintf("Public Interface: Container '%s' is used by humans but lacks a description.", fullName),
							Suggestions: []string{
								fmt.Sprintf("Add a description to Container '%s'", fullName),
								"Public interfaces should be well-documented for API consumers",
							},
						})
					}
					// Check technology for containers (Systems don't have technology)
					hasTech := false
					for i := range cont.Items {
						item := cont.Items[i]
						if item.Technology != nil {
							hasTech = true
						}
					}
					// OR check if we populated it in post-process (which we did in ast_elements.go: Technology *string)
					// But we have to check the Container struct directly.
					// The Container struct in ast_elements.go has Technology *string, but let's check validation logic.
					// Actually, Container struct has Technology *string? No, it has Items.
					// Wait, looking at ast_elements.go again...
					// Container doesn't have Technology field directly on struct in the snippet I saw?
					// Let's re-verify AST.
					// Ah, ast_elements.go:96: "Post-processed fields" doesn't list Technology.
					// But ast_elements.go:118: ContainerItem has Technology.
					// We need to iterate items to be sure or check if we added a convenience field.
					// The AST snippet for Container (lines 88-110) does NOT show a Technology *string convenience field.
					// So we must check Items.

					if !hasTech {
						for i := range cont.Items {
							it := cont.Items[i]
							if it.Technology != nil {
								hasTech = true
								break
							}
						}
					}

					if !hasTech {
						diags = append(diags, diagnostics.Diagnostic{
							Code:     diagnostics.CodeBestPractice,
							Severity: diagnostics.SeverityInfo,
							Message:  fmt.Sprintf("Public Interface: Container '%s' should specify its Technology (e.g., 'React', 'iOS').", fullName),
							Suggestions: []string{
								fmt.Sprintf("Add technology field to Container '%s'", fullName),
								"Example: technology \"React\" or technology \"iOS\"",
								"Technology helps consumers understand implementation details",
							},
						})
					}
				}
			}
		}
	}

	return diags
}

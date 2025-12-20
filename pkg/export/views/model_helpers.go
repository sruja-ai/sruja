// pkg/export/views/model_helpers.go
// Helper functions to extract architecture elements from LikeC4 Model
package views

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// extractSystems extracts all systems from a Program's Model
func extractSystems(prog *language.Program) []*language.System {
	if prog == nil || prog.Model == nil {
		return nil
	}

	var systems []*language.System
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil {
			if sys := extractSystemFromElement(item.ElementDef); sys != nil {
				systems = append(systems, sys)
			}
		}
	}
	return systems
}

// extractSystemFromElement extracts a System from a LikeC4ElementDef if it's a system
func extractSystemFromElement(elem *language.LikeC4ElementDef) *language.System {
	if elem == nil {
		return nil
	}

	// Check if this is a system
	if elem.GetKind() != "system" {
		return nil
	}

	id := elem.GetID()
	if id == "" {
		return nil
	}

	sys := &language.System{
		ID:       id,
		Label:    ptrToString(elem.GetTitle()),
		Metadata: extractMetadata(elem),
	}

	// Extract nested elements
	body := elem.GetBody()
	if body != nil {
		for _, bodyItem := range body.Items {
			if bodyItem.Element != nil {
				// Recursively extract and apply tags from the definition line
				lineTags := extractTagsFromItem(bodyItem)

				if cont := extractContainerFromElement(bodyItem.Element); cont != nil {
					cont.Metadata = appendTagsToMetadata(cont.Metadata, lineTags)
					sys.Containers = append(sys.Containers, cont)
				}
				if ds := extractDataStoreFromElement(bodyItem.Element); ds != nil {
					ds.Metadata = appendTagsToMetadata(ds.Metadata, lineTags)
					sys.DataStores = append(sys.DataStores, ds)
				}
				if q := extractQueueFromElement(bodyItem.Element); q != nil {
					q.Metadata = appendTagsToMetadata(q.Metadata, lineTags)
					sys.Queues = append(sys.Queues, q)
				}
			}
		}
	}

	return sys
}

// extractContainerFromElement extracts a Container from a LikeC4ElementDef
func extractContainerFromElement(elem *language.LikeC4ElementDef) *language.Container {
	if elem == nil || elem.GetKind() != "container" {
		return nil
	}

	id := elem.GetID()
	if id == "" {
		return nil
	}

	cont := &language.Container{
		ID:       id,
		Label:    ptrToString(elem.GetTitle()),
		Metadata: extractMetadata(elem),
	}

	body := elem.GetBody()
	if body != nil {
		for _, bodyItem := range body.Items {
			if bodyItem.Element != nil {
				lineTags := extractTagsFromItem(bodyItem)

				if comp := extractComponentFromElement(bodyItem.Element); comp != nil {
					comp.Metadata = appendTagsToMetadata(comp.Metadata, lineTags)
					cont.Components = append(cont.Components, comp)
				}
				if ds := extractDataStoreFromElement(bodyItem.Element); ds != nil {
					ds.Metadata = appendTagsToMetadata(ds.Metadata, lineTags)
					cont.DataStores = append(cont.DataStores, ds)
				}
				if q := extractQueueFromElement(bodyItem.Element); q != nil {
					q.Metadata = appendTagsToMetadata(q.Metadata, lineTags)
					cont.Queues = append(cont.Queues, q)
				}
			}
		}
	}

	return cont
}

// extractComponentFromElement extracts a Component from a LikeC4ElementDef
func extractComponentFromElement(elem *language.LikeC4ElementDef) *language.Component {
	if elem == nil || elem.GetKind() != "component" {
		return nil
	}

	id := elem.GetID()
	if id == "" {
		return nil
	}

	return &language.Component{
		ID:       id,
		Label:    ptrToString(elem.GetTitle()),
		Metadata: extractMetadata(elem),
	}
}

// extractDataStoreFromElement extracts a DataStore from a LikeC4ElementDef
func extractDataStoreFromElement(elem *language.LikeC4ElementDef) *language.DataStore {
	if elem == nil {
		return nil
	}
	kind := elem.GetKind()
	if kind != "database" && kind != "datastore" {
		return nil
	}

	id := elem.GetID()
	if id == "" {
		return nil
	}

	return &language.DataStore{
		ID:       id,
		Label:    ptrToString(elem.GetTitle()),
		Metadata: extractMetadata(elem),
	}
}

// extractQueueFromElement extracts a Queue from a LikeC4ElementDef
func extractQueueFromElement(elem *language.LikeC4ElementDef) *language.Queue {
	if elem == nil || elem.GetKind() != "queue" {
		return nil
	}

	id := elem.GetID()
	if id == "" {
		return nil
	}

	return &language.Queue{
		ID:       id,
		Label:    ptrToString(elem.GetTitle()),
		Metadata: extractMetadata(elem),
	}
}

// extractPersons extracts all persons from a Program's Model
func extractPersons(prog *language.Program) []*language.Person {
	if prog == nil || prog.Model == nil {
		return nil
	}

	var persons []*language.Person
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "person" {
			id := item.ElementDef.GetID()
			if id != "" {
				// Top level items handled here, tags on item itself handled by extractMetadata(elementDef)??
				// Wait, top level item is ModelItem.
				// ModelItem TagRefs? Yes.
				// But extractSystemFromElement logic is for nested.
				// Here we just use element def.
				persons = append(persons, &language.Person{
					ID:       id,
					Label:    ptrToString(item.ElementDef.GetTitle()),
					Metadata: extractMetadata(item.ElementDef),
				})
			}
		}
	}
	return persons
}

// extractMetadata extracts metadata and tags from element
func extractMetadata(elem *language.LikeC4ElementDef) []*language.MetaEntry {
	if elem == nil {
		return nil
	}

	var metadata []*language.MetaEntry
	var tags []string

	// Tags defined on the element line (e.g. #tag)
	tagRefs := elem.GetTagRefs()
	for _, t := range tagRefs {
		if len(t) > 1 && t[0] == '#' {
			tags = append(tags, t[1:])
		} else {
			tags = append(tags, t)
		}
	}

	body := elem.GetBody()
	if body != nil {
		for _, item := range body.Items {
			// Skip children or relations
			if item.Element != nil || item.Relation != nil {
				continue
			}

			if item.Metadata != nil {
				metadata = append(metadata, item.Metadata.Entries...)
			}
			tags = append(tags, extractTagsFromItem(item)...)
		}
	}

	return appendTagsToMetadata(metadata, tags)
}

func extractTagsFromItem(item *language.LikeC4BodyItem) []string {
	var tags []string
	if len(item.Tags) > 0 {
		tags = append(tags, item.Tags...)
	}
	if len(item.TagRefs) > 0 {
		for _, t := range item.TagRefs {
			if len(t) > 1 && t[0] == '#' {
				tags = append(tags, t[1:])
			} else {
				tags = append(tags, t)
			}
		}
	}
	return tags
}

func appendTagsToMetadata(metadata []*language.MetaEntry, tags []string) []*language.MetaEntry {
	if len(tags) > 0 {
		metadata = append(metadata, &language.MetaEntry{
			Key:   "tags",
			Array: tags,
		})
	}
	return metadata
}

// ptrToString safely converts a string pointer to string
func ptrToString(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

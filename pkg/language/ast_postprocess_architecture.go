// pkg/language/ast_postprocess_architecture.go
// Architecture-level post-processing and relationship inference
package language

// inferImpliedRelationships automatically infers parent relationships when child relationships exist.
// This follows the DRY principle: if A -> B.C exists, then A -> B is implied.
//
// Example: If "User -> System.Container" exists, then "User -> System" is automatically inferred.
// This reduces boilerplate while maintaining clarity.
func (a *Architecture) inferImpliedRelationships() {
	existing := make(map[string]bool)
	for _, rel := range a.Relations {
		key := rel.From.String() + "->" + rel.To.String()
		existing[key] = true
	}

	allRelations := []*Relation{}
	allRelations = append(allRelations, a.Relations...)
	for _, sys := range a.Systems {
		allRelations = append(allRelations, sys.Relations...)
		for _, cont := range sys.Containers {
			allRelations = append(allRelations, cont.Relations...)
			for _, comp := range cont.Components {
				allRelations = append(allRelations, comp.Relations...)
			}
		}
	}

	for _, rel := range allRelations {
		fromParts := rel.From.Parts
		toParts := rel.To.Parts

		if len(toParts) > 1 {
			parentTo := QualifiedIdent{Parts: toParts[:len(toParts)-1]}

			shouldInfer := true
			if len(fromParts) > 0 {
				if len(fromParts) >= len(parentTo.Parts) {
					matches := true
					for i := 0; i < len(parentTo.Parts); i++ {
						if fromParts[i] != parentTo.Parts[i] {
							matches = false
							break
						}
					}
					if matches {
						shouldInfer = false
					}
				}
			}

			if shouldInfer {
				key := rel.From.String() + "->" + parentTo.String()
				if !existing[key] {
					implied := &Relation{
						From:  rel.From,
						To:    parentTo,
						Label: rel.Label,
						Tags:  rel.Tags,
						Pos:   rel.Pos,
						Verb:  rel.Verb,
					}
					a.Relations = append(a.Relations, implied)
					existing[key] = true
				}
			}
		}
	}
}

func normalizeRelation(r *Relation) {
	if r == nil {
		return
	}
	if r.Verb == nil && r.VerbRaw != nil {
		v := r.VerbRaw.Value
		r.Verb = &v
	}
}

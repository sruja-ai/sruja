// pkg/language/ast_postprocess.go
// Package language provides DSL parsing and AST structures.
package language

// normalizeRelation normalizes a relation by converting VerbRaw to Verb if needed.
func normalizeRelation(r *Relation) {
	if r == nil {
		return
	}
	if r.Verb == nil && r.VerbRaw != nil {
		v := r.VerbRaw.Value
		r.Verb = &v
	}
}

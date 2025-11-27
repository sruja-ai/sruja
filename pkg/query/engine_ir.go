// pkg/query/engine_ir.go
// IR-based query methods for querying from model.Model

package query

import (
	"github.com/sruja-ai/sruja/pkg/model"
)

// filterSystemsFromModel filters systems from an IR model.
func (e *Engine) filterSystemsFromModel(expr *Expr, m *model.Model) []ResultElement {
	var out []ResultElement
	if m == nil || m.Architecture == nil {
		return out
	}
	for _, elem := range m.Architecture.Elements {
		if elem.Type != model.ElementTypeSystem {
			continue
		}
		meta := extractMetadataFromElement(elem)
		if evalExpr(elem.ID, elem.Name, expr, meta) {
			out = append(out, ResultElement{
				ID:    elem.ID,
				Label: elem.Name,
				Type:  TypeSystem,
			})
		}
	}
	return out
}

// filterContainersFromModel filters containers from an IR model.
func (e *Engine) filterContainersFromModel(expr *Expr, m *model.Model) []ResultElement {
	var out []ResultElement
	if m == nil || m.Architecture == nil {
		return out
	}
	for _, elem := range m.Architecture.Elements {
		if elem.Type != model.ElementTypeContainer {
			continue
		}
		meta := extractMetadataFromElement(elem)
		if evalExpr(elem.ID, elem.Name, expr, meta) {
			out = append(out, ResultElement{
				ID:    elem.ID,
				Label: elem.Name,
				Type:  TypeContainer,
			})
		}
	}
	return out
}

// filterComponentsFromModel filters components from an IR model.
func (e *Engine) filterComponentsFromModel(expr *Expr, m *model.Model) []ResultElement {
	var out []ResultElement
	if m == nil || m.Architecture == nil {
		return out
	}
	for _, elem := range m.Architecture.Elements {
		if elem.Type != model.ElementTypeComponent {
			continue
		}
		meta := extractMetadataFromElement(elem)
		if evalExpr(elem.ID, elem.Name, expr, meta) {
			out = append(out, ResultElement{
				ID:    elem.ID,
				Label: elem.Name,
				Type:  TypeComponent,
			})
		}
	}
	return out
}

// filterRelationsFromModel filters relations from an IR model.
func (e *Engine) filterRelationsFromModel(expr *Expr, m *model.Model) []ResultRelation {
	var out []ResultRelation
	if m == nil || m.Architecture == nil {
		return out
	}
	for _, rel := range m.Architecture.Relations {
		meta := map[string]string{}
		if evalExpr(rel.From, rel.Description, expr, meta) || evalExpr(rel.To, rel.Description, expr, meta) {
			out = append(out, ResultRelation{
				From:  rel.From,
				To:    rel.To,
				Verb:  string(rel.Type),
				Label: rel.Description,
			})
		}
	}
	return out
}

// extractMetadataFromElement extracts metadata map from a model.Element
func extractMetadataFromElement(elem model.Element) map[string]string {
	meta := make(map[string]string)
	if elem.Metadata != nil {
		for k, v := range elem.Metadata {
			if str, ok := v.(string); ok {
				meta[k] = str
			}
		}
	}
	// Also include tags as metadata
	for _, tag := range elem.Tags {
		meta["tag:"+tag] = "true"
	}
	return meta
}

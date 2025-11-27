package query

import (
	"regexp"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/model"
)

type Engine struct {
	Program *language.Program
	Model   *model.Model // IR-based query support
}

func NewEngine(p *language.Program) *Engine {
	return &Engine{Program: p}
}

// NewEngineFromModel creates a query engine from an IR model.
func NewEngineFromModel(m *model.Model) *Engine {
	return &Engine{Model: m}
}

// SetModel updates the engine to use an IR model for queries.
func (e *Engine) SetModel(m *model.Model) {
	e.Model = m
	e.Program = nil // Clear AST reference when using IR
}

func (e *Engine) Execute(q string) (QueryResult, error) {
	parser, err := BuildParser()
	if err != nil {
		return QueryResult{}, err
	}
	ast, err := parser.ParseString("", q)
	if err != nil {
		return QueryResult{}, err
	}
	if ast.Where == nil {
		return e.selectType(ast.Type, nil), nil
	}
	return e.selectType(ast.Type, ast.Where.Expr), nil
}

func (e *Engine) selectType(t string, expr *Expr) QueryResult {
	t = strings.ToLower(t)
	switch t {
	case "systems", "system":
		return QueryResult{Elements: e.filterSystems(expr)}
	case "containers", "container":
		return QueryResult{Elements: e.filterContainers(expr)}
	case "components", "component":
		return QueryResult{Elements: e.filterComponents(expr)}
	case "relations", "relation":
		return QueryResult{Relations: e.filterRelations(expr)}
	case "anything":
		res := QueryResult{}
		res.Elements = append(res.Elements, e.filterSystems(expr)...)
		res.Elements = append(res.Elements, e.filterContainers(expr)...)
		res.Elements = append(res.Elements, e.filterComponents(expr)...)
		res.Relations = append(res.Relations, e.filterRelations(expr)...)
		return res
	default:
		return QueryResult{}
	}
}

// ExecuteFromModel executes a query against an IR model.
func (e *Engine) ExecuteFromModel(q string, m *model.Model) (QueryResult, error) {
	parser, err := BuildParser()
	if err != nil {
		return QueryResult{}, err
	}
	ast, err := parser.ParseString("", q)
	if err != nil {
		return QueryResult{}, err
	}
	if ast.Where == nil {
		return e.selectTypeFromModel(ast.Type, nil, m), nil
	}
	return e.selectTypeFromModel(ast.Type, ast.Where.Expr, m), nil
}

func (e *Engine) selectTypeFromModel(t string, expr *Expr, m *model.Model) QueryResult {
	t = strings.ToLower(t)
	switch t {
	case "systems", "system":
		return QueryResult{Elements: e.filterSystemsFromModel(expr, m)}
	case "containers", "container":
		return QueryResult{Elements: e.filterContainersFromModel(expr, m)}
	case "components", "component":
		return QueryResult{Elements: e.filterComponentsFromModel(expr, m)}
	case "relations", "relation":
		return QueryResult{Relations: e.filterRelationsFromModel(expr, m)}
	case "anything":
		res := QueryResult{}
		res.Elements = append(res.Elements, e.filterSystemsFromModel(expr, m)...)
		res.Elements = append(res.Elements, e.filterContainersFromModel(expr, m)...)
		res.Elements = append(res.Elements, e.filterComponentsFromModel(expr, m)...)
		res.Relations = append(res.Relations, e.filterRelationsFromModel(expr, m)...)
		return res
	default:
		return QueryResult{}
	}
}

func matchesField(field string, value string, term *Term, meta map[string]string) bool {
	if term == nil || term.Field == nil || term.CmpOp == nil || term.Value == nil {
		return true
	}
	fp := strings.Join(term.Field.Path, ".")
	cmp := *term.CmpOp
	val := ""
	if term.Value.String != nil {
		val = *term.Value.String
	}
	var target string
	switch fp {
	case "id":
		target = field
	case "label":
		target = value
	default:
		if strings.HasPrefix(fp, "metadata.") {
			k := strings.TrimPrefix(fp, "metadata.")
			target = meta[k]
		} else {
			target = ""
		}
	}
	ok := false
	switch cmp {
	case "==":
		ok = target == val
	case "!=":
		ok = target != val
	case "contains":
		ok = strings.Contains(strings.ToLower(target), strings.ToLower(val))
	case "starts_with":
		ok = strings.HasPrefix(strings.ToLower(target), strings.ToLower(val))
	case "ends_with":
		ok = strings.HasSuffix(strings.ToLower(target), strings.ToLower(val))
	case "matches":
		if re, err := regexp.Compile(val); err == nil {
			ok = re.MatchString(target)
		}
	case "in":
		if term.Value != nil && term.Value.List != nil {
			for _, v := range *term.Value.List {
				if v.String != nil && target == *v.String {
					ok = true
					break
				}
			}
		}
	case "exists":
		ok = target != ""
	}
	if term.Not {
		ok = !ok
	}
	return ok
}

func evalExpr(field string, label string, expr *Expr, meta map[string]string) bool {
	if expr == nil || expr.Left == nil {
		return true
	}
	left := matchesField(field, label, expr.Left, meta)
	if expr.Op == nil || expr.Right == nil {
		return left
	}
	switch *expr.Op {
	case "and":
		return left && matchesField(field, label, expr.Right, meta)
	case "or":
		return left || matchesField(field, label, expr.Right, meta)
	default:
		return left
	}
}

func (e *Engine) filterSystems(expr *Expr) []ResultElement {
	var out []ResultElement
	if e.Program == nil || e.Program.Architecture == nil {
		return out
	}
	for _, s := range e.Program.Architecture.Systems {
		meta := map[string]string{}
		for _, m := range s.Metadata {
			meta[m.Key] = m.Value
		}
		if evalExpr(s.ID, s.Label, expr, meta) {
			out = append(out, ResultElement{ID: s.ID, Label: s.Label, Type: TypeSystem})
		}
	}
	return out
}

func (e *Engine) filterContainers(expr *Expr) []ResultElement {
	var out []ResultElement
	if e.Program == nil || e.Program.Architecture == nil {
		return out
	}
	for _, s := range e.Program.Architecture.Systems {
		for _, c := range s.Containers {
			meta := map[string]string{}
			for _, m := range c.Metadata {
				meta[m.Key] = m.Value
			}
			if evalExpr(c.ID, c.Label, expr, meta) {
				out = append(out, ResultElement{ID: c.ID, Label: c.Label, Type: TypeContainer})
			}
		}
	}
	return out
}

func (e *Engine) filterComponents(expr *Expr) []ResultElement {
	var out []ResultElement
	if e.Program == nil || e.Program.Architecture == nil {
		return out
	}
	for _, s := range e.Program.Architecture.Systems {
		for _, c := range s.Containers {
			for _, comp := range c.Components {
				meta := map[string]string{}
				for _, m := range comp.Metadata {
					meta[m.Key] = m.Value
				}
				if evalExpr(comp.ID, comp.Label, expr, meta) {
					out = append(out, ResultElement{ID: comp.ID, Label: comp.Label, Type: TypeComponent})
				}
			}
		}
		for _, comp := range s.Components {
			meta := map[string]string{}
			for _, m := range comp.Metadata {
				meta[m.Key] = m.Value
			}
			if evalExpr(comp.ID, comp.Label, expr, meta) {
				out = append(out, ResultElement{ID: comp.ID, Label: comp.Label, Type: TypeComponent})
			}
		}
	}
	return out
}

func (e *Engine) filterRelations(expr *Expr) []ResultRelation {
	var out []ResultRelation
	if e.Program == nil || e.Program.Architecture == nil {
		return out
	}
	for _, r := range e.Program.Architecture.Relations {
		label := ""
		if r.Label != nil {
			label = *r.Label
		}
		meta := map[string]string{}
		if evalExpr(r.From, label, expr, meta) || evalExpr(r.To, label, expr, meta) {
			verb := ""
			if r.Verb != nil {
				verb = *r.Verb
			}
			out = append(out, ResultRelation{From: r.From, To: r.To, Verb: verb, Label: label})
		}
	}
	for _, s := range e.Program.Architecture.Systems {
		for _, r := range s.Relations {
			label := ""
			if r.Label != nil {
				label = *r.Label
			}
			meta := map[string]string{}
			if evalExpr(r.From, label, expr, meta) || evalExpr(r.To, label, expr, meta) {
				verb := ""
				if r.Verb != nil {
					verb = *r.Verb
				}
				out = append(out, ResultRelation{From: r.From, To: r.To, Verb: verb, Label: label})
			}
		}
		for _, c := range s.Containers {
			for _, r := range c.Relations {
				label := ""
				if r.Label != nil {
					label = *r.Label
				}
				meta := map[string]string{}
				if evalExpr(r.From, label, expr, meta) || evalExpr(r.To, label, expr, meta) {
					verb := ""
					if r.Verb != nil {
						verb = *r.Verb
					}
					out = append(out, ResultRelation{From: r.From, To: r.To, Verb: verb, Label: label})
				}
			}
		}
	}
	return out
}

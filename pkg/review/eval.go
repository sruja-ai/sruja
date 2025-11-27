package review

import (
	"regexp"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

func Evaluate(arch *language.Architecture, rules []Rule) []Diagnostic {
	var diags []Diagnostic
	if arch == nil {
		return diags
	}
	for _, r := range rules {
		switch r.AppliesTo {
		case AppliesSystem:
			for _, s := range arch.Systems {
				if evalExprForElement(s.ID, s.Label, s.Metadata, r.When) && !evalExprForElement(s.ID, s.Label, s.Metadata, r.Ensure) {
					diags = append(diags, diag(r, s.ID, s.Location()))
				}
			}
		case AppliesContainer:
			for _, sys := range arch.Systems {
				for _, c := range sys.Containers {
					if evalExprForElement(c.ID, c.Label, c.Metadata, r.When) && !evalExprForElement(c.ID, c.Label, c.Metadata, r.Ensure) {
						diags = append(diags, diag(r, c.ID, c.Location()))
					}
				}
			}
		case AppliesComponent:
			for _, sys := range arch.Systems {
				for _, comp := range sys.Components {
					if evalExprForElement(comp.ID, comp.Label, comp.Metadata, r.When) && !evalExprForElement(comp.ID, comp.Label, comp.Metadata, r.Ensure) {
						diags = append(diags, diag(r, comp.ID, comp.Location()))
					}
				}
				for _, c := range sys.Containers {
					for _, comp := range c.Components {
						if evalExprForElement(comp.ID, comp.Label, comp.Metadata, r.When) && !evalExprForElement(comp.ID, comp.Label, comp.Metadata, r.Ensure) {
							diags = append(diags, diag(r, comp.ID, comp.Location()))
						}
					}
				}
			}
		case AppliesRelation:
			for _, rel := range arch.Relations {
				if evalExprForRelation(rel, r.When) && !evalExprForRelation(rel, r.Ensure) {
					diags = append(diags, diagRel(r, rel))
				}
			}
			for _, sys := range arch.Systems {
				for _, rel := range sys.Relations {
					if evalExprForRelation(rel, r.When) && !evalExprForRelation(rel, r.Ensure) {
						diags = append(diags, diagRel(r, rel))
					}
				}
				for _, c := range sys.Containers {
					for _, rel := range c.Relations {
						if evalExprForRelation(rel, r.When) && !evalExprForRelation(rel, r.Ensure) {
							diags = append(diags, diagRel(r, rel))
						}
					}
				}
				for _, comp := range sys.Components {
					for _, rel := range comp.Relations {
						if evalExprForRelation(rel, r.When) && !evalExprForRelation(rel, r.Ensure) {
							diags = append(diags, diagRel(r, rel))
						}
					}
				}
			}
		}
	}
	return diags
}

func diag(r Rule, id string, loc language.SourceLocation) Diagnostic {
	return Diagnostic{RuleID: r.ID, ElementID: id, Severity: string(r.Severity), Message: r.Message, File: loc.File, Line: loc.Line, Column: loc.Column}
}

func diagRel(r Rule, rel *language.Relation) Diagnostic {
	l := rel.Location()
	id := rel.From + "->" + rel.To
	return Diagnostic{RuleID: r.ID, ElementID: id, Severity: string(r.Severity), Message: r.Message, File: l.File, Line: l.Line, Column: l.Column}
}

func evalExprForElement(id, label string, metas []*language.MetaEntry, e *Expr) bool {
	if e == nil || e.Left == nil {
		return true
	}
	left := matchTermElement(id, label, metas, e.Left)
	if e.Op == nil || e.Right == nil {
		return left
	}
	switch *e.Op {
	case "and":
		return left && matchTermElement(id, label, metas, e.Right)
	case "or":
		return left || matchTermElement(id, label, metas, e.Right)
	default:
		return left
	}
}

func matchTermElement(id, label string, metas []*language.MetaEntry, t *Term) bool {
	if t == nil || t.Field == nil || t.Op == nil {
		return true
	}
	fp := strings.Join(t.Field.Path, ".")
	op := *t.Op
	val := ""
	if t.Value != nil && t.Value.S != nil {
		val = *t.Value.S
	}
	var target string
	switch fp {
	case "id":
		target = id
	case "label":
		target = label
	default:
		if strings.HasPrefix(fp, "metadata.") {
			k := strings.TrimPrefix(fp, "metadata.")
			for _, m := range metas {
				if m.Key == k {
					target = m.Value
					break
				}
			}
		}
	}
	ok := false
	switch op {
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
		if t.Value != nil && t.Value.List != nil {
			for _, v := range *t.Value.List {
				if v.S != nil && target == *v.S {
					ok = true
					break
				}
			}
		}
	case "exists":
		ok = target != ""
	}
	if t.Not {
		ok = !ok
	}
	return ok
}

func evalExprForRelation(rel *language.Relation, e *Expr) bool {
	if e == nil || e.Left == nil {
		return true
	}
	left := matchTermRelation(rel, e.Left)
	if e.Op == nil || e.Right == nil {
		return left
	}
	switch *e.Op {
	case "and":
		return left && matchTermRelation(rel, e.Right)
	case "or":
		return left || matchTermRelation(rel, e.Right)
	default:
		return left
	}
}

func matchTermRelation(rel *language.Relation, t *Term) bool {
	if t == nil || t.Field == nil || t.Op == nil {
		return true
	}
	fp := strings.Join(t.Field.Path, ".")
	op := *t.Op
	val := ""
	if t.Value != nil && t.Value.S != nil {
		val = *t.Value.S
	}
	var target string
	switch fp {
	case "from":
		target = rel.From
	case "to":
		target = rel.To
	case "verb":
		if rel.Verb != nil {
			target = *rel.Verb
		}
	case "label":
		if rel.Label != nil {
			target = *rel.Label
		}
	}
	ok := false
	switch op {
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
		if t.Value != nil && t.Value.List != nil {
			for _, v := range *t.Value.List {
				if v.S != nil && target == *v.S {
					ok = true
					break
				}
			}
		}
	case "exists":
		ok = target != ""
	}
	if t.Not {
		ok = !ok
	}
	return ok
}

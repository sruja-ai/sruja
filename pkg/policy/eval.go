package policy

import (
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/review"
)

func EvaluatePolicy(arch *language.Architecture, rules map[string]review.Rule, p *Policy) Evaluation {
	result := Evaluation{PolicyID: p.ID}
	targets := selectTargets(arch, p)
	result.AppliesTo = targets
	totalChecks := 0
	passedChecks := 0
	controlsPassed := map[string]bool{}
	controlsFailed := map[string]bool{}

	if p.Body != nil && p.Body.Controls != nil {
		for _, ctrl := range p.Body.Controls.Controls {
			totalChecks++
			allPass := true
			for _, id := range targets {
				el := findElement(arch, id)
				if el == nil || !evalExprForElement(id, elLabel(el), elMeta(el), ctrl) {
					allPass = false
					controlsFailed[exprStr(ctrl)] = true
				}
			}
			if allPass {
				passedChecks++
				controlsPassed[exprStr(ctrl)] = true
			}
		}
	}

	if p.Body != nil && p.Body.Rules != nil {
		for _, rid := range p.Body.Rules.Items {
			totalChecks++
			r, ok := rules[rid]
			if !ok {
				continue
			}
			diags := review.Evaluate(arch, []review.Rule{r})
			viol := false
			for _, d := range diags {
				if contains(targets, d.ElementID) || relatesTarget(d.ElementID, targets) {
					result.Violations = append(result.Violations, Violation{Element: d.ElementID, Rule: rid, Severity: d.Severity, Message: d.Message})
					viol = true
				}
			}
			if !viol {
				passedChecks++
			}
		}
	}

	if totalChecks > 0 {
		result.Compliance = int((float64(passedChecks) / float64(totalChecks)) * 100)
	}
	for k := range controlsPassed {
		result.ControlsPassed = append(result.ControlsPassed, k)
	}
	for k := range controlsFailed {
		result.ControlsFailed = append(result.ControlsFailed, k)
	}
	return result
}

func selectTargets(arch *language.Architecture, p *Policy) []string {
	var ids []string
	if arch == nil || p.Body == nil || p.Body.AppliesTo == nil {
		return ids
	}
	typ := strings.ToLower(p.Body.AppliesTo.Type)
	where := p.Body.AppliesTo.Where
	switch typ {
	case "system":
		for _, s := range arch.Systems {
			if evalExprForElement(s.ID, s.Label, s.Metadata, where) {
				ids = append(ids, s.ID)
			}
		}
	case "container":
		for _, sys := range arch.Systems {
			for _, c := range sys.Containers {
				if evalExprForElement(c.ID, c.Label, c.Metadata, where) {
					ids = append(ids, c.ID)
				}
			}
		}
	case "component":
		for _, sys := range arch.Systems {
			for _, comp := range sys.Components {
				if evalExprForElement(comp.ID, comp.Label, comp.Metadata, where) {
					ids = append(ids, comp.ID)
				}
			}
			for _, c := range sys.Containers {
				for _, comp := range c.Components {
					if evalExprForElement(comp.ID, comp.Label, comp.Metadata, where) {
						ids = append(ids, comp.ID)
					}
				}
			}
		}
	default:
	}
	return ids
}

func contains(xs []string, s string) bool {
	for _, x := range xs {
		if x == s {
			return true
		}
	}
	return false
}

func relatesTarget(relID string, targets []string) bool {
	for _, t := range targets {
		if strings.Contains(relID, t) {
			return true
		}
	}
	return false
}

func findElement(arch *language.Architecture, id string) interface{} {
	for _, s := range arch.Systems {
		if s.ID == id {
			return s
		}
		for _, c := range s.Containers {
			if c.ID == id {
				return c
			}
			for _, comp := range c.Components {
				if comp.ID == id {
					return comp
				}
			}
		}
		for _, comp := range s.Components {
			if comp.ID == id {
				return comp
			}
		}
	}
	for _, p := range arch.Persons {
		if p.ID == id {
			return p
		}
	}
	return nil
}

func elLabel(el interface{}) string {
	switch v := el.(type) {
	case *language.System:
		return v.Label
	case *language.Container:
		return v.Label
	case *language.Component:
		return v.Label
	case *language.Person:
		return v.Label
	}
	return ""
}

func elMeta(el interface{}) []*language.MetaEntry {
	switch v := el.(type) {
	case *language.System:
		return v.Metadata
	case *language.Container:
		return v.Metadata
	case *language.Component:
		return v.Metadata
	}
	return nil
}

func evalExprForElement(id, label string, metas []*language.MetaEntry, e *review.Expr) bool {
	return matchElement(id, label, metas, e)
}

func matchElement(id, label string, metas []*language.MetaEntry, e *review.Expr) bool {
	if e == nil || e.Left == nil {
		return true
	}
	left := matchTerm(id, label, metas, e.Left)
	if e.Op == nil || e.Right == nil {
		return left
	}
	switch *e.Op {
	case "and":
		return left && matchTerm(id, label, metas, e.Right)
	case "or":
		return left || matchTerm(id, label, metas, e.Right)
	default:
		return left
	}
}

func matchTerm(id, label string, metas []*language.MetaEntry, t *review.Term) bool {
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
	case "contains":
		ok = strings.Contains(strings.ToLower(target), strings.ToLower(val))
	case "exists":
		ok = target != ""
	}
	if t.Not {
		ok = !ok
	}
	return ok
}

func exprStr(e *review.Expr) string {
	if e == nil || e.Left == nil || e.Left.Field == nil {
		return ""
	}
	return strings.Join(e.Left.Field.Path, ".")
}

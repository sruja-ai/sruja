package approval

import (
    "regexp"
    "strings"
)

func Evaluate(policy Policy, changes []Change) EvaluationResult {
    res := EvaluationResult{}
    for _, ch := range changes {
        if !applies(policy.AppliesTo, ch) { continue }
        // exception
        if matchCondition(policy.Except, PolicyContext{}, ch) {
            res.Skipped = append(res.Skipped, ch)
            continue
        }
        // auto-approve
        if policy.AutoApprove == "true" || matchCondition(policy.AutoApprove, PolicyContext{}, ch) {
            res.AutoApproved = append(res.AutoApproved, ch)
            continue
        }
        // when
        if matchCondition(policy.When, PolicyContext{}, ch) {
            res.RequiredApprovals = appendUniqueRules(res.RequiredApprovals, policy.Require)
            res.Reasons = append(res.Reasons, reasonFromChange(ch))
        }
    }
    return res
}

func applies(targets []Target, ch Change) bool {
    if len(targets) == 0 { return true }
    for _, t := range targets {
        if strings.EqualFold(t.Type, "architecture") { return true }
        if strings.EqualFold(t.Type, ch.Kind) {
            if t.Value == "*" || t.Value == "" { return true }
            if wildcardMatch(t.Value, ch.ID) { return true }
            for _, v := range t.Values { if wildcardMatch(v, ch.ID) { return true } }
        }
    }
    return false
}

// matchCondition evaluates a condition string against a change and optionally a diff-aware context
func matchCondition(cond string, ctx PolicyContext, ch Change) bool {
    c := strings.TrimSpace(strings.ToLower(cond))
    if c == "" { return false }
    if c == "any_change == true" || c == "any_change" { return true }
    // field
    if strings.Contains(c, "field.pii == true") {
        return ch.Kind == "field" && ch.Metadata["pii"] == "true"
    }
    if strings.Contains(c, "field.changed == true") {
        if ch.Kind == "field" && ch.Details["changed"] == true { return true }
        // diff-aware
        if ctx.Diff != nil && ctx.Diff.Fields != nil {
            if fd, ok := ctx.Diff.Fields[ch.ID]; ok { return fd.Changed }
        }
        return false
    }
    // event
    if strings.Contains(c, "event.version.bump == \"major\"") {
        if ch.Kind == "event" && ch.Details["version_bump"] == "major" { return true }
        if ctx.Diff != nil && ctx.Diff.Events != nil {
            if ed, ok := ctx.Diff.Events[ch.ID]; ok { return strings.ToLower(ed.Version.Bump) == "major" }
        }
        return false
    }
    if strings.Contains(c, "event.schema.breaking == true") {
        if ch.Kind == "event" && ch.Details["schema_breaking"] == true { return true }
        if ctx.Diff != nil && ctx.Diff.Events != nil {
            if ed, ok := ctx.Diff.Events[ch.ID]; ok { return ed.Schema.Breaking }
        }
        return false
    }
    // contract
    if strings.Contains(c, "contract.breaking_change == true") {
        if ch.Kind == "contract" && ch.Details["breaking_change"] == true { return true }
        if ctx.Diff != nil && ctx.Diff.Contracts != nil {
            if cd, ok := ctx.Diff.Contracts[ch.ID]; ok { return cd.BreakingChange }
        }
        return false
    }
    if strings.Contains(c, "contract.compatible == true") {
        if ch.Kind == "contract" && ch.Details["compatible"] == true { return true }
        if ctx.Diff != nil && ctx.Diff.Contracts != nil {
            if cd, ok := ctx.Diff.Contracts[ch.ID]; ok { return cd.Compatible }
        }
        return false
    }
    // system
    if strings.Contains(c, "system.boundaries.changed") {
        if ch.Kind == "system" && ch.Details["boundaries_changed"] == true { return true }
        if ctx.Diff != nil && ctx.Diff.Systems != nil {
            if sd, ok := ctx.Diff.Systems[ch.ID]; ok { return sd.BoundaryChanged }
        }
        return false
    }
    // metadata
    if strings.Contains(c, "metadata.business_critical == true") {
        return ch.Metadata["business_critical"] == "true"
    }
    // branch exceptions
    if strings.Contains(c, "branch.name startswith \"experiment/\"") {
        name := ch.Metadata["branch_name"]
        return strings.HasPrefix(name, "experiment/")
    }
    // entity fields lists: entity.fields.added contains "email"
    if strings.Contains(c, "entity.fields.added contains") {
        if ctx.Diff != nil && ctx.Diff.Entities != nil {
            if ed, ok := ctx.Diff.Entities[ch.ID]; ok {
                // extract quoted value
                m := regexp.MustCompile(`contains\s+\"([^\"]+)\"`).FindStringSubmatch(cond)
                if len(m) == 2 {
                    for _, f := range ed.Fields.Added { if f == m[1] { return true } }
                }
            }
        }
        return false
    }
    return false
}

func wildcardMatch(pattern, s string) bool {
    if pattern == s { return true }
    if strings.HasPrefix(pattern, "*") && strings.HasSuffix(pattern, "*") {
        return strings.Contains(s, strings.Trim(pattern, "*"))
    }
    if strings.HasPrefix(pattern, "*") {
        return strings.HasSuffix(s, strings.TrimLeft(pattern, "*"))
    }
    if strings.HasSuffix(pattern, "*") {
        return strings.HasPrefix(s, strings.TrimRight(pattern, "*"))
    }
    return false
}

func appendUniqueRules(dst []ApprovalRule, src []ApprovalRule) []ApprovalRule {
    exists := func(a ApprovalRule) bool {
        for _, d := range dst {
            if d.Actor == a.Actor && ((d.Quorum==nil && a.Quorum==nil) || (d.Quorum!=nil && a.Quorum!=nil && d.Quorum.Group==a.Quorum.Group && d.Quorum.Count==a.Quorum.Count)) {
                return true
            }
        }
        return false
    }
    for _, r := range src { if !exists(r) { dst = append(dst, r) } }
    return dst
}

func reasonFromChange(ch Change) string {
    switch ch.Kind {
    case "field":
        return "PII field change"
    case "event":
        return "Event change"
    case "contract":
        return "Contract change"
    case "system":
        return "System boundary change"
    default:
        return "Architecture change"
    }
}

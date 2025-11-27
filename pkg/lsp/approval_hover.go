package lsp

import (
    "fmt"
    "strings"
)

// policyKeywordHovers provides static hover docs for Approval Policy DSL keywords
var policyKeywordHovers = map[string]string{
    "policy":         "Defines an approval rule that triggers when architectural changes match certain conditions.\n\nFormat:\n`policy <ID> \"<Label>\" { ... }`",
    "applies_to":     "Specifies the architecture elements that this policy is applied to. Supports single or multiple targets.\n\nExamples:\n`applies_to: system Billing`\n`applies_to: [entity User, entity Payment]`",
    "when":           "Condition expression that determines when this policy is triggered. Supports boolean logic and attribute paths.\n\nExamples:\n`when: field.pii == true`\n`when: event.version.bump == \"major\"`",
    "require":        "Specifies the approval(s) required when the condition evaluates to true.\n\nExamples:\n`require: \"security_team\"`\n`require: [\"platform_architect\", \"domain_owner\"]`\n`require: { group: \"domain_architects\", count: 2 }`",
    "severity":       "Controls policy enforcement level.\n\nAllowed values:\n- info (informational, non-blocking)\n- warning (blocking unless suppressed)\n- error (always blocking)",
    "auto_approve":   "Allows certain changes to bypass approval if conditions match.\n\nExamples:\n`auto_approve: true`\n`auto_approve: when contract.compatible == true`",
    "except":         "Exception to the policy when condition is met.\n\nExamples:\n`except: when branch.name startswith \"experiment/\"`",
    "metadata":       "Arbitrary key-value pairs providing additional metadata about this policy.\n\nExample:\nmetadata {\n  owner: \"security\"\n  criticality: \"high\"\n}",
}

// policyAttributeHovers provides hover docs for common condition attributes
var policyAttributeHovers = map[string]string{
    "field.pii":                "Boolean attribute indicating whether a field contains personally identifiable information (PII). Used for security approval policies.",
    "field.changed":            "Boolean indicating the target field changed in the diff.",
    "event.version.bump":       "Semantic version bump classification. Enum: `major` | `minor` | `patch`. Major bumps typically require domain architect approval.",
    "event.schema.breaking":    "Indicates whether the event schema change is backward-incompatible. Evaluated by the schema diff engine.",
    "system.boundaries.changed": "True when a system's container/component boundaries changed. Usually indicates major architectural refactoring.",
    "contract.breaking_change": "Boolean indicating the API/contract change is breaking.",
    "contract.compatible":      "Boolean indicating changes are backward-compatible.",
}

// policyOperatorHovers provides hover docs for operators
var policyOperatorHovers = map[string]string{
    "==":       "Equality operator",
    "!=":       "Inequality operator",
    "contains": "True if left-hand operand includes the right-hand operand (string/list)",
    "matches":  "Regex operator: left string must match the provided regular expression",
    "and":      "Boolean conjunction",
    "or":       "Boolean disjunction",
    "not":      "Boolean negation",
}

// ProvideApprovalHover tries to provide policy-related hover content based on the current line context
func (hp *HoverProvider) ProvideApprovalHover(text string, line, character int) *HoverInfo {
    lines := strings.Split(text, "\n")
    if line < 0 || line >= len(lines) { return nil }
    l := lines[line]
    before := l[:character]
    ident := identifierAt(text, line, character)
    if ident == "" { return nil }

    // Keyword hovers
    key := strings.TrimSuffix(ident, ":")
    if doc, ok := policyKeywordHovers[key]; ok {
        return &HoverInfo{Contents: fmt.Sprintf("**%s**\n\n%s", key, doc)}
    }

    // Operator hovers
    if doc, ok := policyOperatorHovers[ident]; ok {
        return &HoverInfo{Contents: fmt.Sprintf("**operator** `%s`\n\n%s", ident, doc)}
    }

    // Attribute hovers (field/event/contract/system/metadata paths)
    // Detect attribute path prefix in the text before cursor
    path := detectAttributePath(before)
    if path != "" {
        if doc, ok := policyAttributeHovers[path]; ok {
            return &HoverInfo{Contents: fmt.Sprintf("**attribute** `%s`\n\n%s", path, doc)}
        }
        // metadata key
        if strings.HasPrefix(path, "metadata.") {
            key := strings.TrimPrefix(path, "metadata.")
            if h := hp.generateHoverForMetadataKey(key, "policy"); h != nil { return h }
        }
    }

    // Target type + identifier hovers (IR-driven)
    // Heuristic: look for preceding target type token in the same line
    tokens := strings.Fields(l)
    for i, t := range tokens {
        if t == ident && i > 0 {
            kind := strings.TrimSuffix(tokens[i-1], ":")
            switch kind {
            case "system", "container", "component", "entity", "event", "contract":
                // Try semantic index element
                if elem, ok := hp.semanticIndex.GetElement(ident); ok {
                    return hp.generateHoverForElement(elem, nil)
                }
            }
        }
    }

    // Approver hovers (quoted strings)
    if strings.HasPrefix(ident, "\"") && strings.HasSuffix(ident, "\"") {
        name := strings.Trim(ident, "\"")
        return &HoverInfo{Contents: fmt.Sprintf("**Approval Group:** `%s`\n\nUsed in require blocks to specify teams/users or quorum groups.", name)}
    }

    return nil
}

func detectAttributePath(before string) string {
    s := strings.TrimSpace(before)
    // find last word containing a dot
    parts := strings.Fields(s)
    if len(parts) == 0 { return "" }
    w := parts[len(parts)-1]
    if strings.Contains(w, ".") {
        // sanitize trailing characters
        w = strings.TrimRight(w, ":,)")
        return w
    }
    return ""
}

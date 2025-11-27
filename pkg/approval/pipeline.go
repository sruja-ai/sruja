package approval

import "strings"

type ArchitectureIR struct {
    Systems   []string
    Entities  []string
    Events    []string
    Contracts []string
}

type IRNode struct {
    Kind string
    ID   string
}

type ModelDiff struct {
    Added    []IRNode
    Removed  []IRNode
    Updated  []Change
    // Semantic maps for diff-aware evaluation
    Systems   map[string]*SystemDiff
    Entities  map[string]*EntityDiff
    Events    map[string]*EventDiff
    Contracts map[string]*ContractDiff
    Fields    map[string]*FieldDiff
}

type PolicyContext struct {
    OldIR       *ArchitectureIR
    NewIR       *ArchitectureIR
    Diff        *ModelDiff
    Policy      *Policy
    TargetNodes []IRNode
}

type Diagnostic struct {
    Message  string `json:"message"`
    Severity string `json:"severity"`
    PolicyID string `json:"policy"`
}

type ApprovalRequirement struct {
    PolicyID string `json:"policy_id"`
    NodeID   string `json:"node_id"`
    Team     string `json:"team"`
    Count    int    `json:"count"`
    Reason   string `json:"reason"`
}

type PolicyEvalResult struct {
    PolicyID          string               `json:"policy_id"`
    Triggered         bool                 `json:"triggered"`
    AutoApproved      bool                 `json:"auto_approved"`
    ExceptionApplied  bool                 `json:"exception_applied"`
    Severity          string               `json:"severity"`
    RequiredApprovals []ApprovalRequirement `json:"required_approvals"`
    Diagnostics       []Diagnostic          `json:"diagnostics"`
    Explanation       string               `json:"explanation"`
}

type PolicyEvalReport struct {
    Results           []PolicyEvalResult   `json:"results"`
    RequiredApprovals []ApprovalRequirement `json:"required_approvals"`
    Diagnostics       []Diagnostic          `json:"diagnostics"`
    AutoApproved      []string              `json:"auto_approved"`
}

// EvaluateAll evaluates a set of policies against provided changes and returns a report.
// If OldIR/NewIR/Diff are nil, evaluation will be based only on the changes slice.
func EvaluateAll(oldIR, newIR *ArchitectureIR, diff *ModelDiff, policies []Policy, changes []Change) PolicyEvalReport {
    var report PolicyEvalReport
    for _, pol := range policies {
        ctx := PolicyContext{OldIR: oldIR, NewIR: newIR, Diff: diff, Policy: &pol}
        res := evaluatePolicy(ctx, changes)
        report.Results = append(report.Results, res)
        if res.AutoApproved {
            report.AutoApproved = append(report.AutoApproved, pol.ID)
        }
        for _, r := range res.RequiredApprovals {
            report.RequiredApprovals = append(report.RequiredApprovals, r)
        }
        for _, d := range res.Diagnostics {
            report.Diagnostics = append(report.Diagnostics, d)
        }
    }
    return report
}

func evaluatePolicy(ctx PolicyContext, changes []Change) PolicyEvalResult {
    pol := ctx.Policy
    res := PolicyEvalResult{PolicyID: pol.ID, Severity: pol.Severity}
    // Apply targets: keep changes that match applies_to
    var applicable []Change
    for _, ch := range changes {
        if applies(pol.AppliesTo, ch) { applicable = append(applicable, ch) }
    }
    if len(applicable) == 0 { return res }

    // Exceptions
    for _, ch := range applicable {
        if matchCondition(pol.Except, ctx, ch) {
            res.ExceptionApplied = true
            res.Diagnostics = append(res.Diagnostics, Diagnostic{Message: "Policy exception applied", Severity: "info", PolicyID: pol.ID})
            return res
        }
    }

    // Auto-approve
    if pol.AutoApprove == "true" || pol.AutoApprove == "false" {
        if pol.AutoApprove == "true" {
            res.AutoApproved = true
            res.Diagnostics = append(res.Diagnostics, Diagnostic{Message: "Auto-approved by policy", Severity: "info", PolicyID: pol.ID})
            return res
        }
    } else {
        for _, ch := range applicable {
            if matchCondition(pol.AutoApprove, ctx, ch) {
                res.AutoApproved = true
                res.Diagnostics = append(res.Diagnostics, Diagnostic{Message: "Auto-approved by condition", Severity: "info", PolicyID: pol.ID})
                return res
            }
        }
    }

    // When condition
    triggered := false
    for _, ch := range applicable { if matchCondition(pol.When, ctx, ch) { triggered = true; break } }
    if !triggered { return res }

    res.Triggered = true
    res.Explanation = buildExplanation(pol, applicable)
    // Build approval requirements
    for _, ch := range applicable {
        for _, r := range pol.Require {
            if r.Actor != "" {
                res.RequiredApprovals = append(res.RequiredApprovals, ApprovalRequirement{PolicyID: pol.ID, NodeID: ch.ID, Team: r.Actor, Count: 1, Reason: res.Explanation})
            } else if r.Quorum != nil {
                res.RequiredApprovals = append(res.RequiredApprovals, ApprovalRequirement{PolicyID: pol.ID, NodeID: ch.ID, Team: r.Quorum.Group, Count: r.Quorum.Count, Reason: res.Explanation})
            }
        }
    }
    // Diagnostics
    res.Diagnostics = append(res.Diagnostics, Diagnostic{Message: res.Explanation, Severity: severityOrDefault(pol.Severity), PolicyID: pol.ID})
    return res
}

func severityOrDefault(s string) string { if s == "" { return "error" } ; return s }

func buildExplanation(pol *Policy, changes []Change) string {
    // Simple explanation: list of matched changes and policy rule
    var ids []string
    for _, ch := range changes { ids = append(ids, ch.Kind+":"+ch.ID) }
    return "Policy triggered: " + pol.ID + " due to changes: " + strings.Join(ids, ", ")
}
type FieldDiff struct {
    Added         bool
    Removed       bool
    Changed       bool
    PIIChanged    bool
}

type EntityDiff struct {
    Added    bool
    Removed  bool
    Changed  bool
    Fields struct {
        Added   []string
        Removed []string
        Changed []string
    }
    LifecycleChanged bool
    MetadataChanged  bool
}

type EventDiff struct {
    Added   bool
    Removed bool
    Changed bool
    Version struct {
        Old  string
        New  string
        Bump string
    }
    Schema struct {
        Added    []string
        Removed  []string
        Changed  []string
        Breaking bool
    }
}

type ContractDiff struct {
    Added   bool
    Removed bool
    Changed bool
    Operations struct {
        Added   []string
        Removed []string
        Changed []string
    }
    BreakingChange bool
    Compatible     bool
}

type SystemDiff struct {
    Added   bool
    Removed bool
    Changed bool
    BoundaryChanged  bool
    RelationsChanged bool
}

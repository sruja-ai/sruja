package approval

type Target struct {
    Type   string   `json:"type"`
    Value  string   `json:"value"`
    Values []string `json:"values,omitempty"`
}

type Quorum struct {
    Group string `json:"group"`
    Count int    `json:"count"`
}

type ApprovalRule struct {
    Actor   string  `json:"actor,omitempty"`
    Quorum  *Quorum `json:"quorum,omitempty"`
}

type Policy struct {
    ID        string         `json:"id"`
    Label     string         `json:"label"`
    AppliesTo []Target       `json:"applies_to"`
    When      string         `json:"when"`
    Require   []ApprovalRule `json:"require"`
    Severity  string         `json:"severity,omitempty"`
    AutoApprove string       `json:"auto_approve,omitempty"`
    Except    string         `json:"except,omitempty"`
    Metadata  map[string]string `json:"metadata,omitempty"`
}

type Change struct {
    Kind     string            `json:"kind"`  // field|entity|event|contract|system|architecture
    ID       string            `json:"id"`
    Details  map[string]any    `json:"details,omitempty"`
    Metadata map[string]string `json:"metadata,omitempty"`
}

type EvaluationResult struct {
    RequiredApprovals []ApprovalRule   `json:"required_approvals"`
    AutoApproved      []Change         `json:"auto_approved"`
    Skipped           []Change         `json:"skipped_due_to_exception"`
    Reasons           []string         `json:"reasons"`
}


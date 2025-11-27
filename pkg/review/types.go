package review

type Severity string

const (
    SeverityError   Severity = "error"
    SeverityWarning Severity = "warning"
    SeverityInfo    Severity = "info"
)

type AppliesTo string

const (
    AppliesSystem    AppliesTo = "system"
    AppliesContainer AppliesTo = "container"
    AppliesComponent AppliesTo = "component"
    AppliesRelation  AppliesTo = "relation"
)

type Rule struct {
    ID          string
    Description string
    AppliesTo   AppliesTo
    Message     string
    Severity    Severity
    When        *Expr
    Ensure      *Expr
}

type Diagnostic struct {
    RuleID     string  `json:"rule"`
    ElementID  string  `json:"element"`
    Severity   string  `json:"severity"`
    Message    string  `json:"message"`
    File       string  `json:"file,omitempty"`
    Line       int     `json:"line,omitempty"`
    Column     int     `json:"column,omitempty"`
}


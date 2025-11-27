package policy

type Evaluation struct {
    PolicyID        string            `json:"policy"`
    AppliesTo       []string          `json:"appliesTo"`
    Compliance      int               `json:"compliance"`
    Violations      []Violation       `json:"violations"`
    ControlsPassed  []string          `json:"controlsPassed"`
    ControlsFailed  []string          `json:"controlsFailed"`
}

type Violation struct {
    Element   string `json:"element"`
    Rule      string `json:"rule"`
    Severity  string `json:"severity"`
    Message   string `json:"message"`
}


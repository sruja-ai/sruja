package approval

import (
    "github.com/alecthomas/participle/v2"
    "github.com/alecthomas/participle/v2/lexer"
    "strconv"
    "strings"
)

// Root
type PolicyFile struct {
    Policies []*PolicyDecl `parser:"@@*"`
}

// Policy block
type PolicyDecl struct {
    KwPolicy string          `parser:"'policy'"`
    ID       string          `parser:"@Ident"`
    Label    string          `parser:"@String"`
    LBrace   string          `parser:"'{'"`
    AppliesTo *TargetSelector `parser:"'applies_to' ':' @@?"`
    When       *Expr          `parser:"'when' ':' @@?"`
    Require    *ApprovalSpec  `parser:"'require' ':' @@?"`
    Severity   *string        `parser:"( 'severity' ':' @('info'|'warning'|'error') )?"`
    Auto       *AutoApproval  `parser:"( 'auto_approve' ':' @@ )?"`
    Except     *ExceptionRule `parser:"( 'except' ':' @@ )?"`
    Metadata   []*MetadataEntry `parser:"( 'metadata' '{' @@* '}' )?"`
    RBrace     string          `parser:"'}'"`
}

type MetadataEntry struct {
    Key string `parser:"@Ident ':'"`
    Val string `parser:"@String"`
}

// Target selectors
type TargetSelector struct {
    Single *SingleTarget   `parser:"@@ |"`
    List   []*SingleTarget `parser:"'[' @@ ( ',' @@ )* ']'"`
}

type SingleTarget struct {
    Kind   string    `parser:"@( 'system' | 'container' | 'component' | 'entity' | 'event' | 'contract' | 'domain' | 'field' | 'diagram' | 'architecture' )"`
    Target *TargetKey `parser:"@@?"`
}

type TargetKey struct {
    Ident  *string `parser:"@Ident |"`
    String *string `parser:"@String |"`
    Regex  *string `parser:"@Regex"`
}

// Expressions
type Expr struct {
    Or []*OrExpr `parser:"@@ ( 'or' @@ )*"`
}

type OrExpr struct {
    And []*AndExpr `parser:"@@ ( 'and' @@ )*"`
}

type AndExpr struct {
    Not *NotExpr `parser:"'not' @@ | @@"`
}

type NotExpr struct {
    Primary *PrimaryExpr `parser:"@@"`
}

type PrimaryExpr struct {
    Comparison *Comparison `parser:"@@ |"`
    Group      *Expr       `parser:"'(' @@ ')'"`
}

type Comparison struct {
    Left     *Value  `parser:"@@"`
    Operator string  `parser:"@( '==' | '!=' | '<' | '<=' | '>' | '>=' | 'matches' | 'contains' )"`
    Right    *Value  `parser:"@@"`
}

type Value struct {
    Attr   *AttributePath `parser:"@@ |"`
    String *string        `parser:"@String |"`
    Number *float64       `parser:"@Float |"`
    Bool   *bool          `parser:"@('true'|'false') |"`
    List   *[]Value       `parser:"'[' @@ ( ',' @@ )* ']'"`
}

type AttributePath struct {
    Parts []string `parser:"@Ident ( '.' @Ident )*"`
}

// Approval spec
type ApprovalSpec struct {
    Single *ApprovalRuleAst   `parser:"@@ |"`
    List   []*ApprovalRuleAst `parser:"'[' @@ ( ',' @@ )* ']'"`
}

type ApprovalRuleAst struct {
    String *string         `parser:"@String |"`
    Group  *ApprovalGroupAst `parser:"'{' @@ '}'"`
}

type ApprovalGroupAst struct {
    Group string `parser:"'group' ':' @String"`
    Count int    `parser:"','? 'count' ':' @Int"`
}

// Auto-approve & Exceptions
type AutoApproval struct {
    Always *bool `parser:"@('true'|'false') |"`
    When   *Expr `parser:"'when' @@"`
}

type ExceptionRule struct {
    When *Expr `parser:"'when' @@"`
}

// Parser builder with Regex token
func BuildApprovalPolicyParser() (*participle.Parser[PolicyFile], error) {
    l := lexer.MustSimple([]lexer.SimpleRule{
        {Name: "Comment", Pattern: `//.*|/\*.*?\*/`},
        {Name: "Regex", Pattern: `/[^/]+/`},
        {Name: "String", Pattern: `"(\\"|[^"])*"`},
        {Name: "Int", Pattern: `-?\d+`},
        {Name: "Float", Pattern: `-?\d+(?:\.\d+)?`},
        {Name: "Ident", Pattern: `[a-zA-Z_][a-zA-Z0-9_]*`},
        {Name: "LBrace", Pattern: `{`},
        {Name: "RBrace", Pattern: `}`},
        {Name: "LBracket", Pattern: `\[`},
        {Name: "RBracket", Pattern: `\]`},
        {Name: "Colon", Pattern: `:`},
        {Name: "Comma", Pattern: `,`},
        {Name: "Dot", Pattern: `\.`},
        {Name: "Whitespace", Pattern: `\s+`},
    })
    return participle.Build[PolicyFile](
        participle.Lexer(l),
        participle.Unquote("String"),
        participle.Elide("Whitespace"),
        participle.Elide("Comment"),
        participle.UseLookahead(2),
    )
}

// Utilities to convert parsed AST to evaluation Policy
func ToPolicy(p *PolicyDecl) Policy {
    pol := Policy{ID: p.ID, Label: p.Label}
    if p.AppliesTo != nil {
        var targets []*SingleTarget
        if p.AppliesTo.Single != nil { targets = append(targets, p.AppliesTo.Single) }
        if len(p.AppliesTo.List) > 0 { targets = append(targets, p.AppliesTo.List...) }
        for _, st := range targets {
            t := Target{Type: st.Kind}
            if st.Target != nil {
                if st.Target.Ident != nil { t.Value = *st.Target.Ident }
                if st.Target.String != nil { t.Value = trimQuotes(*st.Target.String) }
                if st.Target.Regex != nil { t.Value = *st.Target.Regex }
            }
            pol.AppliesTo = append(pol.AppliesTo, t)
        }
    }
    if p.When != nil { pol.When = stringifyExpr(p.When) }
    if p.Require != nil {
        var rules []*ApprovalRuleAst
        if p.Require.Single != nil { rules = append(rules, p.Require.Single) }
        if len(p.Require.List) > 0 { rules = append(rules, p.Require.List...) }
        for _, r := range rules {
            if r.String != nil { pol.Require = append(pol.Require, ApprovalRule{Actor: trimQuotes(*r.String)}) }
            if r.Group != nil { pol.Require = append(pol.Require, ApprovalRule{Quorum: &Quorum{Group: trimQuotes(r.Group.Group), Count: r.Group.Count}}) }
        }
    }
    if p.Severity != nil { pol.Severity = *p.Severity }
    if p.Auto != nil {
        if p.Auto.Always != nil {
            if *p.Auto.Always { pol.AutoApprove = "true" } else { pol.AutoApprove = "false" }
        } else if p.Auto.When != nil { pol.AutoApprove = stringifyExpr(p.Auto.When) }
    }
    if p.Except != nil && p.Except.When != nil { pol.Except = stringifyExpr(p.Except.When) }
    if len(p.Metadata) > 0 {
        pol.Metadata = map[string]string{}
        for _, m := range p.Metadata { pol.Metadata[m.Key] = trimQuotes(m.Val) }
    }
    return pol
}

func trimQuotes(s string) string { return strings.Trim(s, "\"") }

func stringifyExpr(e *Expr) string {
    if e == nil { return "" }
    var parts []string
    for i, or := range e.Or {
        if i > 0 { parts = append(parts, "or") }
        parts = append(parts, stringifyOr(or))
    }
    return strings.Join(parts, " ")
}

func stringifyOr(o *OrExpr) string {
    if o == nil { return "" }
    var parts []string
    for i, and := range o.And {
        if i > 0 { parts = append(parts, "and") }
        parts = append(parts, stringifyAnd(and))
    }
    return strings.Join(parts, " ")
}

func stringifyAnd(a *AndExpr) string {
    if a == nil { return "" }
    if a.Not != nil { return "not " + stringifyPrimary(a.Not.Primary) }
    return stringifyPrimary(a.Not.Primary)
}

func stringifyPrimary(p *PrimaryExpr) string {
    if p == nil { return "" }
    if p.Comparison != nil { return stringifyComparison(p.Comparison) }
    if p.Group != nil { return "(" + stringifyExpr(p.Group) + ")" }
    return ""
}

func stringifyComparison(c *Comparison) string {
    return stringifyValue(c.Left) + " " + c.Operator + " " + stringifyValue(c.Right)
}

func stringifyValue(v *Value) string {
    if v == nil { return "" }
    if v.Attr != nil { return strings.Join(v.Attr.Parts, ".") }
    if v.String != nil { return "\"" + trimQuotes(*v.String) + "\"" }
    if v.Number != nil { return strings.TrimRight(strings.TrimRight(fmtFloat(*v.Number), "0"), ".") }
    if v.Bool != nil { if *v.Bool { return "true" } else { return "false" } }
    if v.List != nil {
        var xs []string
        for _, item := range *v.List { xs = append(xs, stringifyValue(&item)) }
        return "[" + strings.Join(xs, ", ") + "]"
    }
    return ""
}

func fmtFloat(f float64) string {
    return strconv.FormatFloat(f, 'f', -1, 64)
}

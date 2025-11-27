package policy

import (
	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
	"github.com/sruja-ai/sruja/pkg/review"
)

type PolicyFile struct {
	Policies []*Policy `parser:"@@*"`
}

type Policy struct {
	KwPolicy string      `parser:"'policy'"`
	ID       string      `parser:"@Ident"`
	LBrace   string      `parser:"'{'"`
	Body     *PolicyBody `parser:"@@"`
	RBrace   string      `parser:"'}'"`
}

type PolicyBody struct {
	Description      *string       `parser:"( 'description' @String )?"`
	AppliesTo        *AppliesTo    `parser:"( 'applies_to' @@ )?"`
	Rules            *RuleRefs     `parser:"( 'rules' '{' @@ '}' )?"`
	Controls         *ControlBlock `parser:"( 'controls' '{' @@ '}' )?"`
	RelatedStandards *StringList   `parser:"( 'related_standards' @@ )?"`
	RelatedADRs      *IdentList    `parser:"( 'related_adrs' @@ )?"`
	Severity         *Severity     `parser:"( 'severity' @@ )?"`
	Owner            *string       `parser:"( 'owner' @String )?"`
	Version          *string       `parser:"( 'version' @String )?"`
	Remediation      *string       `parser:"( 'remediation' @String )?"`
}

type AppliesTo struct {
	Type  string       `parser:"@Ident"`
	Where *review.Expr `parser:"( 'where' @@ )?"`
}

type RuleRefs struct {
	Items []string `parser:"@Ident*"`
}

type ControlBlock struct {
	Controls []*review.Expr `parser:"@@*"`
}

type StringList struct {
	Values []string `parser:"'[' @String ( ',' @String )* ']'"`
}

type IdentList struct {
	Ids []string `parser:"'[' @Ident ( ',' @Ident )* ']'"`
}

type Severity struct {
	Level string `parser:"@( 'info' | 'warning' | 'error' | 'critical' )"`
}

func BuildPolicyParser() (*participle.Parser[PolicyFile], error) {
	l := lexer.MustSimple([]lexer.SimpleRule{
		{Name: "Comment", Pattern: `//.*|/\*.*?\*/`},
		{Name: "String", Pattern: `"(\\"|[^"])*"`},
		{Name: "Float", Pattern: `-?\d+(?:\.\d+)?`},
		{Name: "Ident", Pattern: `[a-zA-Z_][a-zA-Z0-9_]*`},
		{Name: "Eq", Pattern: `==`},
		{Name: "Neq", Pattern: `!=`},
		{Name: "Dot", Pattern: `\.`},
		{Name: "Comma", Pattern: `,`},
		{Name: "LBracket", Pattern: `\[`},
		{Name: "RBracket", Pattern: `\]`},
		{Name: "LBrace", Pattern: `{`},
		{Name: "RBrace", Pattern: `}`},
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

package review

import (
	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
)

type RuleSet struct {
	Rules []*RuleDecl `parser:"@@*"`
}

type RuleDecl struct {
	Name string    `parser:"'rule' @Ident"`
	L    string    `parser:"'{'"`
	Body *RuleBody `parser:"@@"`
	R    string    `parser:"'}'"`
}

type RuleBody struct {
	Description *string `parser:"( 'description' @String )?"`
	AppliesTo   *string `parser:"( 'applies_to' @Ident )?"`
	When        *Expr   `parser:"( 'when' '{' @@ '}' )?"`
	Ensure      *Expr   `parser:"( 'ensure' '{' @@ '}' )?"`
	Severity    *string `parser:"( 'severity' @Ident )?"`
	Message     *string `parser:"( 'message' @String )?"`
}

type Expr struct {
	Left  *Term   `parser:"@@"`
	Op    *string `parser:"( @('and'|'or') )?"`
	Right *Term   `parser:"( @@ )?"`
}

type Term struct {
	Not   bool    `parser:"( 'not' )?"`
	Field *Field  `parser:"@@"`
	Op    *string `parser:"@('=='|'!='|'in'|'contains'|'starts_with'|'ends_with'|'matches'|'exists')"`
	Value *Value  `parser:"( @@ )?"`
}

type Field struct {
	Path []string `parser:"@Ident ( '.' @Ident )*"`
}

type Value struct {
	S    *string  `parser:"@String"`
	N    *float64 `parser:"| @Float"`
	List *[]Value `parser:"| '[' @@ ( ',' @@ )* ']'"`
}

func BuildRuleParser() (*participle.Parser[RuleSet], error) {
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
	return participle.Build[RuleSet](
		participle.Lexer(l),
		participle.Unquote("String"),
		participle.Elide("Whitespace"),
		participle.Elide("Comment"),
		participle.UseLookahead(2),
	)
}

func ToRules(rs *RuleSet) []Rule {
	var out []Rule
	for _, d := range rs.Rules {
		r := Rule{ID: d.Name}
		if d.Body != nil {
			if d.Body.Description != nil {
				r.Description = *d.Body.Description
			}
			if d.Body.AppliesTo != nil {
				r.AppliesTo = AppliesTo(*d.Body.AppliesTo)
			}
			if d.Body.Severity != nil {
				r.Severity = Severity(*d.Body.Severity)
			}
			if d.Body.Message != nil {
				r.Message = *d.Body.Message
			}
			r.When = d.Body.When
			r.Ensure = d.Body.Ensure
		}
		out = append(out, r)
	}
	return out
}

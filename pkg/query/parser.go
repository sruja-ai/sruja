package query

import (
    "github.com/alecthomas/participle/v2"
    "github.com/alecthomas/participle/v2/lexer"
)

type Query struct {
    Find  string       `parser:"'find'"`
    Type  string       `parser:"@Ident"`
    Where *WhereClause `parser:"( 'where' @@ )?"`
}

type WhereClause struct {
    Expr *Expr `parser:"@@"`
}

type Expr struct {
    Left   *Term   `parser:"@@"`
    Op     *string `parser:"( @('and'|'or') )?"`
    Right  *Term   `parser:"( @@ )?"`
}

type Term struct {
    Not    bool    `parser:"( 'not' )?"`
    Field  *Field  `parser:"@@"`
    CmpOp  *string `parser:"@('=='|'!='|'in'|'contains'|'starts_with'|'ends_with'|'matches'|'exists')"`
    Value  *Value  `parser:"( @@ )?"`
}

type Field struct {
    Path []string `parser:"@Ident ( '.' @Ident )*"`
}

type Value struct {
    String *string  `parser:"@String"`
    Number *float64 `parser:"| @Float | @Int"`
    List   *[]Value `parser:"| '[' @@ ( ',' @@ )* ']'"`
}

func BuildParser() (*participle.Parser[Query], error) {
    l := lexer.MustSimple([]lexer.SimpleRule{
        {Name: "Comment", Pattern: `//.*|/\*.*?\*/`},
        {Name: "String", Pattern: `"(\\"|[^"])*"`},
        {Name: "Float", Pattern: `\d+\.\d+`},
        {Name: "Int", Pattern: `\d+`},
        {Name: "Ident", Pattern: `[a-zA-Z_][a-zA-Z0-9_]*`},
        {Name: "Whitespace", Pattern: `\s+`},
    })
    return participle.Build[Query](
        participle.Lexer(l),
        participle.Unquote("String"),
        participle.Elide("Whitespace"),
        participle.Elide("Comment"),
        participle.UseLookahead(2),
    )
}

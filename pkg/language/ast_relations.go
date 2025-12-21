package language

import (
	"fmt"
	"strings"

	"github.com/alecthomas/participle/v2/lexer"
)

// ============================================================================
// Relations + Qualified References
// ============================================================================

// Relation represents a relationship between elements.
//
// A relation describes how one element interacts with another.
// The From and To fields can be qualified references (e.g., "System.Container").
//
// Example DSL:
//
//	User -> WebApp "Uses"
//	WebApp -> Database "Reads/Writes"
//	API -> UserService "calls" "Makes HTTP requests"
type Relation struct {
	From    QualifiedIdent `parser:"@@"`   // possibly qualified
	Arrow   string         `parser:"'->'"` // explicit arrow
	To      QualifiedIdent `parser:"@@"`   // possibly qualified
	VerbRaw *RelationVerb  `parser:"@@?"`
	Verb    *string
	Label   *string  `parser:"( @String )?"`                        // Description
	Tags    []string `parser:"( '[' @Ident ( ',' @Ident )* ']' )?"` // Semantic tags

	// Post-processed: resolved refs
	ResolvedFrom Element
	ResolvedTo   Element

	Pos lexer.Position
}

func (r *Relation) Location() SourceLocation {
	return SourceLocation{
		File:   r.Pos.Filename,
		Line:   r.Pos.Line,
		Column: r.Pos.Column,
		Offset: r.Pos.Offset,
	}
}

type QualifiedIdent struct {
	Parts []string `parser:"@Ident ( '.' @Ident )*"`
}

func (q QualifiedIdent) String() string {
	return strings.Join(q.Parts, ".")
}

type QualifiedList struct {
	Items []*QualifiedIdent `parser:"@@*"`
}

type RelationVerb struct {
	Value string
}

func (v *RelationVerb) Parse(lex *lexer.PeekingLexer) error {
	token := lex.Peek()
	if token.EOF() {
		return nil
	}
	if strings.HasPrefix(token.Value, "\"") {
		t := lex.Next()
		if len(t.Value) >= 2 {
			v.Value = t.Value[1 : len(t.Value)-1]
		} else {
			v.Value = t.Value
		}
		return nil
	}
	if token.Value == "->" || token.Value == "." || token.Value == "{" || token.Value == "}" || token.Value == "[" {
		return fmt.Errorf("not a verb")
	}
	t := lex.Next()
	next := lex.Peek()
	if next.Value == "->" || next.Value == "." {
		return fmt.Errorf("ambiguous verb followed by -> or")
	}
	v.Value = t.Value
	return nil
}

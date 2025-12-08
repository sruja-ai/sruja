package language

import (
	"fmt"
	"testing"

	"github.com/alecthomas/participle/v2"
	"github.com/alecthomas/participle/v2/lexer"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParser_ReproCycle(t *testing.T) {
	dsl := `
architecture "Cycle" {
	container A "Service A" { description "A" }
	container B "Service B" { description "B" }
	A -> B
	B -> A
}`
	p, err := NewParser()
	require.NoError(t, err)

	prog, diags, err := p.Parse("repro.sruja", dsl)
	if err != nil {
		t.Logf("Parse error: %v", err)
		for _, d := range diags {
			t.Logf("Diag: %v", d)
		}
	}
	require.NoError(t, err)
	require.NotNil(t, prog)
	require.NotNil(t, prog.Architecture)
	assert.Len(t, prog.Architecture.Relations, 2)
}

type BacktrackTest struct {
	First  string `parser:"@Ident"`
	Second string `parser:"@Ident"`
}

type FailParser struct {
	Value string
}

func (f *FailParser) Parse(lex *lexer.PeekingLexer) error {
	_ = lex.Next() // Consume one token
	return fmt.Errorf("intentional failure")
}

type BacktrackContainer struct {
	Fail *FailParser    `parser:"@@?"`
	Item *BacktrackTest `parser:"@@"`
}

func TestParser_Backtrack(t *testing.T) {
	// Input: "A B"
	// We want FailParser to consume "A" and fail.
	// If backtracking works, Item should match "A" and "B".
	// If not, Item will try to match "B" as First, and fail on EOF for Second.

	dsl := `A B`

	// We need a custom parser for this test
	lexerDef := lexer.MustSimple([]lexer.SimpleRule{
		{Name: "Ident", Pattern: `[a-zA-Z]+`},
		{Name: "Whitespace", Pattern: `\s+`},
	})

	p, err := participle.Build[BacktrackContainer](
		participle.Lexer(lexerDef),
		participle.Elide("Whitespace"),
	)
	require.NoError(t, err)

	container, err := p.ParseString("", dsl)
	if err != nil {
		t.Logf("Parse error: %v", err)
	}
	require.NoError(t, err)
	require.NotNil(t, container.Item)
	assert.Equal(t, "A", container.Item.First)
	assert.Equal(t, "B", container.Item.Second)
}

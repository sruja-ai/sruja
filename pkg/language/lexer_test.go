// pkg/language/lexer_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestNewLexer(t *testing.T) {
	input := "system API"
	l := language.NewLexer(input)
	if l == nil {
		t.Fatal("NewLexer returned nil")
	}
	// Test that lexer works by reading first token
	tok := l.NextToken()
	if tok.Type != language.TOKEN_SYSTEM {
		t.Errorf("Expected TOKEN_SYSTEM, got %v", tok.Type)
	}
	if tok.Line != 1 {
		t.Errorf("Expected line 1, got %d", tok.Line)
	}
}

func TestNextToken_Operators(t *testing.T) {
	input := "= : , ."
	l := language.NewLexer(input)

	tests := []struct {
		expectedType    language.TokenType
		expectedLiteral string
	}{
		{language.TOKEN_ASSIGN, "="},
		{language.TOKEN_COLON, ":"},
		{language.TOKEN_COMMA, ","},
		{language.TOKEN_DOT, "."},
		{language.TOKEN_EOF, ""},
	}

	for _, tt := range tests {
		tok := l.NextToken()
		if tok.Type != tt.expectedType {
			t.Errorf("Expected type %v, got %v", tt.expectedType, tok.Type)
		}
		if tok.Literal != tt.expectedLiteral {
			t.Errorf("Expected literal %q, got %q", tt.expectedLiteral, tok.Literal)
		}
	}
}

func TestNextToken_Delimiters(t *testing.T) {
	input := "{ } [ ]"
	l := language.NewLexer(input)

	tests := []struct {
		expectedType    language.TokenType
		expectedLiteral string
	}{
		{language.TOKEN_LBRACE, "{"},
		{language.TOKEN_RBRACE, "}"},
		{language.TOKEN_LBRACKET, "["},
		{language.TOKEN_RBRACKET, "]"},
		{language.TOKEN_EOF, ""},
	}

	for _, tt := range tests {
		tok := l.NextToken()
		if tok.Type != tt.expectedType {
			t.Errorf("Expected type %v, got %v", tt.expectedType, tok.Type)
		}
	}
}

func TestNextToken_Arrow(t *testing.T) {
	input := "->"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_ARROW {
		t.Errorf("Expected TOKEN_ARROW, got %v", tok.Type)
	}
	if tok.Literal != "->" {
		t.Errorf("Expected literal '->', got %q", tok.Literal)
	}
}

func TestNextToken_IllegalMinus(t *testing.T) {
	input := "-"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_ILLEGAL {
		t.Errorf("Expected TOKEN_ILLEGAL, got %v", tok.Type)
	}
}

func TestNextToken_String(t *testing.T) {
	input := `"Hello World"`
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_STRING {
		t.Errorf("Expected TOKEN_STRING, got %v", tok.Type)
	}
	if tok.Literal != "Hello World" {
		t.Errorf("Expected literal 'Hello World', got %q", tok.Literal)
	}
}

func TestNextToken_String_Empty(t *testing.T) {
	input := `""`
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_STRING {
		t.Errorf("Expected TOKEN_STRING, got %v", tok.Type)
	}
	if tok.Literal != "" {
		t.Errorf("Expected empty string, got %q", tok.Literal)
	}
}

func TestNextToken_String_Unterminated(t *testing.T) {
	input := `"Hello`
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_STRING {
		t.Errorf("Expected TOKEN_STRING, got %v", tok.Type)
	}
}

func TestNextToken_Identifier(t *testing.T) {
	input := "system"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_SYSTEM {
		t.Errorf("Expected TOKEN_SYSTEM, got %v", tok.Type)
	}
	if tok.Literal != "system" {
		t.Errorf("Expected literal 'system', got %q", tok.Literal)
	}
}

func TestNextToken_Identifier_WithUnderscore(t *testing.T) {
	input := "my_system"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_IDENT {
		t.Errorf("Expected TOKEN_IDENT, got %v", tok.Type)
	}
	if tok.Literal != "my_system" {
		t.Errorf("Expected literal 'my_system', got %q", tok.Literal)
	}
}

func TestNextToken_Identifier_WithHyphen(t *testing.T) {
	input := "my-system"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_IDENT {
		t.Errorf("Expected TOKEN_IDENT, got %v", tok.Type)
	}
	if tok.Literal != "my-system" {
		t.Errorf("Expected literal 'my-system', got %q", tok.Literal)
	}
}

func TestNextToken_Number(t *testing.T) {
	input := "123"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_NUMBER {
		t.Errorf("Expected TOKEN_NUMBER, got %v", tok.Type)
	}
	if tok.Literal != "123" {
		t.Errorf("Expected literal '123', got %q", tok.Literal)
	}
}

func TestNextToken_Number_Long(t *testing.T) {
	input := "12345"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_NUMBER {
		t.Errorf("Expected TOKEN_NUMBER, got %v", tok.Type)
	}
	if tok.Literal != "12345" {
		t.Errorf("Expected literal '12345', got %q", tok.Literal)
	}
}

func TestNextToken_Illegal(t *testing.T) {
	input := "@"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_ILLEGAL {
		t.Errorf("Expected TOKEN_ILLEGAL, got %v", tok.Type)
	}
}

func TestNextToken_EOF(t *testing.T) {
	input := ""
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_EOF {
		t.Errorf("Expected TOKEN_EOF, got %v", tok.Type)
	}
}

func TestNextToken_Whitespace(t *testing.T) {
	input := "   system"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_SYSTEM {
		t.Errorf("Expected TOKEN_SYSTEM, got %v", tok.Type)
	}
}

func TestNextToken_Newline(t *testing.T) {
	input := "\nsystem"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_SYSTEM {
		t.Errorf("Expected TOKEN_SYSTEM, got %v", tok.Type)
	}
}

func TestNextToken_Complex(t *testing.T) {
	input := `API = system "API Service" {}`
	l := language.NewLexer(input)

	tests := []struct {
		expectedType    language.TokenType
		expectedLiteral string
	}{
		{language.TOKEN_IDENT, "API"},
		{language.TOKEN_ASSIGN, "="},
		{language.TOKEN_SYSTEM, "system"},
		{language.TOKEN_STRING, "API Service"},
		{language.TOKEN_LBRACE, "{"},
		{language.TOKEN_RBRACE, "}"},
		{language.TOKEN_EOF, ""},
	}

	for _, tt := range tests {
		tok := l.NextToken()
		if tok.Type != tt.expectedType {
			t.Errorf("Expected type %v, got %v", tt.expectedType, tok.Type)
		}
		if tt.expectedLiteral != "" && tok.Literal != tt.expectedLiteral {
			t.Errorf("Expected literal %q, got %q", tt.expectedLiteral, tok.Literal)
		}
	}
}

func TestNextToken_LineNumbers(t *testing.T) {
	input := "system\nAPI"
	l := language.NewLexer(input)

	tok1 := l.NextToken()
	if tok1.Line != 1 {
		t.Errorf("Expected line 1, got %d", tok1.Line)
	}

	tok2 := l.NextToken()
	if tok2.Line != 2 {
		t.Errorf("Expected line 2, got %d", tok2.Line)
	}
}

func TestNextToken_TabsAndCarriageReturn(t *testing.T) {
	input := "\t\rsystem"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_SYSTEM {
		t.Errorf("Expected TOKEN_SYSTEM, got %v", tok.Type)
	}
}

func TestNextToken_String_WithEscapedQuote(t *testing.T) {
	// Test string that contains quote-like characters
	input := `"Hello \"World\""`
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_STRING {
		t.Errorf("Expected TOKEN_STRING, got %v", tok.Type)
	}
}

func TestNextToken_Identifier_WithNumbers(t *testing.T) {
	input := "system123"
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_IDENT {
		t.Errorf("Expected TOKEN_IDENT, got %v", tok.Type)
	}
	if tok.Literal != "system123" {
		t.Errorf("Expected literal 'system123', got %q", tok.Literal)
	}
}

func TestNextToken_MultipleTokens(t *testing.T) {
	input := `API = system "Service" {}`
	l := language.NewLexer(input)

	tokens := []language.TokenType{
		language.TOKEN_IDENT,  // API
		language.TOKEN_ASSIGN, // =
		language.TOKEN_SYSTEM, // system
		language.TOKEN_STRING, // "Service"
		language.TOKEN_LBRACE, // {
		language.TOKEN_RBRACE, // }
		language.TOKEN_EOF,
	}

	for i, expected := range tokens {
		tok := l.NextToken()
		if tok.Type != expected {
			t.Errorf("Token %d: Expected %v, got %v", i, expected, tok.Type)
		}
	}
}

func TestNextToken_SingleQuoteString(t *testing.T) {
	input := `'Hello World'`
	l := language.NewLexer(input)
	tok := l.NextToken()

	if tok.Type != language.TOKEN_STRING {
		t.Errorf("Expected TOKEN_STRING, got %v", tok.Type)
	}
	if tok.Literal != "Hello World" {
		t.Errorf("Expected literal 'Hello World', got %q", tok.Literal)
	}
}

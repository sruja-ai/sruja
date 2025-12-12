package language

type TokenType string

//nolint:revive // Token constants usage
const (
	TOKEN_EOF     TokenType = "EOF"
	TOKEN_ILLEGAL TokenType = "ILLEGAL"

	// Identifiers + Literals
	TOKEN_IDENT  TokenType = "IDENT"  // mySystem
	TOKEN_STRING TokenType = "STRING" // "My System"
	TOKEN_NUMBER TokenType = "NUMBER" // 123

	// Operators
	TOKEN_ASSIGN TokenType = "="
	TOKEN_ARROW  TokenType = "->"
	TOKEN_COLON  TokenType = ":"
	TOKEN_COMMA  TokenType = ","
	TOKEN_DOT    TokenType = "."

	// Delimiters
	TOKEN_LBRACE   TokenType = "{"
	TOKEN_RBRACE   TokenType = "}"
	TOKEN_LBRACKET TokenType = "["
	TOKEN_RBRACKET TokenType = "]"

	// Keywords
	TOKEN_WORKSPACE TokenType = "WORKSPACE"
	TOKEN_MODEL     TokenType = "MODEL"
	TOKEN_VIEWS     TokenType = "VIEWS"
	TOKEN_SYSTEM    TokenType = "SYSTEM"
	TOKEN_COMPONENT TokenType = "COMPONENT"
	TOKEN_CONTAINER TokenType = "CONTAINER"
	TOKEN_RELATION  TokenType = "RELATION"
	TOKEN_TRUE      TokenType = "TRUE"
	TOKEN_FALSE     TokenType = "FALSE"

	// New Phase 2 Tokens
	TOKEN_LIBRARY       TokenType = "LIBRARY"
	TOKEN_REQUIREMENTS  TokenType = "REQUIREMENTS"
	TOKEN_ADRS          TokenType = "ADRS"
	TOKEN_FUNCTIONAL    TokenType = "FUNCTIONAL"
	TOKEN_NONFUNCTIONAL TokenType = "NONFUNCTIONAL"
	TOKEN_CONSTRAINT    TokenType = "CONSTRAINT"
	TOKEN_STORY         TokenType = "STORY"
)

type Token struct {
	Type    TokenType
	Literal string
	Line    int
	Column  int
}

var keywords = map[string]TokenType{
	"workspace":     TOKEN_WORKSPACE,
	"model":         TOKEN_MODEL,
	"views":         TOKEN_VIEWS,
	"system":        TOKEN_SYSTEM,
	"component":     TOKEN_COMPONENT,
	"container":     TOKEN_CONTAINER,
	"true":          TOKEN_TRUE,
	"false":         TOKEN_FALSE,
	"library":       TOKEN_LIBRARY,
	"requirements":  TOKEN_REQUIREMENTS,
	"adrs":          TOKEN_ADRS,
	"functional":    TOKEN_FUNCTIONAL,
	"nonfunctional": TOKEN_NONFUNCTIONAL,
	"constraint":    TOKEN_CONSTRAINT,
	"story":         TOKEN_STORY,
}

func LookupIdent(ident string) TokenType {
	if tok, ok := keywords[ident]; ok {
		return tok
	}
	return TOKEN_IDENT
}

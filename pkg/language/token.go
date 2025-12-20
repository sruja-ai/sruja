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
	TOKEN_ASSIGN    TokenType = "="
	TOKEN_ARROW     TokenType = "->"
	TOKEN_BACKARROW TokenType = "<-"
	TOKEN_BIARROW   TokenType = "<->"
	TOKEN_COLON     TokenType = ":"
	TOKEN_COMMA     TokenType = ","
	TOKEN_DOT       TokenType = "."
	TOKEN_STAR      TokenType = "*"
	TOKEN_TAG_REF   TokenType = "TAG_REF" // #tagname

	// Delimiters
	TOKEN_LBRACE   TokenType = "{"
	TOKEN_RBRACE   TokenType = "}"
	TOKEN_LBRACKET TokenType = "["
	TOKEN_RBRACKET TokenType = "]"

	// LikeC4 Structure Keywords
	TOKEN_SPECIFICATION TokenType = "SPECIFICATION"
	TOKEN_MODEL         TokenType = "MODEL"
	TOKEN_VIEWS         TokenType = "VIEWS"
	TOKEN_VIEW          TokenType = "VIEW"

	// LikeC4 View Predicates
	TOKEN_INCLUDE TokenType = "INCLUDE"
	TOKEN_EXCLUDE TokenType = "EXCLUDE"
	TOKEN_OF      TokenType = "OF"
	TOKEN_TITLE   TokenType = "TITLE"
	TOKEN_IMPORT  TokenType = "IMPORT"
	TOKEN_FROM    TokenType = "FROM"

	// Element Types
	TOKEN_ELEMENT   TokenType = "ELEMENT"
	TOKEN_PERSON    TokenType = "PERSON"
	TOKEN_SYSTEM    TokenType = "SYSTEM"
	TOKEN_CONTAINER TokenType = "CONTAINER"
	TOKEN_COMPONENT TokenType = "COMPONENT"
	TOKEN_DATABASE  TokenType = "DATABASE"
	TOKEN_QUEUE     TokenType = "QUEUE"

	// Properties
	TOKEN_DESCRIPTION TokenType = "DESCRIPTION"
	TOKEN_TECHNOLOGY  TokenType = "TECHNOLOGY"
	TOKEN_METADATA    TokenType = "METADATA"

	// Booleans
	TOKEN_TRUE  TokenType = "TRUE"
	TOKEN_FALSE TokenType = "FALSE"

	// Sruja-specific keywords (still in use)
	TOKEN_RELATION      TokenType = "RELATION"
	TOKEN_FUNCTIONAL    TokenType = "FUNCTIONAL"
	TOKEN_NONFUNCTIONAL TokenType = "NONFUNCTIONAL"
	TOKEN_CONSTRAINT    TokenType = "CONSTRAINT"
	TOKEN_STORY         TokenType = "STORY"
	TOKEN_SCENARIO      TokenType = "SCENARIO"
	TOKEN_REQUIREMENT   TokenType = "REQUIREMENT"
	TOKEN_ADR           TokenType = "ADR"
	TOKEN_STYLE         TokenType = "STYLE"
	TOKEN_STYLES        TokenType = "STYLES"
	TOKEN_PROPERTIES    TokenType = "PROPERTIES"
	TOKEN_TECH          TokenType = "TECH"
	// Legacy plural forms (not used in parser)
	TOKEN_REQUIREMENTS TokenType = "REQUIREMENTS"
	TOKEN_ADRS         TokenType = "ADRS"
)

type Token struct {
	Type    TokenType
	Literal string
	Line    int
	Column  int
}

var keywords = map[string]TokenType{
	// LikeC4 Structure
	"specification": TOKEN_SPECIFICATION,
	"model":         TOKEN_MODEL,
	"views":         TOKEN_VIEWS,
	"view":          TOKEN_VIEW,

	// LikeC4 View Predicates
	"include": TOKEN_INCLUDE,
	"exclude": TOKEN_EXCLUDE,
	"of":      TOKEN_OF,
	"title":   TOKEN_TITLE,
	"import":  TOKEN_IMPORT,
	"from":    TOKEN_FROM,

	// Element Types
	"element":   TOKEN_ELEMENT,
	"person":    TOKEN_PERSON,
	"system":    TOKEN_SYSTEM,
	"container": TOKEN_CONTAINER,
	"component": TOKEN_COMPONENT,
	"database":  TOKEN_DATABASE,
	"queue":     TOKEN_QUEUE,

	// Properties
	"description": TOKEN_DESCRIPTION,
	"technology":  TOKEN_TECHNOLOGY,
	"metadata":    TOKEN_METADATA,

	// Booleans
	"true":  TOKEN_TRUE,
	"false": TOKEN_FALSE,

	// Sruja-specific keywords (still in use)
	"functional":    TOKEN_FUNCTIONAL,
	"nonfunctional": TOKEN_NONFUNCTIONAL,
	"constraint":    TOKEN_CONSTRAINT,
	"story":         TOKEN_STORY,
	"scenario":      TOKEN_SCENARIO,
	"requirement":   TOKEN_REQUIREMENT,
	"adr":           TOKEN_ADR,
	"style":         TOKEN_STYLE,
	"styles":        TOKEN_STYLES,
	"properties":    TOKEN_PROPERTIES,
	"tech":          TOKEN_TECH,
	// Legacy plural forms (not used in parser)
	"requirements": TOKEN_REQUIREMENTS,
	"adrs":         TOKEN_ADRS,
}

func LookupIdent(ident string) TokenType {
	if tok, ok := keywords[ident]; ok {
		return tok
	}
	return TOKEN_IDENT
}

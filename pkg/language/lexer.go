package language

type Lexer struct {
	input        string
	position     int  // current position in input (points to current char)
	readPosition int  // current reading position in input (after current char)
	ch           byte // current char under examination
	line         int
	column       int
}

func NewLexer(input string) *Lexer {
	l := &Lexer{input: input, line: 1, column: 0}
	l.readChar()
	return l
}

func (l *Lexer) readChar() {
	if l.readPosition >= len(l.input) {
		l.ch = 0
	} else {
		l.ch = l.input[l.readPosition]
	}
	l.position = l.readPosition
	l.readPosition += 1
	l.column += 1
}

func (l *Lexer) NextToken() Token {
	var tok Token

	l.skipWhitespace()

	switch l.ch {
	case '=':
		tok = newToken(TOKEN_ASSIGN, l.ch, l.line, l.column)
	case ':':
		tok = newToken(TOKEN_COLON, l.ch, l.line, l.column)
	case ',':
		tok = newToken(TOKEN_COMMA, l.ch, l.line, l.column)
	case '.':
		tok = newToken(TOKEN_DOT, l.ch, l.line, l.column)
	case '{':
		tok = newToken(TOKEN_LBRACE, l.ch, l.line, l.column)
	case '}':
		tok = newToken(TOKEN_RBRACE, l.ch, l.line, l.column)
	case '[':
		tok = newToken(TOKEN_LBRACKET, l.ch, l.line, l.column)
	case ']':
		tok = newToken(TOKEN_RBRACKET, l.ch, l.line, l.column)
	case '-':
		if l.peekChar() == '>' {
			ch := l.ch
			l.readChar()
			literal := string(ch) + string(l.ch)
			tok = Token{Type: TOKEN_ARROW, Literal: literal, Line: l.line, Column: l.column - 1}
		} else {
			tok = newToken(TOKEN_ILLEGAL, l.ch, l.line, l.column)
		}
	case '"':
		tok.Type = TOKEN_STRING
		tok.Literal = l.readString()
		tok.Line = l.line
		tok.Column = l.column // Approximate
	case 0:
		tok.Literal = ""
		tok.Type = TOKEN_EOF
	default:
		switch {
		case isLetter(l.ch):
			tok.Literal = l.readIdentifier()
			tok.Type = LookupIdent(tok.Literal)
			tok.Line = l.line
			tok.Column = l.column // Approximate
			return tok
		case isDigit(l.ch):
			tok.Type = TOKEN_NUMBER
			tok.Literal = l.readNumber()
			tok.Line = l.line
			tok.Column = l.column
			return tok
		default:
			tok = newToken(TOKEN_ILLEGAL, l.ch, l.line, l.column)
		}
	}

	l.readChar()
	return tok
}

func newToken(tokenType TokenType, ch byte, line, col int) Token {
	return Token{Type: tokenType, Literal: string(ch), Line: line, Column: col}
}

func (l *Lexer) readIdentifier() string {
	position := l.position
	for isLetter(l.ch) || isDigit(l.ch) || l.ch == '_' || l.ch == '-' {
		l.readChar()
	}
	return l.input[position:l.position]
}

func (l *Lexer) readNumber() string {
	position := l.position
	for isDigit(l.ch) {
		l.readChar()
	}
	return l.input[position:l.position]
}

func (l *Lexer) readString() string {
	position := l.position + 1
	for {
		l.readChar()
		if l.ch == '"' || l.ch == 0 {
			break
		}
	}
	return l.input[position:l.position]
}

func (l *Lexer) skipWhitespace() {
	for l.ch == ' ' || l.ch == '\t' || l.ch == '\n' || l.ch == '\r' {
		if l.ch == '\n' {
			l.line++
			l.column = 0
		}
		l.readChar()
	}
}

func (l *Lexer) peekChar() byte {
	if l.readPosition >= len(l.input) {
		return 0
	}
	return l.input[l.readPosition]
}

func isLetter(ch byte) bool {
	return 'a' <= ch && ch <= 'z' || 'A' <= ch && ch <= 'Z' || ch == '_'
}

func isDigit(ch byte) bool {
	return '0' <= ch && ch <= '9'
}

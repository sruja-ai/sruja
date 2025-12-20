package language

import (
	"github.com/alecthomas/participle/v2/lexer"
	"testing"
)

func TestViewLocationMethods(t *testing.T) {
	vb := &ViewBlock{Pos: lexer.Position{Filename: "views.sruja", Line: 10, Column: 2, Offset: 100}}
	v := &View{Pos: lexer.Position{Filename: "views.sruja", Line: 12, Column: 4, Offset: 140}}
	ve := &ViewExpression{Pos: lexer.Position{Filename: "views.sruja", Line: 14, Column: 6, Offset: 180}}
	sb := &StylesBlock{Pos: lexer.Position{Filename: "views.sruja", Line: 20, Column: 2, Offset: 260}}
	es := &ElementStyle{Pos: lexer.Position{Filename: "views.sruja", Line: 22, Column: 4, Offset: 290}}

	tests := []struct {
		name string
		got  SourceLocation
		want SourceLocation
	}{
		{"ViewBlock", vb.Location(), SourceLocation{File: "views.sruja", Line: 10, Column: 2, Offset: 100}},
		{"View", v.Location(), SourceLocation{File: "views.sruja", Line: 12, Column: 4, Offset: 140}},
		{"ViewExpression", ve.Location(), SourceLocation{File: "views.sruja", Line: 14, Column: 6, Offset: 180}},
		{"StylesBlock", sb.Location(), SourceLocation{File: "views.sruja", Line: 20, Column: 2, Offset: 260}},
		{"ElementStyle", es.Location(), SourceLocation{File: "views.sruja", Line: 22, Column: 4, Offset: 290}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.got.File != tt.want.File || tt.got.Line != tt.want.Line || tt.got.Column != tt.want.Column || tt.got.Offset != tt.want.Offset {
				t.Fatalf("Location mismatch: got %+v want %+v", tt.got, tt.want)
			}
		})
	}
}

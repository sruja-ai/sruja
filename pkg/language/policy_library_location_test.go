package language

import (
    "testing"
    "github.com/alecthomas/participle/v2/lexer"
)

func TestPolicyLibrarySharedArtifactLocation(t *testing.T) {
    pol := &Policy{Pos: lexer.Position{Filename: "lib.sruja", Line: 3, Column: 5, Offset: 30}}
    lib := &Library{Pos: lexer.Position{Filename: "lib.sruja", Line: 1, Column: 1, Offset: 0}}
    sa := &SharedArtifact{Pos: lexer.Position{Filename: "lib.sruja", Line: 8, Column: 2, Offset: 120}}

    cases := []struct{
        name string
        got  SourceLocation
        want SourceLocation
    }{
        {"Policy", pol.Location(), SourceLocation{File: "lib.sruja", Line: 3, Column: 5, Offset: 30}},
        {"Library", lib.Location(), SourceLocation{File: "lib.sruja", Line: 1, Column: 1, Offset: 0}},
        {"SharedArtifact", sa.Location(), SourceLocation{File: "lib.sruja", Line: 8, Column: 2, Offset: 120}},
    }

    for _, c := range cases {
        t.Run(c.name, func(t *testing.T) {
            if c.got != c.want {
                t.Fatalf("Location mismatch: got %+v want %+v", c.got, c.want)
            }
        })
    }
}


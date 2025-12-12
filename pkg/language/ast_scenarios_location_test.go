package language

import (
    "testing"
    "github.com/alecthomas/participle/v2/lexer"
)

func TestScenarioAndFlowLocationMethods(t *testing.T) {
    sc := &Scenario{Pos: lexer.Position{Filename: "scenario.sruja", Line: 5, Column: 1, Offset: 50}}
    step := &ScenarioStep{Pos: lexer.Position{Filename: "scenario.sruja", Line: 7, Column: 3, Offset: 80}}
    fl := &Flow{Pos: lexer.Position{Filename: "scenario.sruja", Line: 15, Column: 2, Offset: 160}}

    tests := []struct{
        name string
        got  SourceLocation
        want SourceLocation
    }{
        {"Scenario", sc.Location(), SourceLocation{File: "scenario.sruja", Line: 5, Column: 1, Offset: 50}},
        {"ScenarioStep", step.Location(), SourceLocation{File: "scenario.sruja", Line: 7, Column: 3, Offset: 80}},
        {"Flow", fl.Location(), SourceLocation{File: "scenario.sruja", Line: 15, Column: 2, Offset: 160}},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            if tt.got.File != tt.want.File || tt.got.Line != tt.want.Line || tt.got.Column != tt.want.Column || tt.got.Offset != tt.want.Offset {
                t.Fatalf("Location mismatch: got %+v want %+v", tt.got, tt.want)
            }
        })
    }
}


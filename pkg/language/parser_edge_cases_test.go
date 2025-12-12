package language

import (
	"strings"
	"testing"
)

func TestParser_EdgeCases(t *testing.T) {
	tests := []struct {
		name      string
		input     string
		shouldErr bool
		errMsg    string
	}{
        {
            name:      "Empty Input",
            input:     "",
            shouldErr: true,
            errMsg:    "sub-expression",
        },
		{
			name:      "Unclosed Block",
			input:     `system S1 {`,
			shouldErr: true,
			errMsg:    "unexpected token",
		},
        {
            name:      "Missing ID",
            input:     `system "Label"`,
            shouldErr: true,
            errMsg:    "sub-expression",
        },
        {
            name:      "Invalid Keyword",
            input:     `unknown S1 "Label"`,
            shouldErr: true,
            errMsg:    "sub-expression",
        },
		{
			name:      "Unclosed String",
			input:     `system S1 "Label`,
			shouldErr: true,
			errMsg:    "invalid input text",
		},
		{
			name: "Nested System (Valid in AST but maybe not logic)",
			input: `
				system S1 {
					system S2 "Nested"
				}
			`,
			shouldErr: true, // Grammar likely doesn't allow system inside system
			errMsg:    "unexpected token",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p, err := NewParser()
			if err != nil {
				t.Fatalf("Failed to create parser: %v", err)
			}
			_, diags, err := p.Parse("test.sruja", tt.input)
			if tt.shouldErr {
				if len(diags) == 0 && err == nil {
					t.Error("Expected error or diagnostics, got nil")
				} else if tt.errMsg != "" {
					found := false
					if err != nil && strings.Contains(err.Error(), tt.errMsg) {
						found = true
					}
					if !found {
						for _, d := range diags {
							if strings.Contains(d.Message, tt.errMsg) {
								found = true
								break
							}
						}
					}
					if !found {
						t.Errorf("Expected error containing %q, got err=%v, diags=%v", tt.errMsg, err, diags)
					}
				}
			} else {
				if err != nil || len(diags) > 0 {
					t.Errorf("Unexpected error: %v, diags: %v", err, diags)
				}
			}
		})
	}
}

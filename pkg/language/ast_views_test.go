// pkg/language/ast_views_test.go
package language

import (
	"fmt"
	"testing"
)

func TestParser_ViewsBlock(t *testing.T) {
	tests := []struct {
		name    string
		dsl     string
		wantErr bool
		checkFn func(*Architecture) error
	}{
		{
			name: "Simple views block",
			dsl: `
architecture "Test" {
    system Shop "Shop" {
        container WebApp "Web Application"
    }
    
    views {
        container Shop "Container View" {
            include *
        }
    }
}`,
			wantErr: false,
			checkFn: func(arch *Architecture) error {
				if arch.Views == nil {
					return fmt.Errorf("Views block is nil")
				}
				if len(arch.Views.Views) != 1 {
					return fmt.Errorf("Expected 1 view, got %d", len(arch.Views.Views))
				}
				view := arch.Views.Views[0]
				if view.Type != "container" {
					return fmt.Errorf("Expected view type 'container', got '%s'", view.Type)
				}
				if view.Scope.String() != "Shop" {
					return fmt.Errorf("Expected scope 'Shop', got '%s'", view.Scope.String())
				}
				if len(view.Expressions) != 1 {
					return fmt.Errorf("Expected 1 expression, got %d", len(view.Expressions))
				}
				expr := view.Expressions[0]
				if expr.Type != "include" {
					return fmt.Errorf("Expected expression type 'include', got '%s'", expr.Type)
				}
				if expr.Wildcard == nil {
					return fmt.Errorf("Expected wildcard '*', got nil")
				}
				return nil
			},
		},
		{
			name: "Views block with include elements",
			dsl: `
architecture "Test" {
    system Shop "Shop" {
        container WebApp "Web Application"
        container API "API Gateway"
    }
    
    views {
        container Shop "API Focus" {
            include Shop.API Shop.WebApp
        }
    }
}`,
			wantErr: false,
			checkFn: func(arch *Architecture) error {
				if arch.Views == nil {
					return fmt.Errorf("Views block is nil")
				}
				view := arch.Views.Views[0]
				if len(view.Expressions) != 1 {
					return fmt.Errorf("Expected 1 expression, got %d", len(view.Expressions))
				}
				expr := view.Expressions[0]
				if len(expr.Elements) != 2 {
					return fmt.Errorf("Expected 2 elements, got %d", len(expr.Elements))
				}
				return nil
			},
		},
		{
			name: "Views block with styles",
			dsl: `
architecture "Test" {
    system Shop "Shop" {
        container DB "Database" {
            tags ["Database"]
        }
    }
    
    views {
        container Shop "Container View" {
            include *
        }
        
        styles {
            element "Database" {
                shape "cylinder"
                color "#ff0000"
            }
        }
    }
}`,
			wantErr: false,
			checkFn: func(arch *Architecture) error {
				if arch.Views == nil {
					return fmt.Errorf("Views block is nil")
				}
				// Styles block should be in the ViewBlock, not in individual views
				if arch.Views.Styles == nil {
					return fmt.Errorf("Expected styles block, got nil")
				}
				if len(arch.Views.Styles.Styles) != 1 {
					return fmt.Errorf("Expected 1 style, got %d", len(arch.Views.Styles.Styles))
				}
				style := arch.Views.Styles.Styles[0]
				if style.Target != "element" {
					return fmt.Errorf("Expected target 'element', got '%s'", style.Target)
				}
				if style.Tag != "Database" {
					return fmt.Errorf("Expected tag 'Database', got '%s'", style.Tag)
				}
				return nil
			},
		},
		{
			name: "Architecture without views block",
			dsl: `
architecture "Test" {
    system Shop "Shop" {
        container WebApp "Web Application"
    }
}`,
			wantErr: false,
			checkFn: func(arch *Architecture) error {
				if arch.Views != nil {
					return fmt.Errorf("Expected no views block, but got one")
				}
				return nil
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			parser, err := NewParser()
			if err != nil {
				t.Fatalf("Failed to create parser: %v", err)
			}

			program, _, err := parser.Parse("test.sruja", tt.dsl)
			if (err != nil) != tt.wantErr {
				t.Fatalf("Parse() error = %v, wantErr %v", err, tt.wantErr)
			}

			if err != nil {
				return
			}

			if program.Architecture == nil {
				t.Fatal("Architecture is nil")
			}

			if tt.checkFn != nil {
				if err := tt.checkFn(program.Architecture); err != nil {
					t.Error(err)
				}
			}
		})
	}
}

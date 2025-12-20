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
		checkFn func(*Program) error // Updated to use Program instead of Architecture
	}{
		{
			name: "Simple views block",
			dsl: `
specification {
	element system
	element container
}
model {
	Shop = system "Shop" {
		WebApp = container "Web Application"
	}
}
views {
	view index {
		include Shop.*
	}
}`,
			wantErr: false,
			checkFn: func(p *Program) error {
				if p.Views == nil {
					return fmt.Errorf("Views block is nil")
				}
				// Filter for Views (Items can be Styles or Views)
				var views []*LikeC4ViewDef
				for _, item := range p.Views.Items {
					if item.View != nil {
						views = append(views, item.View)
					}
				}
				if len(views) != 1 {
					return fmt.Errorf("Expected 1 view, got %d", len(views))
				}
				view := views[0]
				if view.Name == nil || *view.Name != "index" {
					return fmt.Errorf("Expected view name 'index', got %v", view.Name)
				}
				// Check includes
				if view.Body == nil || len(view.Body.Items) == 0 {
					return fmt.Errorf("Expected view body items")
				}
				includeFound := false
				for _, item := range view.Body.Items {
					if item.Include != nil {
						includeFound = true
						if len(item.Include.Expressions) != 1 {
							return fmt.Errorf("Expected 1 include expression, got %d", len(item.Include.Expressions))
						}
						// Verify explicit Shop.* wildcard if possible, or just parse success
					}
				}
				if !includeFound {
					return fmt.Errorf("Expected 'include' predicate")
				}
				return nil
			},
		},
		{
			name: "Views block with include elements",
			dsl: `
specification {
	element system
	element container
}
model {
	Shop = system "Shop" {
		WebApp = container "Web Application"
		API = container "API Gateway"
	}
}
views {
	view apiFocus {
		include Shop.API Shop.WebApp
	}
}`,
			wantErr: false,
			checkFn: func(p *Program) error {
				if p.Views == nil {
					return fmt.Errorf("Views block is nil")
				}
				// Find view
				var view *LikeC4ViewDef
				for _, item := range p.Views.Items {
					if item.View != nil {
						view = item.View
						break
					}
				}
				if view == nil {
					return fmt.Errorf("Expected 1 view, got 0")
				}

				if view.Body == nil || len(view.Body.Items) == 0 {
					return fmt.Errorf("Expected view body items")
				}
				// Find include
				var include *IncludePredicate
				for _, item := range view.Body.Items {
					if item.Include != nil {
						include = item.Include
						break
					}
				}
				if include == nil {
					return fmt.Errorf("Expected include predicate")
				}
				// Check elements count (flatten expressions)
				count := 0
				for _, expr := range include.Expressions {
					// each expr is a ViewExpr
					// Check if it's wildcard or specific?
					// AST structure for IncludePredicate: Expressions []ViewExpr
					// ViewExpr union: Wildcard, Element ...
					// The DSL is "include Shop.API Shop.WebApp"
					// This likely parses as multiple expressions if comma separated or space separated?
					// Parser rule: 'include' @@ ( ','? @@ )*
					// So it should be 2 expressions
					count++
					_ = expr
				}
				if count != 2 {
					return fmt.Errorf("Expected 2 include expressions, got %d", count)
				}
				return nil
			},
		},
		{
			name: "Views block with styles",
			dsl: `
specification {
	element system
	element container
	tag Database
}
model {
	Shop = system "Shop" {
		DB = container "Database" #Database
	}
}
views {
	view index {
		include Shop.*
	}
	styles {
		element #Database {
			shape cylinder
			color "#ff0000"
		}
	}
}`,
			wantErr: false,
			checkFn: func(p *Program) error {
				if p.Views == nil {
					return fmt.Errorf("Views block is nil")
				}
				// Find styles
				var styles *StyleDecl
				for _, item := range p.Views.Items {
					if item.Styles != nil {
						styles = item.Styles
						break
					}
				}
				if styles == nil {
					return fmt.Errorf("Expected styles block, got nil")
				}
				if styles.Body == nil {
					return fmt.Errorf("Expected styles body")
				}
				if len(styles.Body.Entries) != 1 {
					return fmt.Errorf("Expected 1 style entry, got %d", len(styles.Body.Entries))
				}
				entry := styles.Body.Entries[0]
				if entry.Key != "element" {
					return fmt.Errorf("Expected key 'element', got '%s'", entry.Key)
				}
				// Value should be #Database (TagRef)
				if entry.Value == nil || *entry.Value != "#Database" {
					return fmt.Errorf("Expected value '#Database', got %v", entry.Value)
				}
				// Check inner properties
				if entry.Body == nil || len(entry.Body.Entries) != 2 {
					return fmt.Errorf("Expected 2 inner style properties")
				}
				return nil
			},
		},
		{
			name: "Model without views block",
			dsl: `
specification {
	element system
	element container
}
model {
	Shop = system "Shop" {
		WebApp = container "Web Application"
	}
}`,
			wantErr: false,
			checkFn: func(p *Program) error {
				if p.Views != nil {
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

			// Architecture removed - these tests need to be migrated to LikeC4 syntax
			if tt.checkFn != nil {
				if err := tt.checkFn(program); err != nil {
					t.Error(err)
				}
			}
		})
	}
}

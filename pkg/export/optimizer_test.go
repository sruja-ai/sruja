// pkg/export/optimizer_test.go
package export

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func parseDSL(t *testing.T, dsl string) *language.Program {
	t.Helper()
	p, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	return program
}

func TestEstimateTokens(t *testing.T) {
	tests := []struct {
		name    string
		content string
		want    int
	}{
		{
			name:    "empty string",
			content: "",
			want:    0,
		},
		{
			name:    "short string",
			content: "Hello world",
			want:    2, // 11 chars / 4 = 2
		},
		{
			name:    "long string",
			content: strings.Repeat("a", 400),
			want:    100, // 400 / 4 = 100
		},
		{
			name:    "typical description",
			content: "This is a typical description that might appear in an architecture document.",
			want:    19, // 76 chars / 4 = 19
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := EstimateTokens(tt.content)
			if got != tt.want {
				t.Errorf("EstimateTokens(%q) = %d, want %d", tt.content, got, tt.want)
			}
		})
	}
}

func TestTokenOptimizer_ShouldOptimize(t *testing.T) {
	tests := []struct {
		name         string
		limit        int
		current      int
		wantOptimize bool
	}{
		{
			name:         "no limit",
			limit:        0,
			current:      10000,
			wantOptimize: false,
		},
		{
			name:         "under limit",
			limit:        8000,
			current:      1000,
			wantOptimize: false,
		},
		{
			name:         "at limit",
			limit:        8000,
			current:      8000,
			wantOptimize: false,
		},
		{
			name:         "over limit",
			limit:        8000,
			current:      10000,
			wantOptimize: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			optimizer := NewTokenOptimizer(tt.limit)
			got := optimizer.ShouldOptimize(tt.current)
			if got != tt.wantOptimize {
				t.Errorf("ShouldOptimize(limit=%d, current=%d) = %v, want %v",
					tt.limit, tt.current, got, tt.wantOptimize)
			}
		})
	}
}

func TestTokenOptimizer_TruncateDescription(t *testing.T) {
	tests := []struct {
		name      string
		desc      string
		maxTokens int
		want      string
	}{
		{
			name:      "no truncation needed",
			desc:      "Short description",
			maxTokens: 10,
			want:      "Short description",
		},
		{
			name:      "truncation needed",
			desc:      strings.Repeat("word ", 100), // ~500 chars
			maxTokens: 50,                           // ~200 chars
			want:      strings.Repeat("word ", 49) + "...",
		},
		{
			name:      "zero max tokens",
			desc:      "Some description",
			maxTokens: 0,
			want:      "Some description",
		},
		{
			name:      "empty description",
			desc:      "",
			maxTokens: 10,
			want:      "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			optimizer := NewTokenOptimizer(0)
			got := optimizer.TruncateDescription(tt.desc, tt.maxTokens)

			// Check that result is not longer than expected
			maxChars := tt.maxTokens * 4
			if tt.maxTokens > 0 && len(got) > maxChars+10 { // Allow some margin
				t.Errorf("TruncateDescription() result too long: %d chars (max: %d)",
					len(got), maxChars)
			}

			// Check that it ends with ellipsis if truncated
			if len(got) < len(tt.desc) && !strings.HasSuffix(got, "...") {
				t.Error("Expected truncated description to end with '...'")
			}
		})
	}
}

func TestPrioritizeElements(t *testing.T) {
	dsl := `
		System = kind "System"
		Container = kind "Container"
		Component = kind "Component"
		Sys1 = System "System 1" {
			Cont1 = Container "Container 1" {
				Comp1 = Component "Component 1"
			}
		}
		Sys2 = System "System 2" {
			Cont2 = Container "Container 2"
		}
	`

	program := parseDSL(t, dsl)
	systems, containers, components := PrioritizeElements(program)

	if len(systems) < 2 {
		t.Errorf("Expected at least 2 systems, got %d", len(systems))
	}

	if len(containers) < 2 {
		t.Errorf("Expected at least 2 containers, got %d", len(containers))
	}

	if len(components) < 1 {
		t.Errorf("Expected at least 1 component, got %d", len(components))
	}

	// Verify systems are prioritized
	if len(systems) == 0 {
		t.Error("Expected systems to be found")
	}
}

func TestFilterByScope_Full(t *testing.T) {
	dsl := `
		System = kind "System"
		OrderService = System "Order Service" {
			description "Handles orders"
		}
		PaymentService = System "Payment Service" {
			description "Handles payments"
		}
	`

	program := parseDSL(t, dsl)
	filtered := FilterByScope(program, "full", "")

	if filtered == nil {
		t.Fatal("Expected non-nil filtered program")
	}

	// Full scope should return original or equivalent
	if filtered.Model == nil {
		t.Error("Expected filtered program to have model")
	}
}

func TestFilterByScope_System(t *testing.T) {
	dsl := `
		System = kind "System"
		Container = kind "Container"
		OrderService = System "Order Service" {
			description "Handles orders"
			API = Container "API" {
				technology "Go"
			}
		}
		PaymentService = System "Payment Service" {
			description "Handles payments"
		}
	`

	program := parseDSL(t, dsl)
	filtered := FilterByScope(program, "system", "OrderService")

	if filtered == nil {
		t.Fatal("Expected non-nil filtered program")
	}

	if filtered.Model == nil {
		t.Error("Expected filtered program to have model")
	}

	// Should contain the scoped system
	if len(filtered.Model.Items) == 0 {
		t.Error("Expected filtered program to contain items")
	}
}

func TestFilterByScope_NotFound(t *testing.T) {
	dsl := `
		System = kind "System"
		OrderService = System "Order Service" {
			description "Handles orders"
		}
	`

	program := parseDSL(t, dsl)
	filtered := FilterByScope(program, "system", "NonExistent")

	if filtered == nil {
		t.Fatal("Expected non-nil filtered program")
	}

	// Should return empty program when element not found
	if filtered.Model == nil {
		t.Error("Expected filtered program to have model")
	}
}

func TestFilterByScope_EmptyProgram(t *testing.T) {
	dsl := ``

	program := parseDSL(t, dsl)
	filtered := FilterByScope(program, "system", "OrderService")

	if filtered == nil {
		t.Fatal("Expected non-nil filtered program")
	}

	if filtered.Model == nil {
		t.Error("Expected filtered program to have model")
	}
}

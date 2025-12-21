package engine

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

// BenchmarkScorer benchmarks the score calculation for architectures.
func BenchmarkScorer(b *testing.B) {
	parser, err := language.NewParser()
	if err != nil {
		b.Fatalf("Failed to create parser: %v", err)
	}

	dsl := `model {
		API = system "API Gateway" {
			description "Main API entry point"
			WebApp = container "Web Application" {
				technology "Go"
				description "Serves HTTP requests"
			}
			Database = container "PostgreSQL" {
				technology "PostgreSQL"
				description "Primary data store"
			}
			WebApp -> Database "reads/writes"
		}
		Auth = system "Authentication" {
			description "Handles authentication"
			AuthService = container "Auth Service" {
				technology "Go"
			}
		}
		API -> Auth "authenticates via"
	}`

	program, _, err := parser.Parse("bench.sruja", dsl)
	if err != nil {
		b.Fatalf("Parse error: %v", err)
	}

	scorer := NewScorer()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		_ = scorer.CalculateScore(program)
	}
}

// BenchmarkValidator benchmarks the validator with all default rules.
func BenchmarkValidator(b *testing.B) {
	parser, err := language.NewParser()
	if err != nil {
		b.Fatalf("Failed to create parser: %v", err)
	}

	dsl := `model {
		API = system "API Gateway" {
			description "Main API entry point"
			WebApp = container "Web Application" {
				technology "Go"
				description "Serves HTTP requests"
			}
			Database = container "PostgreSQL" {
				technology "PostgreSQL"
				description "Primary data store"
			}
			WebApp -> Database "reads/writes"
		}
		Auth = system "Authentication" {
			description "Handles authentication"  
			AuthService = container "Auth Service" {
				technology "Go"
				description "Authentication microservice"
			}
		}
		API -> Auth "authenticates via"
	}`

	program, _, err := parser.Parse("bench.sruja", dsl)
	if err != nil {
		b.Fatalf("Parse error: %v", err)
	}

	validator := NewValidator()
	validator.RegisterDefaultRules()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		_ = validator.Validate(program)
	}
}

// BenchmarkCycleDetection benchmarks cycle detection on a graph with no cycles.
func BenchmarkCycleDetection(b *testing.B) {
	parser, err := language.NewParser()
	if err != nil {
		b.Fatalf("Failed to create parser: %v", err)
	}

	dsl := `model {
		A = system "System A" { description "First" }
		B = system "System B" { description "Second" }
		C = system "System C" { description "Third" }
		D = system "System D" { description "Fourth" }
		E = system "System E" { description "Fifth" }
		A -> B "uses"
		B -> C "uses"
		C -> D "uses"
		D -> E "uses"
	}`

	program, _, err := parser.Parse("bench.sruja", dsl)
	if err != nil {
		b.Fatalf("Parse error: %v", err)
	}

	rule := &CycleDetectionRule{}

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		_ = rule.Validate(program)
	}
}

// BenchmarkResolver benchmarks reference resolution.
func BenchmarkResolver(b *testing.B) {
	parser, err := language.NewParser()
	if err != nil {
		b.Fatalf("Failed to create parser: %v", err)
	}

	dsl := `model {
		API = system "API Gateway" {
			WebApp = container "Web Application" {
				technology "Go"
				Handler = component "Request Handler" {
					technology "Go"
				}
				Service = component "Business Service" {
					technology "Go"
				}
				Handler -> Service "calls"
			}
			Database = container "PostgreSQL" {
				technology "PostgreSQL"
			}
			WebApp -> Database "reads"
		}
	}`

	program, _, err := parser.Parse("bench.sruja", dsl)
	if err != nil {
		b.Fatalf("Parse error: %v", err)
	}

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		// Create new resolver each time (simulates real usage)
		_ = NewResolverFromModel(program.Model)
	}
}

// BenchmarkBuildQualifiedID benchmarks the pooled qualified ID builder.
func BenchmarkBuildQualifiedID(b *testing.B) {
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		_ = BuildQualifiedID("system", "container", "component")
	}
}

// BenchmarkStringBuilderPool benchmarks pooled vs non-pooled string builders.
func BenchmarkStringBuilderPool(b *testing.B) {
	b.Run("Pooled", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			sb := GetStringBuilder()
			sb.WriteString("test message with some content")
			_ = sb.String()
			PutStringBuilder(sb)
		}
	})

	b.Run("NonPooled", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			var sb strings.Builder
			sb.WriteString("test message with some content")
			_ = sb.String()
		}
	})
}

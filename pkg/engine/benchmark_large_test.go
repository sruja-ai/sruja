package engine

import (
	"fmt"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

// BenchmarkLargeArchitecture benchmarks validation on a large architecture.
func BenchmarkLargeArchitecture(b *testing.B) {
	parser, err := language.NewParser()
	if err != nil {
		b.Fatalf("Failed to create parser: %v", err)
	}

	// Generate a 50-element architecture
	dsl := generateLargeArchitecture(50)

	program, _, err := parser.Parse("large_bench.sruja", dsl)
	if err != nil {
		b.Fatalf("Parse error: %v", err)
	}

	b.Run("Validator_50Elements", func(b *testing.B) {
		validator := NewValidator()
		validator.RegisterDefaultRules()

		b.ResetTimer()
		b.ReportAllocs()

		for i := 0; i < b.N; i++ {
			_ = validator.Validate(program)
		}
	})

	b.Run("Scorer_50Elements", func(b *testing.B) {
		scorer := NewScorer()

		b.ResetTimer()
		b.ReportAllocs()

		for i := 0; i < b.N; i++ {
			_ = scorer.CalculateScore(program)
		}
	})

	b.Run("Resolver_50Elements", func(b *testing.B) {
		b.ResetTimer()
		b.ReportAllocs()

		for i := 0; i < b.N; i++ {
			_ = NewResolverFromModel(program.Model)
		}
	})
}

// BenchmarkDeepNesting benchmarks traversal on deeply nested architectures.
func BenchmarkDeepNesting(b *testing.B) {
	parser, err := language.NewParser()
	if err != nil {
		b.Fatalf("Failed to create parser: %v", err)
	}

	// Generate deeply nested architecture (5 levels)
	dsl := generateDeepArchitecture(5)

	program, _, err := parser.Parse("deep_bench.sruja", dsl)
	if err != nil {
		b.Fatalf("Parse error: %v", err)
	}

	b.Run("CycleDetection_DeepNesting", func(b *testing.B) {
		rule := &CycleDetectionRule{}

		b.ResetTimer()
		b.ReportAllocs()

		for i := 0; i < b.N; i++ {
			_ = rule.Validate(program)
		}
	})
}

// BenchmarkMapPools benchmarks pooled vs non-pooled map operations.
func BenchmarkMapPools(b *testing.B) {
	b.Run("StringBoolMap_Pooled", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			m := GetStringBoolMap()
			for j := 0; j < 20; j++ {
				(*m)[fmt.Sprintf("key%d", j)] = true
			}
			PutStringBoolMap(m)
		}
	})

	b.Run("StringBoolMap_NonPooled", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			m := make(map[string]bool, 32)
			for j := 0; j < 20; j++ {
				m[fmt.Sprintf("key%d", j)] = true
			}
			_ = m
		}
	})
}

// generateLargeArchitecture creates a DSL string with N systems and containers.
func generateLargeArchitecture(n int) string {
	var sb strings.Builder
	sb.WriteString("model {\n")

	systemsPerRow := 5
	containersPerSystem := 3

	for i := 0; i < n/containersPerSystem; i++ {
		sb.WriteString(fmt.Sprintf("\tSys%d = system \"System %d\" {\n", i, i))
		sb.WriteString(fmt.Sprintf("\t\tdescription \"System %d description\"\n", i))

		for j := 0; j < containersPerSystem && (i*containersPerSystem+j) < n; j++ {
			sb.WriteString(fmt.Sprintf("\t\tCont%d_%d = container \"Container %d.%d\" {\n", i, j, i, j))
			sb.WriteString(fmt.Sprintf("\t\t\ttechnology \"Tech%d\"\n", j))
			sb.WriteString(fmt.Sprintf("\t\t\tdescription \"Container %d.%d description\"\n", i, j))
			sb.WriteString("\t\t}\n")
		}

		sb.WriteString("\t}\n")

		// Add relations between systems
		if i > 0 && i%systemsPerRow != 0 {
			sb.WriteString(fmt.Sprintf("\tSys%d -> Sys%d \"uses\"\n", i-1, i))
		}
	}

	sb.WriteString("}\n")
	return sb.String()
}

// generateDeepArchitecture creates a deeply nested DSL structure.
func generateDeepArchitecture(depth int) string {
	var sb strings.Builder
	sb.WriteString("model {\n")

	indent := "\t"
	for i := 0; i < depth; i++ {
		kind := "system"
		if i == 1 {
			kind = "container"
		} else if i >= 2 {
			kind = "component"
		}

		sb.WriteString(fmt.Sprintf("%sLevel%d = %s \"Level %d\" {\n", indent, i, kind, i))
		sb.WriteString(fmt.Sprintf("%s\tdescription \"Level %d\"\n", indent, i))
		if kind != "system" {
			sb.WriteString(fmt.Sprintf("%s\ttechnology \"Lang%d\"\n", indent, i))
		}
		indent += "\t"
	}

	// Close all levels
	for i := depth - 1; i >= 0; i-- {
		indent = strings.Repeat("\t", i+1)
		sb.WriteString(fmt.Sprintf("%s}\n", indent))
	}

	sb.WriteString("}\n")
	return sb.String()
}

// BenchmarkCollectElements benchmarks the traversal helper.
func BenchmarkCollectElements(b *testing.B) {
	parser, err := language.NewParser()
	if err != nil {
		b.Fatalf("Failed to create parser: %v", err)
	}

	dsl := generateLargeArchitecture(100)
	program, _, err := parser.Parse("collect_bench.sruja", dsl)
	if err != nil {
		b.Fatalf("Parse error: %v", err)
	}

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		_, _ = collectElements(program.Model)
	}
}

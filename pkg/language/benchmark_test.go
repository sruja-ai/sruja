package language

import (
	"fmt"
	"strings"
	"testing"
)

func BenchmarkParser(b *testing.B) {
	// Generate a large Sruja DSL content
	var sb strings.Builder
	sb.WriteString("architecture \"Benchmark Arch\" {\n")

	// Add systems
	for i := 0; i < 100; i++ {
		sb.WriteString(fmt.Sprintf("\tsystem System%d \"Benchmark System %d\" {\n", i, i))
		sb.WriteString(fmt.Sprintf("\t\tdescription \"Description for system %d\"\n", i))

		// Add containers
		for j := 0; j < 5; j++ {
			sb.WriteString(fmt.Sprintf("\t\tcontainer Container%d \"Benchmark Container %d\" {\n", j, j))
            sb.WriteString("\t\t\ttechnology \"Go\"\n")
			sb.WriteString("\t\t}\n")
		}
		sb.WriteString("\t}\n")
	}

	// Add relations
	for i := 0; i < 99; i++ {
		sb.WriteString(fmt.Sprintf("\tSystem%d -> System%d \"uses\"\n", i, i+1))
	}

	sb.WriteString("}\n")
	content := sb.String()

	parser, err := NewParser()
	if err != nil {
		b.Fatalf("Failed to create parser: %v", err)
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _, err := parser.Parse("benchmark.sruja", content)
		if err != nil {
			b.Fatalf("Parse error: %v", err)
		}
	}
}

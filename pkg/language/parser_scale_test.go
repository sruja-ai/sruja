package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_Scale(t *testing.T) {
	dsl := `
architecture "Scale Test" {
    system MySystem "System" {
        container WebApp "Web App" {
            technology "Go"
            scale {
                min 3
                max 10
                metric "cpu > 80%"
            }
        }

        container Worker "Worker" {
            scale {
                min 1
            }
        }
    }
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	sys := program.Architecture.Systems[0]
	webApp := sys.Containers[0]

	if webApp.Scale == nil {
		t.Fatal("Expected WebApp to have a Scale block")
	}

	if *webApp.Scale.Min != 3 {
		t.Errorf("Expected min 3, got %d", *webApp.Scale.Min)
	}
	if *webApp.Scale.Max != 10 {
		t.Errorf("Expected max 10, got %d", *webApp.Scale.Max)
	}
	if *webApp.Scale.Metric != "cpu > 80%" {
		t.Errorf("Expected metric 'cpu > 80%%', got '%s'", *webApp.Scale.Metric)
	}

	worker := sys.Containers[1]
	if worker.Scale == nil {
		t.Fatal("Expected Worker to have a Scale block")
	}
	if *worker.Scale.Min != 1 {
		t.Errorf("Expected min 1, got %d", *worker.Scale.Min)
	}
	if worker.Scale.Max != nil {
		t.Errorf("Expected max to be nil, got %d", *worker.Scale.Max)
	}
}

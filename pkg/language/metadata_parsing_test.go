//go:build legacy

// pkg/language/metadata_parsing_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_MetadataOnContainer(t *testing.T) {
	dsl := `
architecture "Test" {
  system App "Application" {
    container API "API Service" {
      metadata {
        team: "Payments"
        tier: "critical"
        rate_limit: "100/s"
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
		t.Fatalf("Failed to parse metadata: %v", err)
	}

	if len(program.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(program.Architecture.Systems))
	}

	sys := program.Architecture.Systems[0]
	if len(sys.Containers) != 1 {
		t.Fatalf("Expected 1 container, got %d", len(sys.Containers))
	}

	cont := sys.Containers[0]
	if cont.ID != "API" {
		t.Errorf("Expected container ID 'API', got '%s'", cont.ID)
	}

	// Test metadata
	team, ok := cont.MetaString("team")
	if !ok {
		t.Fatal("Expected 'team' metadata key")
	}
	if team != "Payments" {
		t.Errorf("Expected team 'Payments', got '%s'", team)
	}
}

func TestParser_MetadataOnSystem(t *testing.T) {
	dsl := `
architecture "Test" {
  system BillingAPI "Billing API" {
    metadata {
      team: "Payments"
      tier: "critical"
    }
    container API "API" {}
  }
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse system with metadata: %v", err)
	}

	if len(program.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(program.Architecture.Systems))
	}

	sys := program.Architecture.Systems[0]
	if sys.ID != "BillingAPI" {
		t.Errorf("Expected system ID 'BillingAPI', got '%s'", sys.ID)
	}

	// Test metadata helper methods
	if len(sys.Metadata) == 0 {
		t.Fatal("Expected metadata to be parsed")
	}

	team, ok := sys.MetaString("team")
	if !ok {
		t.Fatal("Expected 'team' metadata key")
	}
	if team != "Payments" {
		t.Errorf("Expected team 'Payments', got '%s'", team)
	}

	tier, ok := sys.MetaString("tier")
	if !ok {
		t.Fatal("Expected 'tier' metadata key")
	}
	if tier != "critical" {
		t.Errorf("Expected tier 'critical', got '%s'", tier)
	}

	// Test HasMeta
	if !sys.HasMeta("team") {
		t.Error("HasMeta('team') should return true")
	}
	if sys.HasMeta("nonexistent") {
		t.Error("HasMeta('nonexistent') should return false")
	}

	// Test AllMetadata
	allMeta := sys.AllMetadata()
	if len(allMeta) != 2 {
		t.Errorf("Expected 2 metadata entries, got %d", len(allMeta))
	}
	if allMeta["team"] != "Payments" {
		t.Errorf("Expected allMeta['team']='Payments', got '%s'", allMeta["team"])
	}
}

func TestParser_MetadataMultipleEntries(t *testing.T) {
	dsl := `
architecture "Test" {
  system App "Application" {
    container API "API Service" {
      metadata {
        team: "Payments"
        tier: "critical"
        rate_limit: "100/s"
        cloud: "aws"
        lambda_memory: "512MB"
        tracing: "enabled"
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
		t.Fatalf("Failed to parse multiple metadata entries: %v", err)
	}

	if program.Architecture == nil {
		t.Fatal("Expected architecture to be parsed")
	}

	sys := program.Architecture.Systems[0]
	cont := sys.Containers[0]
	if len(cont.Metadata) != 6 {
		t.Errorf("Expected 6 metadata entries, got %d", len(cont.Metadata))
	}
}

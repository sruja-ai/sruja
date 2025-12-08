//go:build legacy

// pkg/language/metadata_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_MetadataBlocks(t *testing.T) {
	dsl := `
architecture "Test" {
  system App "Application" {
    metadata {
      owner: "team-a"
      tier: "gold"
    }
    container API "API" {
      metadata {
        version: "v1"
      }
    }
  }
}`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if prog.Architecture == nil {
		t.Fatalf("expected architecture to be parsed")
	}
	if len(prog.Architecture.Systems) != 1 {
		t.Fatalf("expected 1 system, got %d", len(prog.Architecture.Systems))
	}

	sys := prog.Architecture.Systems[0]
	if !sys.HasMeta("owner") {
		t.Fatalf("expected system to have 'owner' metadata")
	}
	if !sys.HasMeta("tier") {
		t.Fatalf("expected system to have 'tier' metadata")
	}

	owner, _ := sys.MetaString("owner")
	if owner != "team-a" {
		t.Errorf("expected owner 'team-a', got '%s'", owner)
	}

	if len(sys.Containers) != 1 {
		t.Fatalf("expected 1 container, got %d", len(sys.Containers))
	}

	cont := sys.Containers[0]
	if !cont.HasMeta("version") {
		t.Fatalf("expected container to have 'version' metadata")
	}

	version, _ := cont.MetaString("version")
	if version != "v1" {
		t.Errorf("expected version 'v1', got '%s'", version)
	}
}

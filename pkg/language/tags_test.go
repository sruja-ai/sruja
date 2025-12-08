//go:build legacy

// pkg/language/tags_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_TagsOnSystemAndContainer(t *testing.T) {
	dsl := `
architecture "Test" {
  system App "Application" {
    container API "API" {
      tags ["http", "public"]
    }
  }
}`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if len(prog.Architecture.Systems) < 1 {
		t.Fatalf("no systems")
	}
	sys := prog.Architecture.Systems[0]
	if sys == nil {
		t.Fatalf("expected system")
	}
	if len(sys.Containers) < 1 {
		t.Fatalf("expected container in system")
	}
	cont := sys.Containers[0]
	foundTags := false
	for _, item := range cont.Items {
		if item.Tags != nil && len(item.Tags) >= 2 {
			foundTags = true
			break
		}
	}
	if !foundTags {
		t.Fatalf("expected tags in container items")
	}
}

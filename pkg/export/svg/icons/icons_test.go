package icons

import (
	"os"
	"sync"
	"testing"
)

func TestLoad(t *testing.T) {
	// Reset cache
	cache = nil
	once = sync.Once{}

	// Create nested directory structure to satisfy os.ReadFile path
	// The code looks for "pkg/export/svg/icons/icons.json" relative to CWD
	err := os.MkdirAll("pkg/export/svg/icons", 0o755)
	if err != nil {
		t.Fatalf("Failed to create dir: %v", err)
	}
	defer os.RemoveAll("pkg") // Cleanup

	// Write dummy icons.json
	content := `{
  "test": "<svg>test</svg>"
}`
	err = os.WriteFile("pkg/export/svg/icons/icons.json", []byte(content), 0o644)
	if err != nil {
		t.Fatal(err)
	}

	// Call Get to trigger load
	icon := Get("test")
	if icon != "test" {
		t.Errorf("Expected 'test', got '%s'", icon)
	}

	// Verify cache is populated
	if cache == nil {
		t.Error("Cache should not be nil after Get")
	}
}

package language_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestAllExamples(t *testing.T) {
	examplesDir := "../../examples"
	files, err := os.ReadDir(examplesDir)
	if err != nil {
		t.Fatalf("Failed to read examples directory: %v", err)
	}

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	for _, file := range files {
		if file.IsDir() || !strings.HasSuffix(file.Name(), ".sruja") {
			continue
		}

		// Skip invalid examples intentionally
		if strings.Contains(file.Name(), "invalid") || strings.Contains(file.Name(), "violation") {
			continue
		}

		t.Run(file.Name(), func(t *testing.T) {
			path := filepath.Join(examplesDir, file.Name())
			content, err := os.ReadFile(path)
			if err != nil {
				t.Fatalf("Failed to read file %s: %v", file.Name(), err)
			}

			_, err = parser.Parse(file.Name(), string(content))
			if err != nil {
				t.Errorf("Failed to parse %s: %v", file.Name(), err)
			}
		})
	}
}

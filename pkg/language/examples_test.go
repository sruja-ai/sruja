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

	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	err = filepath.Walk(examplesDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() || !strings.HasSuffix(path, ".sruja") {
			return nil
		}

		// Skip invalid examples intentionally
		if strings.Contains(path, "invalid") || strings.Contains(path, "violation") {
			return nil
		}

		t.Run(path, func(t *testing.T) {
			content, err := os.ReadFile(path)
			if err != nil {
				t.Fatalf("Failed to read file %s: %v", path, err)
			}

			_, diags, err := parser.Parse(path, string(content))
			if err != nil {
				t.Errorf("Failed to parse %s: %v", path, err)
			}
			for _, diag := range diags {
				if diag.Severity == "error" {
					t.Errorf("Parse error in %s: %s", path, diag.Message)
				}
			}
		})
		return nil
	})
	if err != nil {
		t.Fatalf("Failed to walk examples directory: %v", err)
	}
}

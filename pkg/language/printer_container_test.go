// pkg/language/printer_container_test.go
package language_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestPrinter_PrintContainer_WithTags(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Items: []language.ArchitectureItem{
			{
				System: &language.System{
					ID:    "Sys1",
					Label: "System 1",
					Items: []language.SystemItem{
						{
							Container: &language.Container{
								ID:    "Cont1",
								Label: "Container 1",
								Items: []language.ContainerItem{
									{
										Tags: []string{"web", "api"},
									},
								},
							},
						},
					},
				},
			},
		},
	}
	prog := &language.Program{Architecture: arch}
	prog.Architecture.PostProcess()
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `tags [`) {
		t.Errorf("Should print tags. Output: %s", output)
	}
	if !strings.Contains(output, `"web"`) {
		t.Errorf("Should print tag 'web'. Output: %s", output)
	}
	if !strings.Contains(output, `"api"`) {
		t.Errorf("Should print tag 'api'. Output: %s", output)
	}
}

func TestPrinter_PrintContainer_WithVersion(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Items: []language.ArchitectureItem{
			{
				System: &language.System{
					ID:    "Sys1",
					Label: "System 1",
					Items: []language.SystemItem{
						{
							Container: &language.Container{
								ID:    "Cont1",
								Label: "Container 1",
								Items: []language.ContainerItem{
									{
										Version: stringPtr("1.0.0"),
									},
								},
							},
						},
					},
				},
			},
		},
	}
	prog := &language.Program{Architecture: arch}
	prog.Architecture.PostProcess()
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, `version "1.0.0"`) {
		t.Errorf("Should print version. Output: %s", output)
	}
}

func TestPrinter_PrintContainer_WithMetadata(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test",
		Items: []language.ArchitectureItem{
			{
				System: &language.System{
					ID:    "Sys1",
					Label: "System 1",
					Items: []language.SystemItem{
						{
							Container: &language.Container{
								ID:    "Cont1",
								Label: "Container 1",
								Metadata: []*language.MetaEntry{
									{Key: "team", Value: stringPtr("backend")},
								},
							},
						},
					},
				},
			},
		},
	}
	prog := &language.Program{Architecture: arch}
	prog.Architecture.PostProcess()
	printer := language.NewPrinter()
	output := printer.Print(prog)

	if !strings.Contains(output, "metadata {") {
		t.Errorf("Should print metadata block. Output: %s", output)
	}
	if !strings.Contains(output, `team "backend"`) {
		t.Errorf("Should print metadata entry. Output: %s", output)
	}
}

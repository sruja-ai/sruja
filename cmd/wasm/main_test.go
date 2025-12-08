package main

import (
	"testing"

	"github.com/sruja-ai/sruja/internal/converter"
	"github.com/sruja-ai/sruja/pkg/language"
)

func TestConvertToJSON(t *testing.T) {
	arch := &language.Architecture{
		Name: "Test Arch",
		Items: []language.ArchitectureItem{
			{
				System: &language.System{
					ID:    "S1",
					Label: "System 1",
					Items: []language.SystemItem{
						{
							Container: &language.Container{
								ID:    "C1",
								Label: "Container 1",
							},
						},
					},
				},
			},
		},
	}
	// Also populate the typed slices as the converter might use them or Items
	arch.Systems = []*language.System{arch.Items[0].System}

	jsonArch := converter.ConvertToJSON(arch)

	if jsonArch.Metadata.Name != "Sruja Architecture" {
		t.Errorf("Expected metadata name 'Sruja Architecture', got '%s'", jsonArch.Metadata.Name)
	}

	if len(jsonArch.Architecture.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(jsonArch.Architecture.Systems))
	}
	sys := jsonArch.Architecture.Systems[0]
	if sys.ID != "S1" {
		t.Errorf("Expected system ID 'S1', got '%s'", sys.ID)
	}
	if len(sys.Containers) != 1 {
		t.Fatalf("Expected 1 container, got %d", len(sys.Containers))
	}
	if sys.Containers[0].ID != "C1" {
		t.Errorf("Expected container ID 'C1', got '%s'", sys.Containers[0].ID)
	}
}

func TestConvertFromJSON(t *testing.T) {
	jsonArch := converter.ArchitectureJSON{
		Metadata: converter.MetadataJSON{
			Name: "Test Arch",
		},
		Architecture: converter.Architecture{
			Systems: []converter.System{
				{
					ID:    "S1",
					Label: "System 1",
					Containers: []converter.Container{
						{
							ID:    "C1",
							Label: "Container 1",
						},
					},
				},
			},
		},
	}

	arch := converter.ConvertFromJSON(jsonArch)

	if arch.Name != "Test Arch" {
		t.Errorf("Expected name 'Test Arch', got '%s'", arch.Name)
	}

	if len(arch.Systems) != 1 {
		t.Fatalf("Expected 1 system, got %d", len(arch.Systems))
	}
	if arch.Systems[0].ID != "S1" {
		t.Errorf("Expected system ID 'S1', got '%s'", arch.Systems[0].ID)
	}
	// Note: convertFromJSON populates Items and typed slices
	// But system items (containers) are inside the system struct
	if len(arch.Systems[0].Items) != 1 {
		t.Fatalf("Expected 1 item in system, got %d", len(arch.Systems[0].Items))
	}
	if arch.Systems[0].Items[0].Container == nil {
		t.Error("Expected container item in system")
	}
	if arch.Systems[0].Items[0].Container.ID != "C1" {
		t.Errorf("Expected container ID 'C1', got '%s'", arch.Systems[0].Items[0].Container.ID)
	}
}

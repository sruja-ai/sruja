package json

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExporter_Export_TableDriven(t *testing.T) {
	tests := []struct {
		name        string
		prog        *language.Program
		wantEmpty   bool
		expectError bool
	}{
		{
			name:      "Nil program",
			prog:      nil,
			wantEmpty: false, // returns "{}" not empty string
		},
		{
			name: "Empty program",
			prog: &language.Program{},
		},
		{
			name: "Program with nil model",
			prog: &language.Program{Model: nil},
		},
		{
			name: "Program with basic model",
			prog: &language.Program{
				Model: &language.Model{
					Items: []language.ModelItem{
						{
							ElementDef: &language.ElementDef{
								Assignment: &language.ElementAssignment{
									Kind: "system",
									Name: "Sys",
								},
							},
						},
					},
				},
			},
		},
	}

	exporter := NewExporter()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := exporter.Export(tt.prog)
			if (err != nil) != tt.expectError {
				t.Errorf("Export() error = %v, expectError %v", err, tt.expectError)
				return
			}
			if tt.wantEmpty && got != "" {
				t.Errorf("Export() = %v, want empty", got)
			}
			if !tt.wantEmpty && got == "" {
				t.Errorf("Export() = empty, want JSON")
			}
			if tt.prog == nil && got != "{}" {
				t.Errorf("Export(nil) = %v, want {}", got)
			}
		})
	}
}

func TestExporter_ExportCompact_TableDriven(t *testing.T) {
	tests := []struct {
		name        string
		prog        *language.Program
		expectError bool
	}{
		{"Nil program", nil, false},
		{"Empty program", &language.Program{}, false},
	}

	exporter := NewExporter()

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := exporter.ExportCompact(tt.prog)
			if (err != nil) != tt.expectError {
				t.Errorf("ExportCompact() error = %v, expectError %v", err, tt.expectError)
				return
			}
			if len(got) == 0 {
				t.Error("ExportCompact() returned empty bytes")
			}
			if tt.prog == nil && string(got) != "{}" {
				t.Errorf("ExportCompact(nil) = %s, want {}", string(got))
			}
		})
	}
}

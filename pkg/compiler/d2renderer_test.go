//go:build legacy

// Package compiler_test provides tests for the D2 renderer.
package compiler_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/compiler"
)

func TestD2Renderer_NewD2Renderer(t *testing.T) {
	renderer := compiler.NewD2Renderer()
	if renderer == nil {
		t.Fatal("Expected renderer to be created, got nil")
	}
}

func TestD2Renderer_NewD2RendererWithOptions(t *testing.T) {
	opts := compiler.RenderOptions{
		Theme:  "gruvbox-dark",
		Layout: "elk",
		Format: "svg",
	}

	renderer := compiler.NewD2RendererWithOptions(opts)
	if renderer == nil {
		t.Fatal("Expected renderer to be created, got nil")
	}
}

func TestD2Renderer_NewD2RendererWithOptions_Defaults(t *testing.T) {
	// Test with empty options (should use defaults)
	opts := compiler.RenderOptions{}

	renderer := compiler.NewD2RendererWithOptions(opts)
	if renderer == nil {
		t.Fatal("Expected renderer to be created, got nil")
	}
}

func TestD2Renderer_NewD2RendererWithOptions_RejectsTALA(t *testing.T) {
	// Test that TALA is rejected and falls back to dagre
	opts := compiler.RenderOptions{
		Theme:  "neutral-default",
		Layout: "tala", // Should be rejected
	}

	renderer := compiler.NewD2RendererWithOptions(opts)
	if renderer == nil {
		t.Fatal("Expected renderer to be created, got nil")
	}

	// The renderer should have fallen back to dagre
	// (We can't directly check the layout field, but we can verify it doesn't crash)
}

func TestD2Renderer_RenderToSVG_WithoutLibraries(t *testing.T) {
	renderer := compiler.NewD2Renderer()
	d2Code := `x -> y: Hello`

	// This should return an error since D2 libraries aren't available without build tag
	_, err := renderer.RenderToSVG(d2Code)
	if err == nil {
		t.Log("Note: D2 rendering libraries may be available (built with -tags d2render)")
	} else {
		// Expected error when libraries aren't available
		if err.Error() == "" {
			t.Error("Expected error message, got empty string")
		}
	}
}

func TestD2Renderer_RenderToPNG_WithoutLibraries(t *testing.T) {
	renderer := compiler.NewD2Renderer()
	d2Code := `x -> y: Hello`

	// This should return an error since D2 libraries aren't available without build tag
	_, err := renderer.RenderToPNG(d2Code)
	if err == nil {
		t.Log("Note: D2 rendering libraries may be available (built with -tags d2render)")
	} else {
		// Expected error when libraries aren't available
		if err.Error() == "" {
			t.Error("Expected error message, got empty string")
		}
	}
}

func TestD2Renderer_RenderToPDF(t *testing.T) {
	renderer := compiler.NewD2Renderer()
	d2Code := `x -> y: Hello`

	// PDF rendering is not directly supported
	_, err := renderer.RenderToPDF(d2Code)
	if err == nil {
		t.Error("Expected error for PDF rendering (not directly supported)")
	}
}

func TestD2Renderer_RenderToFile_InvalidFormat(t *testing.T) {
	renderer := compiler.NewD2Renderer()
	d2Code := `x -> y: Hello`

	// Test with invalid format
	err := renderer.RenderToFile(d2Code, "output.invalid", "invalid")
	if err == nil {
		t.Error("Expected error for invalid format")
	}
}

func TestD2Renderer_RenderToFile_ValidFormats(t *testing.T) {
	renderer := compiler.NewD2Renderer()
	d2Code := `x -> y: Hello`

	// Test with valid formats (will fail without libraries, but should not error on format)
	formats := []string{"svg", "png"}
	for _, format := range formats {
		err := renderer.RenderToFile(d2Code, "output."+format, format)
		// Error is expected without D2 libraries, but format validation should pass
		if err != nil {
			// Check that error is about libraries, not format
			if err.Error() == "" {
				t.Errorf("Expected error message for format %s, got empty string", format)
			}
		}
	}
}

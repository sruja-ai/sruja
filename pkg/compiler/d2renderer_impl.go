//go:build d2render
// +build d2render

// Package compiler provides compilation from model to various diagram formats.
//
// This file contains the actual D2 rendering implementation.
// It requires the terrastruct/d2 packages to be installed.
// Build with: go build -tags d2render
package compiler

import (
	"fmt"

	"github.com/terrastruct/d2/d2lib"
	"github.com/terrastruct/d2/d2renderers/d2png"
	"github.com/terrastruct/d2/d2renderers/d2svg"
	"github.com/terrastruct/d2/d2themes/d2themescatalog"
)

// hasD2Libraries checks if D2 rendering libraries are available.
func (r *D2Renderer) hasD2Libraries() bool {
	return true
}

// getLayoutEngine converts layout string to d2lib layout constant.
// Only free/open-source layout engines are supported (dagre, elk).
// TALA is not supported as it requires a commercial license.
func (r *D2Renderer) getLayoutEngine() d2lib.LayoutEngine {
	switch r.Layout {
	case "elk":
		return d2lib.LayoutELK
	default:
		return d2lib.LayoutDagre
	}
}

// getThemeID converts theme string to theme ID.
func (r *D2Renderer) getThemeID() int64 {
	switch r.Theme {
	case "gruvbox-dark":
		return d2themescatalog.GruvboxDark.ID
	case "gruvbox-light":
		return d2themescatalog.GruvboxLight.ID
	case "neon":
		return d2themescatalog.Neon.ID
	case "terminal":
		return d2themescatalog.Terminal.ID
	case "violet":
		return d2themescatalog.Violet.ID
	case "warm-neon":
		return d2themescatalog.WarmNeon.ID
	default:
		return d2themescatalog.NeutralDefault.ID
	}
}

// RenderToSVG renders D2 code to SVG bytes.
func (r *D2Renderer) RenderToSVG(d2Code string) ([]byte, error) {
	// Compile D2 code to diagram model
	diagram, _, err := d2lib.Compile("diagram.d2", []byte(d2Code), &d2lib.CompileOptions{
		Layout: r.getLayoutEngine(),
	})
	if err != nil {
		return nil, fmt.Errorf("D2 compilation error: %w", err)
	}

	// Render to SVG
	themeID := r.getThemeID()
	out, err := d2svg.Render(diagram, &d2svg.RenderOpts{
		ThemeID: themeID,
	})
	if err != nil {
		return nil, fmt.Errorf("D2 SVG rendering error: %w", err)
	}

	return out, nil
}

// RenderToPNG renders D2 code to PNG bytes.
func (r *D2Renderer) RenderToPNG(d2Code string) ([]byte, error) {
	// Compile D2 code to diagram model
	diagram, _, err := d2lib.Compile("diagram.d2", []byte(d2Code), &d2lib.CompileOptions{
		Layout: r.getLayoutEngine(),
	})
	if err != nil {
		return nil, fmt.Errorf("D2 compilation error: %w", err)
	}

	// Render to PNG
	themeID := r.getThemeID()
	pngBytes, err := d2png.Render(diagram, &d2png.RenderOpts{
		ThemeID: themeID,
	})
	if err != nil {
		return nil, fmt.Errorf("D2 PNG rendering error: %w", err)
	}

	return pngBytes, nil
}

// RenderToPDF renders D2 code to PDF bytes.
func (r *D2Renderer) RenderToPDF(d2Code string) ([]byte, error) {
	// D2 doesn't have direct PDF support, but we can render to SVG and convert
	// For now, return an error suggesting SVG conversion
	return nil, fmt.Errorf("PDF rendering not directly supported. Render to SVG and convert using external tool")
}


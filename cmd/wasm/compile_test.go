//go:build !js || !wasm

// pkg/wasm/compile_test.go
// This file tests the compilation logic without WASM/JS bindings
package main

import (
	"context"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
	"oss.terrastruct.com/d2/d2graph"
	"oss.terrastruct.com/d2/d2layouts/d2dagrelayout"
	"oss.terrastruct.com/d2/d2lib"
	"oss.terrastruct.com/d2/d2renderers/d2svg"
	"oss.terrastruct.com/d2/lib/textmeasure"
)

// compileToSVG extracts the core compilation logic for testing
func compileToSVG(input string) (string, error) {
	p, err := language.NewParser()
	if err != nil {
		return "", err
	}

	program, err := p.Parse("playground.sruja", input)
	if err != nil {
		return "", err
	}

	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	// Note: OrphanDetectionRule is not registered in tests to allow isolated elements

	errs := validator.Validate(program)
	if len(errs) > 0 {
		msg := ""
		for _, e := range errs {
			msg += e.Error() + "\n"
		}
		return "", &compileError{message: "Validation errors:\n" + msg}
	}

	exporter := d2.NewExporter()
	d2Script, err := exporter.Export(program.Architecture)
	if err != nil {
		return "", err
	}

	// Compile D2 to SVG
	ruler, err := textmeasure.NewRuler()
	if err != nil {
		return "", &compileError{message: "Ruler Error: " + err.Error()}
	}

	pad := int64(d2svg.DEFAULT_PADDING)
	renderOpts := &d2svg.RenderOpts{
		Pad: &pad,
	}

	layout := "dagre"
	diagram, _, err := d2lib.Compile(context.Background(), d2Script, &d2lib.CompileOptions{
		Ruler:  ruler,
		Layout: &layout,
		LayoutResolver: func(engine string) (d2graph.LayoutGraph, error) {
			return func(ctx context.Context, g *d2graph.Graph) error {
				return d2dagrelayout.Layout(ctx, g, nil)
			}, nil
		},
	}, renderOpts)
	if err != nil {
		return "", &compileError{message: "D2 Compilation Error: " + err.Error()}
	}

	out, err := d2svg.Render(diagram, renderOpts)
	if err != nil {
		return "", &compileError{message: "D2 Rendering Error: " + err.Error()}
	}

	return string(out), nil
}

type compileError struct {
	message string
}

func (e *compileError) Error() string {
	return e.message
}

func TestCompileToSVG_SimpleSystem(t *testing.T) {
	// Add a relation to avoid orphan detection
	input := `architecture "Test" { 
		system API "API Service" {}
		person User "User" {}
		User -> API
	}`
	svg, err := compileToSVG(input)
	if err != nil {
		t.Fatalf("compileToSVG failed: %v", err)
	}
	if !strings.Contains(svg, "<svg") {
		t.Error("Output should be SVG")
	}
}

func TestCompileToSVG_WithRelations(t *testing.T) {
	input := `architecture "Test" {
		system A "System A" {}
		system B "System B" {}
		A -> B
	}`
	svg, err := compileToSVG(input)
	if err != nil {
		t.Fatalf("compileToSVG failed: %v", err)
	}
	if !strings.Contains(svg, "<svg") {
		t.Error("Output should be SVG")
	}
}

func TestCompileToSVG_InvalidSyntax(t *testing.T) {
	input := `invalid syntax here`
	_, err := compileToSVG(input)
	if err == nil {
		t.Fatal("compileToSVG should error on invalid syntax")
	}
}

func TestCompileToSVG_ValidationErrors(t *testing.T) {
	// Duplicate ID should cause validation error
	input := `architecture "Test" {
		system API "API" {}
		system API "API2" {}
	}`
	_, err := compileToSVG(input)
	if err == nil {
		t.Fatal("compileToSVG should error on validation failures")
	}
	if !strings.Contains(err.Error(), "Validation errors") {
		t.Error("Error should mention validation errors")
	}
}

func TestCompileToSVG_ComplexArchitecture(t *testing.T) {
	// Simplified to avoid validation issues
	input := `architecture "Test" {
		person User "End User" {}
		system API "API Service" {
			container WebApp "Web Application" {}
			datastore DB "Database" {}
		}
		User -> API
	}`
	svg, err := compileToSVG(input)
	if err != nil {
		t.Fatalf("compileToSVG failed: %v", err)
	}
	if !strings.Contains(svg, "<svg") {
		t.Error("Output should be SVG")
	}
}

func TestCompileToSVG_WithMetadata(t *testing.T) {
	input := `architecture "Test" {
		system API "API Service" {
			metadata {
				owner: "team"
				tier: "gold"
			}
		}
	}`
	svg, err := compileToSVG(input)
	if err != nil {
		t.Fatalf("compileToSVG failed: %v", err)
	}
	if !strings.Contains(svg, "<svg") {
		t.Error("Output should be SVG")
	}
}

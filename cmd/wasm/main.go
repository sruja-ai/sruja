//go:build js && wasm

package main

import (
	"context"
	"fmt"
	"log/slog"

	"syscall/js"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
	"oss.terrastruct.com/d2/d2graph"
	"oss.terrastruct.com/d2/d2layouts/d2dagrelayout"
	"oss.terrastruct.com/d2/d2lib"
	"oss.terrastruct.com/d2/d2renderers/d2svg"
	d2log "oss.terrastruct.com/d2/lib/log"
	"oss.terrastruct.com/d2/lib/textmeasure"
)

func main() {
	c := make(chan struct{})
	js.Global().Set("compileSruja", js.FuncOf(compileSruja))
	<-c
}

func compileSruja(this js.Value, args []js.Value) (res interface{}) {
	defer func() {
		if r := recover(); r != nil {
			res = result(nil, fmt.Errorf("panic: %v", r))
		}
	}()

	return doCompile(args)
}

func doCompile(args []js.Value) interface{} {
	if len(args) < 1 {
		return result(nil, fmt.Errorf("invalid arguments"))
	}
	input := args[0].String()
	filename := "playground.sruja"
	if len(args) >= 2 {
		fn := args[1].String()
		if fn != "" {
			filename = fn
		}
	}

	p, err := language.NewParser()
	if err != nil {
		return result(nil, err)
	}

	program, err := p.Parse(filename, input)
	if err != nil {
		return result(nil, fmt.Errorf("%s: %w", filename, err))
	}

	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	validator.RegisterRule(&engine.OrphanDetectionRule{})

	errs := validator.Validate(program)
	if len(errs) > 0 {
		msg := ""
		for _, e := range errs {
			msg += filename + ": " + e.Error() + "\n"
		}
		return result(nil, fmt.Errorf("Validation errors:\n%s", msg))
	}

	exporter := d2.NewExporter()
	d2Script, err := exporter.Export(program.Architecture)
	if err != nil {
		return result(nil, err)
	}

	// Compile D2 to SVG
	ruler, err := textmeasure.NewRuler()
	if err != nil {
		return result(nil, fmt.Errorf("Ruler Error: %w", err))
	}

	pad := int64(d2svg.DEFAULT_PADDING)
	renderOpts := &d2svg.RenderOpts{
		Pad: &pad,
	}

	// Create a no-op logger for WASM to suppress d2 library warnings
	// In WASM, we don't need logging output (it would go to browser console anyway)
	// Using nil writer creates a no-op handler that discards all logs
	logger := slog.New(slog.NewTextHandler(nil, &slog.HandlerOptions{Level: slog.LevelError}))
	ctx := d2log.With(context.Background(), logger)

	layout := "dagre"
	diagram, _, err := d2lib.Compile(ctx, d2Script, &d2lib.CompileOptions{
		Ruler:  ruler,
		Layout: &layout,
		LayoutResolver: func(engine string) (d2graph.LayoutGraph, error) {
			return func(ctx context.Context, g *d2graph.Graph) error {
				return d2dagrelayout.Layout(ctx, g, nil)
			}, nil
		},
	}, renderOpts)
	if err != nil {
		return result(nil, fmt.Errorf("%s: D2 Compilation Error: %w", filename, err))
	}

	out, err := d2svg.Render(diagram, renderOpts)
	if err != nil {
		return result(nil, fmt.Errorf("%s: D2 Rendering Error: %w", filename, err))
	}

	svg := string(out)
	return result(&svg, nil)
}

func result(svg *string, err error) interface{} {
	res := make(map[string]interface{})
	if svg != nil {
		res["svg"] = *svg
	}
	if err != nil {
		res["error"] = err.Error()
	}
	return js.ValueOf(res)
}

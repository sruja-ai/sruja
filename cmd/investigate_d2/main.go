package main

import (
	"context"
	"fmt"
	"os"

	"oss.terrastruct.com/d2/d2graph"
	"oss.terrastruct.com/d2/d2layouts/d2dagrelayout"
	"oss.terrastruct.com/d2/d2lib"
	"oss.terrastruct.com/d2/d2renderers/d2svg"
	"oss.terrastruct.com/d2/d2themes/d2themescatalog"
	"oss.terrastruct.com/d2/lib/textmeasure"
)

func main() {
	// Simple D2 script
	script := `
my_system: {
  tooltip: "ID:System1"
  my_container: {
    tooltip: "ID:Container1"
  }
}
user -> my_system
`

	// Compile
	ruler, err := textmeasure.NewRuler()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Ruler error: %v\n", err)
		os.Exit(1)
	}

	renderOpts := &d2svg.RenderOpts{
		ThemeID: &d2themescatalog.GrapeSoda.ID,
	}

	diagram, _, err := d2lib.Compile(context.Background(), script, &d2lib.CompileOptions{
		Ruler: ruler,
		LayoutResolver: func(engine string) (d2graph.LayoutGraph, error) {
			return d2dagrelayout.DefaultLayout, nil
		},
	}, renderOpts)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Compile error: %v\n", err)
		os.Exit(1)
	}

	// Render to SVG
	out, err := d2svg.Render(diagram, renderOpts)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Render error: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(string(out))
}

package main

import (
	"fmt"
	"io"

	"github.com/sruja-ai/sruja/pkg/dx"
)

func printDiffList(w io.Writer, title string, items []string, prefix, attr string) {
	if len(items) == 0 {
		return
	}
	useColor := dx.SupportsColor()
	_, _ = fmt.Fprintln(w, dx.Section(title))
	for _, item := range items {
		_, _ = fmt.Fprintf(w, "  %s %s\n", dx.Colorize(attr, prefix, useColor), dx.Colorize(dx.ColorBold, item, useColor))
	}
	_, _ = fmt.Fprintln(w)
}

func printContainerDiffs(w io.Writer, sys string, containers map[string][]string, prefix, attr string) {
	if items, ok := containers[sys]; ok && len(items) > 0 {
		useColor := dx.SupportsColor()
		for _, cont := range items {
			_, _ = fmt.Fprintf(w, "    %s container %s\n", dx.Colorize(attr, prefix, useColor), cont)
		}
	}
}

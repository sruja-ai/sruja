package main

import (
	"flag"
	"fmt"
	"io"
)

var (
	version = "dev"
	commit  = "none"
	date    = "unknown"
)

func runVersion(args []string, stdout, stderr io.Writer) int {
	versionCmd := flag.NewFlagSet("version", flag.ContinueOnError)
	versionCmd.SetOutput(stderr)

	if err := versionCmd.Parse(args); err != nil {
		_, _ = fmt.Fprintf(stderr, "Error parsing version flags: %v\n", err)
		return 1
	}

	fmt.Fprintf(stdout, "sruja version %s (commit: %s, date: %s)\n", version, commit, date)
	return 0
}

// apps/cli/cmd/version.go
package main

import (
	"fmt"
	"io"
)

const Version = "0.1.0"
const BuildDate = "unknown"
const GitCommit = "unknown"

func runVersion(stdout io.Writer) int {
	_, _ = fmt.Fprintf(stdout, "sruja version %s\n", Version)
	if BuildDate != "unknown" {
		_, _ = fmt.Fprintf(stdout, "Build date: %s\n", BuildDate)
	}
	if GitCommit != "unknown" {
		_, _ = fmt.Fprintf(stdout, "Git commit: %s\n", GitCommit)
	}
	return 0
}

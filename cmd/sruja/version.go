// apps/cli/cmd/version.go
package main

import (
	"fmt"
	"os"
)

const Version = "0.1.0"
const BuildDate = "unknown"
const GitCommit = "unknown"

func runVersion() {
	fmt.Printf("sruja version %s\n", Version)
	if BuildDate != "unknown" {
		fmt.Printf("Build date: %s\n", BuildDate)
	}
	if GitCommit != "unknown" {
		fmt.Printf("Git commit: %s\n", GitCommit)
	}
	os.Exit(0)
}

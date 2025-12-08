package main

import (
	"bytes"
	"strings"
	"testing"
)

func TestRunVersion(t *testing.T) {
	var stdout, stderr bytes.Buffer

	// Test version output
	exitCode := runVersion([]string{}, &stdout, &stderr)
	if exitCode != 0 {
		t.Errorf("Expected exit code 0, got %d. Stderr: %s", exitCode, stderr.String())
	}
	output := stdout.String()
	if !strings.Contains(output, "sruja version") {
		t.Error("Output missing version string")
	}

	// Test with flags (should fail as it doesn't accept flags but flags are parsed)
	// The implementation uses flag.FlagSet, so unknown flags will cause error.
	stdout.Reset()
	stderr.Reset()
	exitCode = runVersion([]string{"--unknown"}, &stdout, &stderr)
	if exitCode == 0 {
		t.Error("Expected non-zero exit code for unknown flag")
	}
	if !strings.Contains(stderr.String(), "flag provided but not defined") {
		t.Errorf("Expected flag error, got: %s", stderr.String())
	}
}

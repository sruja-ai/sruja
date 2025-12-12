package main

import (
    "bytes"
    "strings"
    "testing"
)

func TestRunVersion_PrintsVersion(t *testing.T) {
    var out, err bytes.Buffer
    code := runVersion([]string{}, &out, &err)
    if code != 0 {
        t.Fatalf("expected exit code 0, got %d", code)
    }
    line := out.String()
    if !strings.HasPrefix(line, "sruja version ") {
        t.Fatalf("unexpected version output: %q", line)
    }
}

func TestRunVersion_InvalidFlag(t *testing.T) {
    var out, err bytes.Buffer
    code := runVersion([]string{"--unknown"}, &out, &err)
    if code != 1 {
        t.Fatalf("expected exit code 1 on invalid flag, got %d", code)
    }
    if !strings.Contains(err.String(), "Error parsing version flags") {
        t.Fatalf("expected error message, got %q", err.String())
    }
}

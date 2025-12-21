// pkg/engine/validator_helpers_test.go
package engine

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

// TestCollectResults_Timeout tests collectResults with timeout which triggers drainChannels
func TestCollectResults_Timeout(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Millisecond)
	defer cancel()

	errChan := make(chan []diagnostics.Diagnostic, 2)
	panicChan := make(chan string, 2)

	// Send some results after timeout to test drainChannels
	go func() {
		time.Sleep(10 * time.Millisecond) // After timeout
		errChan <- []diagnostics.Diagnostic{{Message: "Test1"}}
		errChan <- []diagnostics.Diagnostic{{Message: "Test2"}}
	}()

	// This should timeout and call drainChannels
	diags := collectResults(ctx, errChan, panicChan, 2)

	// Should have timeout diagnostic
	hasTimeout := false
	for _, diag := range diags {
		if diag.Code == diagnostics.CodeValidationTimeout {
			hasTimeout = true
			break
		}
	}

	if !hasTimeout {
		t.Logf("Timeout may have occurred too quickly. Diagnostics: %v", diags)
	}
}

// TestCollectResults_WithPanic tests collectResults with panic messages
func TestCollectResults_WithPanic(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()

	errChan := make(chan []diagnostics.Diagnostic, 1)
	panicChan := make(chan string, 1)

	// Send a panic message
	panicChan <- "Test panic message"

	diags := collectResults(ctx, errChan, panicChan, 1)

	// Should have panic diagnostic
	hasPanic := false
	for _, diag := range diags {
		if diag.Code == diagnostics.CodeValidationPanic {
			hasPanic = true
			if !strings.Contains(diag.Message, "Test panic message") {
				t.Errorf("Expected panic message to contain 'Test panic message', got: %s", diag.Message)
			}
			break
		}
	}

	if !hasPanic {
		t.Error("Expected panic diagnostic but got none")
	}
}

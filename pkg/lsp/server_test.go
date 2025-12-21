// pkg/lsp/server_test.go
package lsp

import (
	"bytes"
	"encoding/json"
	"fmt"
	"testing"
	"time"

	"github.com/sourcegraph/go-lsp"
)

// TestStartServer_Initialize tests basic server initialization
func TestStartServer_Initialize(t *testing.T) {
	// Create mock input/output
	input := bytes.NewBuffer(nil)
	output := bytes.NewBuffer(nil)

	// Craft an LSP initialize request
	initRequest := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      1,
		"method":  "initialize",
		"params": map[string]interface{}{
			"processId": 12345,
			"rootUri":   "file:///test",
			"capabilities": map[string]interface{}{
				"textDocument": map[string]interface{}{
					"hover": map[string]interface{}{
						"contentFormat": []string{"markdown"},
					},
				},
			},
		},
	}

	// Encode as JSON-RPC message
	reqBytes, _ := json.Marshal(initRequest)
	// JSON-RPC over stdio uses Content-Length header
	msg := fmt.Sprintf("Content-Length: %d\r\n\r\n%s", len(reqBytes), reqBytes)
	input.WriteString(msg)

	// Start server in goroutine (it blocks until disconnected)
	done := make(chan error, 1)
	go func() {
		done <- StartServer(input, output)
	}()

	// Give server time to process
	time.Sleep(100 * time.Millisecond)

	// Check that we got a response (output buffer should have data)
	if output.Len() == 0 {
		t.Fatal("expected server to write response, got nothing")
	}

	// Server should still be running, so we can't check done yet
	// In a real test, we'd send a shutdown request
	t.Log("Server started and responded to initialize request")
}

// TestNewServer tests server creation
func TestNewServer(t *testing.T) {
	srv := NewServer()
	if srv == nil {
		t.Fatal("expected non-nil server")
	}
	if srv.workspace == nil {
		t.Fatal("expected server to have workspace")
	}
}

// TestServer_Initialize tests the Initialize method directly
func TestServer_Initialize(t *testing.T) {
	srv := NewServer()
	processID := 12345
	rootURI := lsp.DocumentURI("file:///test")
	params := lsp.InitializeParams{
		ProcessID: processID,
		RootURI:   rootURI,
	}

	result, err := srv.Initialize(nil, params)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}

	// Check capabilities - result is already *InitializeResult
	if result.Capabilities.TextDocumentSync == nil {
		t.Fatal("expected TextDocumentSync capability")
	}
}

func intPtr(i int) *int {
	return &i
}

func strPtr(s string) *string {
	return &s
}

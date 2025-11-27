// pkg/kernel/jupyter/server_test.go
package jupyter

import (
	"encoding/json"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/kernel"
)

func TestNewServer(t *testing.T) {
	k, err := kernel.NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	sessionID := "test-session-123"
	server := NewServer(k, sessionID)

	if server.session != sessionID {
		t.Errorf("Expected session %q, got %q", sessionID, server.session)
	}

	if server.kernel == nil {
		t.Error("Expected non-nil kernel")
	}

	if server.execCount != 0 {
		t.Errorf("Expected exec_count 0, got %d", server.execCount)
	}
}

func TestServerDetectCellType(t *testing.T) {
	k, err := kernel.NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	server := NewServer(k, "test-session")

	tests := []struct {
		name     string
		code     string
		expected kernel.CellType
	}{
		{
			name:     "magic command",
			code:     "%ir",
			expected: kernel.CellTypeDSL,
		},
		{
			name:     "snapshot command",
			code:     "%snapshot test",
			expected: kernel.CellTypeDSL,
		},
		{
			name:     "diagram command",
			code:     "diagram system Billing",
			expected: kernel.CellTypeDiagram,
		},
		{
			name:     "validation command",
			code:     "validate all",
			expected: kernel.CellTypeValidation,
		},
		{
			name:     "query command - select",
			code:     "select systems where name == 'Billing'",
			expected: kernel.CellTypeQuery,
		},
		{
			name:     "query command - find",
			code:     "find components",
			expected: kernel.CellTypeQuery,
		},
		{
			name:     "DSL code",
			code:     "system Billing {}",
			expected: kernel.CellTypeDSL,
		},
		{
			name:     "DSL code with whitespace",
			code:     "  system Billing {}  ",
			expected: kernel.CellTypeDSL,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cellType := server.detectCellType(tt.code)
			if cellType != tt.expected {
				t.Errorf("Expected cell type %q, got %q", tt.expected, cellType)
			}
		})
	}
}

func TestServerGetCompletions(t *testing.T) {
	k, err := kernel.NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	server := NewServer(k, "test-session")

	tests := []struct {
		name      string
		code      string
		cursorPos int
		wantAny   []string // At least one of these should be in matches
	}{
		{
			name:      "empty code",
			code:      "",
			cursorPos: 0,
			wantAny:   []string{"system", "architecture"},
		},
		{
			name:      "partial keyword",
			code:      "sys",
			cursorPos: 3,
			wantAny:   []string{"system"},
		},
		{
			name:      "partial keyword - diag",
			code:      "diag",
			cursorPos: 4,
			wantAny:   []string{"diagram"},
		},
		{
			name:      "full word",
			code:      "system ",
			cursorPos: 7,
			wantAny:   []string{}, // Should return keywords
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			matches := server.getCompletions(tt.code, tt.cursorPos)

			if len(tt.wantAny) > 0 {
				found := false
				for _, want := range tt.wantAny {
					for _, match := range matches {
						if match == want {
							found = true
							break
						}
					}
					if found {
						break
					}
				}
				if !found {
					t.Errorf("Expected at least one of %v in matches %v", tt.wantAny, matches)
				}
			}
		})
	}
}

func TestServerHandleKernelInfo(t *testing.T) {
	k, err := kernel.NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	server := NewServer(k, "test-session")

	header := NewMessageHeader("kernel_info_request", "test-session")
	msg := &JupyterMessage{
		Header:   header,
		Content:  []byte("{}"),
		Metadata: make(map[string]interface{}),
	}

	// We can't easily test the full flow without mocking stdout,
	// but we can test that it doesn't panic
	err = server.handleKernelInfo(msg)
	if err != nil {
		t.Errorf("handleKernelInfo returned error: %v", err)
	}
}

func TestServerHandleIsComplete(t *testing.T) {
	k, err := kernel.NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	server := NewServer(k, "test-session")

	tests := []struct {
		name     string
		code     string
		expected string // "complete" or "incomplete"
	}{
		{
			name:     "balanced braces",
			code:     "system Test {}",
			expected: "complete",
		},
		{
			name:     "unbalanced braces - missing close",
			code:     "system Test {",
			expected: "incomplete",
		},
		{
			name:     "unbalanced braces - missing open",
			code:     "system Test }",
			expected: "complete", // Current logic only checks open > close
		},
		{
			name:     "multiple balanced braces",
			code:     "system Test { container API {} }",
			expected: "complete",
		},
		{
			name:     "empty code",
			code:     "",
			expected: "complete",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := IsCompleteRequest{Code: tt.code}
			reqJSON, err := json.Marshal(req)
			if err != nil {
				t.Fatalf("Failed to marshal request: %v", err)
			}

			header := NewMessageHeader("is_complete_request", "test-session")
			msg := &JupyterMessage{
				Header:   header,
				Content:  reqJSON,
				Metadata: make(map[string]interface{}),
			}

			handleErr := server.handleIsComplete(msg)
			if handleErr != nil {
				t.Errorf("handleIsComplete returned error: %v", handleErr)
			}
			// Note: We can't easily verify the reply without capturing stdout
			// This test mainly ensures it doesn't panic
		})
	}
}

func TestServerHandleComplete(t *testing.T) {
	k, err := kernel.NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	server := NewServer(k, "test-session")

	req := CompleteRequest{
		Code:      "sys",
		CursorPos: 3,
	}
	reqJSON, err := json.Marshal(req)
	if err != nil {
		t.Fatalf("Failed to marshal request: %v", err)
	}

	header := NewMessageHeader("complete_request", "test-session")
	msg := &JupyterMessage{
		Header:   header,
		Content:  reqJSON,
		Metadata: make(map[string]interface{}),
	}

	handleErr := server.handleComplete(msg)
	if handleErr != nil {
		t.Errorf("handleComplete returned error: %v", handleErr)
	}
}

func TestServerHandleInspect(t *testing.T) {
	k, err := kernel.NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	server := NewServer(k, "test-session")

	req := InspectRequest{
		Code:      "system Billing",
		CursorPos: 7,
	}
	reqJSON, err := json.Marshal(req)
	if err != nil {
		t.Fatalf("Failed to marshal request: %v", err)
	}

	header := NewMessageHeader("inspect_request", "test-session")
	msg := &JupyterMessage{
		Header:   header,
		Content:  reqJSON,
		Metadata: make(map[string]interface{}),
	}

	handleErr := server.handleInspect(msg)
	if handleErr != nil {
		t.Errorf("handleInspect returned error: %v", handleErr)
	}
}

func TestServerHandleMessage(t *testing.T) {
	k, err := kernel.NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	server := NewServer(k, "test-session")

	tests := []struct {
		name    string
		msgType string
		content string
		wantErr bool
	}{
		{
			name:    "kernel_info_request",
			msgType: "kernel_info_request",
			content: "{}",
			wantErr: false,
		},
		{
			name:    "unsupported message type",
			msgType: "unsupported_type",
			content: "{}",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			header := NewMessageHeader(tt.msgType, "test-session")
			msg := &JupyterMessage{
				Header:   header,
				Content:  []byte(tt.content),
				Metadata: make(map[string]interface{}),
			}

			err := server.handleMessage(msg)
			if (err != nil) != tt.wantErr {
				t.Errorf("handleMessage() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestCreateDisplayDataFromOutputs(t *testing.T) {
	data := map[string]interface{}{
		"text/plain":       "Test output",
		"application/json": `{"result": "ok"}`,
	}

	displayData := CreateDisplayData(data, nil)

	if displayData.Data["text/plain"] != "Test output" {
		t.Error("Text data not preserved")
	}

	jsonData, ok := displayData.Data["application/json"].(string)
	if !ok {
		t.Error("JSON data not preserved as string")
	}

	if !strings.Contains(jsonData, "ok") {
		t.Error("JSON data content not preserved")
	}
}

func TestSetConnectionFile(t *testing.T) {
	k, err := kernel.NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	server := NewServer(k, "test-session")

	connFile := "/tmp/connection.json"
	server.SetConnectionFile(connFile)

	if server.connFile != connFile {
		t.Errorf("Expected connFile %q, got %q", connFile, server.connFile)
	}
}

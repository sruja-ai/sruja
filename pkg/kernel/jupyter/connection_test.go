// pkg/kernel/jupyter/connection_test.go
// Tests for connection file parsing

package jupyter

import (
	"os"
	"testing"
)

func TestParseConnectionFile(t *testing.T) {
	// Create a temporary connection file
	tmpFile, err := os.CreateTemp("", "connection-*.json")
	if err != nil {
		t.Fatalf("Failed to create temp file: %v", err)
	}
	defer os.Remove(tmpFile.Name())

	// Write test connection file
	connJSON := `{
		"transport": "tcp",
		"ip": "127.0.0.1",
		"shell_port": 49152,
		"iopub_port": 49153,
		"stdin_port": 49154,
		"control_port": 49155,
		"hb_port": 49156,
		"signature_scheme": "hmac-sha256",
		"key": "a0436f6c-1916-498b-9ebd-6ca7a0d4c7b0"
	}`

	if _, err := tmpFile.WriteString(connJSON); err != nil {
		t.Fatalf("Failed to write connection file: %v", err)
	}
	tmpFile.Close()

	// Parse connection file
	connInfo, err := ParseConnectionFile(tmpFile.Name())
	if err != nil {
		t.Fatalf("Failed to parse connection file: %v", err)
	}

	// Verify fields
	if connInfo.Transport != "tcp" {
		t.Errorf("Expected transport 'tcp', got '%s'", connInfo.Transport)
	}
	if connInfo.IP != "127.0.0.1" {
		t.Errorf("Expected IP '127.0.0.1', got '%s'", connInfo.IP)
	}
	if connInfo.ShellPort != 49152 {
		t.Errorf("Expected shell port 49152, got %d", connInfo.ShellPort)
	}
	if connInfo.Key == "" {
		t.Error("Expected key to be set")
	}
}

func TestGetAddresses(t *testing.T) {
	connInfo := &ConnectionInfo{
		Transport:   "tcp",
		IP:          "127.0.0.1",
		ShellPort:   49152,
		IOPubPort:   49153,
		StdinPort:   49154,
		ControlPort: 49155,
		HBPort:      49156,
	}

	if addr := connInfo.GetShellAddress(); addr != "tcp://127.0.0.1:49152" {
		t.Errorf("Expected shell address 'tcp://127.0.0.1:49152', got '%s'", addr)
	}

	if addr := connInfo.GetIOPubAddress(); addr != "tcp://127.0.0.1:49153" {
		t.Errorf("Expected iopub address 'tcp://127.0.0.1:49153', got '%s'", addr)
	}
}

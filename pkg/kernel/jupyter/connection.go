// pkg/kernel/jupyter/connection.go
// Connection file parsing for Jupyter kernels

package jupyter

import (
	"encoding/json"
	"fmt"
	"os"
)

// ConnectionInfo represents the connection information from a Jupyter connection file.
//
// The connection file is a JSON file that contains:
//   - Transport protocol (tcp, ipc)
//   - IP address and ports for each channel
//   - Signing key for message authentication
//   - HMAC key for message signing
//
// Example connection file:
//
//	{
//	  "transport": "tcp",
//	  "ip": "127.0.0.1",
//	  "shell_port": 49152,
//	  "iopub_port": 49153,
//	  "stdin_port": 49154,
//	  "control_port": 49155,
//	  "hb_port": 49156,
//	  "signature_scheme": "hmac-sha256",
//	  "key": "a0436f6c-1916-498b-9ebd-6ca7a0d4c7b0"
//	}
type ConnectionInfo struct {
	Transport       string `json:"transport"`        // "tcp" or "ipc"
	IP              string `json:"ip"`               // IP address (for tcp)
	ShellPort       int    `json:"shell_port"`       // Port for shell channel
	IOPubPort       int    `json:"iopub_port"`       // Port for iopub channel
	StdinPort       int    `json:"stdin_port"`       // Port for stdin channel
	ControlPort     int    `json:"control_port"`     // Port for control channel
	HBPort          int    `json:"hb_port"`          // Port for heartbeat channel
	SignatureScheme string `json:"signature_scheme"` // "hmac-sha256"
	Key             string `json:"key"`              // HMAC key for message signing
}

// ParseConnectionFile parses a Jupyter connection file.
func ParseConnectionFile(path string) (*ConnectionInfo, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read connection file: %w", err)
	}

	var connInfo ConnectionInfo
	if err := json.Unmarshal(data, &connInfo); err != nil {
		return nil, fmt.Errorf("failed to parse connection file: %w", err)
	}

	// Validate required fields
	if connInfo.Transport == "" {
		connInfo.Transport = "tcp" // Default to tcp
	}
	if connInfo.IP == "" {
		connInfo.IP = "127.0.0.1" // Default to localhost
	}
	if connInfo.SignatureScheme == "" {
		connInfo.SignatureScheme = "hmac-sha256" // Default
	}

	return &connInfo, nil
}

// GetShellAddress returns the ZeroMQ address for the shell channel.
func (c *ConnectionInfo) GetShellAddress() string {
	return fmt.Sprintf("%s://%s:%d", c.Transport, c.IP, c.ShellPort)
}

// GetIOPubAddress returns the ZeroMQ address for the iopub channel.
func (c *ConnectionInfo) GetIOPubAddress() string {
	return fmt.Sprintf("%s://%s:%d", c.Transport, c.IP, c.IOPubPort)
}

// GetStdinAddress returns the ZeroMQ address for the stdin channel.
func (c *ConnectionInfo) GetStdinAddress() string {
	return fmt.Sprintf("%s://%s:%d", c.Transport, c.IP, c.StdinPort)
}

// GetControlAddress returns the ZeroMQ address for the control channel.
func (c *ConnectionInfo) GetControlAddress() string {
	return fmt.Sprintf("%s://%s:%d", c.Transport, c.IP, c.ControlPort)
}

// GetHBAddress returns the ZeroMQ address for the heartbeat channel.
func (c *ConnectionInfo) GetHBAddress() string {
	return fmt.Sprintf("%s://%s:%d", c.Transport, c.IP, c.HBPort)
}

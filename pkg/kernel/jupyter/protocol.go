// pkg/kernel/jupyter/protocol.go
// Jupyter Kernel Messaging Protocol implementation for Sruja Kernel

package jupyter

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"time"
)

// JupyterMessage represents a Jupyter protocol message.
// Reference: https://jupyter-client.readthedocs.io/en/stable/messaging.html
type JupyterMessage struct {
	Header       MessageHeader             `json:"header"`
	ParentHeader *MessageHeader            `json:"parent_header,omitempty"`
	Metadata     map[string]interface{}    `json:"metadata,omitempty"`
	Content      json.RawMessage           `json:"content,omitempty"`
	Buffers      []interface{}             `json:"buffers,omitempty"` // For binary data
}

// MessageHeader contains message routing information.
type MessageHeader struct {
	MsgID   string                 `json:"msg_id"`
	Username string                `json:"username"`
	Session  string                `json:"session"`
	MsgType  string                `json:"msg_type"`
	Version  string                `json:"version"` // Protocol version, typically "5.3"
	Date     time.Time             `json:"date"`
}

// ExecuteRequest represents an execute_request message.
type ExecuteRequest struct {
	Code         string                 `json:"code"`
	Silent       bool                   `json:"silent"`
	StoreHistory bool                   `json:"store_history"`
	UserExpressions map[string]string   `json:"user_expressions,omitempty"`
	AllowStdin   bool                   `json:"allow_stdin,omitempty"`
	StopOnError  bool                   `json:"stop_on_error,omitempty"`
}

// ExecuteReply represents an execute_reply message.
type ExecuteReply struct {
	Status         string                 `json:"status"` // "ok", "error", "abort"
	ExecutionCount *int                   `json:"execution_count,omitempty"`
	Payload        []map[string]interface{} `json:"payload,omitempty"`
	UserExpressions map[string]interface{}  `json:"user_expressions,omitempty"`
}

// ExecuteResult represents execution results (status="ok").
type ExecuteResult struct {
	ExecutionCount int                      `json:"execution_count"`
	Data           map[string]interface{}   `json:"data"`
	Metadata       map[string]interface{}   `json:"metadata,omitempty"`
}

// ExecuteError represents execution errors (status="error").
type ExecuteError struct {
	ExecutionCount int      `json:"execution_count"`
	Ename          string   `json:"ename"`
	Evalue         string   `json:"evalue"`
	Traceback      []string `json:"traceback,omitempty"`
}

// DisplayData represents a display_data message (rich output).
type DisplayData struct {
	Data     map[string]interface{}   `json:"data"`
	Metadata map[string]interface{}   `json:"metadata,omitempty"`
}

// Stream represents a stream message (stdout/stderr).
type Stream struct {
	Name string `json:"name"` // "stdout" or "stderr"
	Text string `json:"text"`
}

// KernelInfoRequest represents a kernel_info_request message.
type KernelInfoRequest struct{}

// KernelInfoReply represents a kernel_info_reply message.
type KernelInfoReply struct {
	ProtocolVersion      string                `json:"protocol_version"`
	Implementation       string                `json:"implementation"`
	ImplementationVersion string               `json:"implementation_version"`
	LanguageInfo         LanguageInfo          `json:"language_info"`
	Banner               string                `json:"banner,omitempty"`
	HelpLinks            []map[string]string   `json:"help_links,omitempty"`
	Status               string                `json:"status"` // "ok"
	WSProtocol           string                `json:"ws_protocol,omitempty"` // WebSocket protocol version for JupyterLab 4.x
}

// LanguageInfo describes the kernel's language.
type LanguageInfo struct {
	Name         string `json:"name"`
	Version      string `json:"version"`
	MIMEType     string `json:"mimetype,omitempty"`
	FileExtension string `json:"file_extension,omitempty"`
	PygmentsLexer string `json:"pygments_lexer,omitempty"`
	CodemirrorMode string `json:"codemirror_mode,omitempty"`
	NbconvertExporter string `json:"nbconvert_exporter,omitempty"`
}

// CompleteRequest represents a complete_request message (autocomplete).
type CompleteRequest struct {
	Code       string `json:"code"`
	CursorPos  int    `json:"cursor_pos"`
}

// CompleteReply represents a complete_reply message.
type CompleteReply struct {
	Status   string   `json:"status"` // "ok" or "error"
	Matches  []string `json:"matches"`
	CursorStart int   `json:"cursor_start"`
	CursorEnd   int   `json:"cursor_end"`
	Metadata  map[string]interface{} `json:"metadata,omitempty"`
}

// InspectRequest represents an inspect_request message (hover).
type InspectRequest struct {
	Code       string `json:"code"`
	CursorPos  int    `json:"cursor_pos"`
	DetailLevel int   `json:"detail_level,omitempty"` // 0=normal, 1=detailed
}

// InspectReply represents an inspect_reply message.
type InspectReply struct {
	Status string                 `json:"status"` // "ok" or "error"
	Data   map[string]interface{} `json:"data,omitempty"`
	Metadata map[string]interface{} `json:"metadata,omitempty"`
	Found  bool                   `json:"found"`
}

// IsCompleteRequest represents an is_complete_request message.
type IsCompleteRequest struct {
	Code string `json:"code"`
}

// IsCompleteReply represents an is_complete_reply message.
type IsCompleteReply struct {
	Status string `json:"status"` // "complete", "incomplete", "invalid", "unknown"
	Indent string `json:"indent,omitempty"`
}

// ShutdownRequest represents a shutdown_request message.
type ShutdownRequest struct {
	Restart bool `json:"restart"`
}

// ShutdownReply represents a shutdown_reply message.
type ShutdownReply struct {
	Restart bool `json:"restart"`
	Status  string `json:"status"` // "ok"
}

// CommMsg represents a comm_* message (for widgets, not used by Sruja).
// We can ignore these for now.

// NewMessageHeader creates a new message header.
func NewMessageHeader(msgType, session string) MessageHeader {
	return MessageHeader{
		MsgID:    generateMsgID(),
		Username: "sruja",
		Session:  session,
		MsgType:  msgType,
		Version:  "5.3",
		Date:     time.Now(),
	}
}

// generateMsgID generates a unique message ID.
func generateMsgID() string {
	// Simple UUID-like generation (can be improved)
	return time.Now().Format("20060102150405") + "-" + randomHex(8)
}

// randomHex generates a random hex string.
func randomHex(n int) string {
	bytes := make([]byte, n/2+1)
	if _, err := rand.Read(bytes); err != nil {
		// Fallback to timestamp-based if crypto/rand fails
		return time.Now().Format("20060102150405")[len(time.Now().Format("20060102150405"))-n:]
	}
	return hex.EncodeToString(bytes)[:n]
}

// CreateDisplayData creates a display_data message from kernel outputs.
func CreateDisplayData(data map[string]interface{}, metadata map[string]interface{}) DisplayData {
	if metadata == nil {
		metadata = make(map[string]interface{})
	}
	return DisplayData{
		Data:     data,
		Metadata: metadata,
	}
}

// CreateStream creates a stream message.
func CreateStream(name, text string) Stream {
	return Stream{
		Name: name,
		Text: text,
	}
}


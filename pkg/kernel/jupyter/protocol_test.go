// pkg/kernel/jupyter/protocol_test.go
package jupyter

import (
	"encoding/json"
	"testing"
	"time"
)

func TestNewMessageHeader(t *testing.T) {
	session := "test-session-123"
	msgType := "execute_request"
	
	header := NewMessageHeader(msgType, session)
	
	if header.MsgType != msgType {
		t.Errorf("Expected msg_type %q, got %q", msgType, header.MsgType)
	}
	
	if header.Session != session {
		t.Errorf("Expected session %q, got %q", session, header.Session)
	}
	
	if header.Username != "sruja" {
		t.Errorf("Expected username 'sruja', got %q", header.Username)
	}
	
	if header.Version != "5.3" {
		t.Errorf("Expected version '5.3', got %q", header.Version)
	}
	
	if header.MsgID == "" {
		t.Error("Expected non-empty msg_id")
	}
	
	if header.Date.IsZero() {
		t.Error("Expected non-zero date")
	}
	
	// Check that date is recent (within last minute)
	if time.Since(header.Date) > time.Minute {
		t.Error("Expected date to be recent")
	}
}

func TestMessageHeaderJSON(t *testing.T) {
	header := NewMessageHeader("test_message", "session-1")
	
	data, err := json.Marshal(header)
	if err != nil {
		t.Fatalf("Failed to marshal header: %v", err)
	}
	
	var decoded MessageHeader
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("Failed to unmarshal header: %v", err)
	}
	
	if decoded.MsgType != header.MsgType {
		t.Errorf("Expected msg_type %q, got %q", header.MsgType, decoded.MsgType)
	}
	
	if decoded.Session != header.Session {
		t.Errorf("Expected session %q, got %q", header.Session, decoded.Session)
	}
}

func TestCreateDisplayData(t *testing.T) {
	data := map[string]interface{}{
		"text/plain": "Hello, World!",
		"application/json": `{"key": "value"}`,
	}
	
	displayData := CreateDisplayData(data, nil)
	
	if displayData.Data["text/plain"] != data["text/plain"] {
		t.Error("Data not preserved correctly")
	}
	
	if displayData.Metadata == nil {
		t.Error("Expected metadata map to be initialized")
	}
	
	if len(displayData.Metadata) != 0 {
		t.Error("Expected empty metadata")
	}
}

func TestCreateDisplayDataWithMetadata(t *testing.T) {
	data := map[string]interface{}{
		"text/plain": "Test",
	}
	
	metadata := map[string]interface{}{
		"key": "value",
	}
	
	displayData := CreateDisplayData(data, metadata)
	
	if displayData.Metadata["key"] != "value" {
		t.Error("Metadata not preserved correctly")
	}
}

func TestCreateStream(t *testing.T) {
	stream := CreateStream("stdout", "Hello, World!")
	
	if stream.Name != "stdout" {
		t.Errorf("Expected name 'stdout', got %q", stream.Name)
	}
	
	if stream.Text != "Hello, World!" {
		t.Errorf("Expected text 'Hello, World!', got %q", stream.Text)
	}
}

func TestExecuteRequestJSON(t *testing.T) {
	req := ExecuteRequest{
		Code:         "system Test {}",
		Silent:       false,
		StoreHistory: true,
	}
	
	data, err := json.Marshal(req)
	if err != nil {
		t.Fatalf("Failed to marshal ExecuteRequest: %v", err)
	}
	
	var decoded ExecuteRequest
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("Failed to unmarshal ExecuteRequest: %v", err)
	}
	
	if decoded.Code != req.Code {
		t.Errorf("Expected code %q, got %q", req.Code, decoded.Code)
	}
	
	if decoded.Silent != req.Silent {
		t.Errorf("Expected silent %v, got %v", req.Silent, decoded.Silent)
	}
	
	if decoded.StoreHistory != req.StoreHistory {
		t.Errorf("Expected store_history %v, got %v", req.StoreHistory, decoded.StoreHistory)
	}
}

func TestExecuteReplyJSON(t *testing.T) {
	execCount := 42
	reply := ExecuteReply{
		Status:         "ok",
		ExecutionCount: &execCount,
	}
	
	data, err := json.Marshal(reply)
	if err != nil {
		t.Fatalf("Failed to marshal ExecuteReply: %v", err)
	}
	
	var decoded ExecuteReply
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("Failed to unmarshal ExecuteReply: %v", err)
	}
	
	if decoded.Status != reply.Status {
		t.Errorf("Expected status %q, got %q", reply.Status, decoded.Status)
	}
	
	if decoded.ExecutionCount == nil {
		t.Error("Expected non-nil execution_count")
	} else if *decoded.ExecutionCount != execCount {
		t.Errorf("Expected execution_count %d, got %d", execCount, *decoded.ExecutionCount)
	}
}

func TestKernelInfoReplyJSON(t *testing.T) {
	reply := KernelInfoReply{
		ProtocolVersion:      "5.3",
		Implementation:       "sruja-kernel",
		ImplementationVersion: "0.1.0",
		LanguageInfo: LanguageInfo{
			Name:    "sruja",
			Version: "1.0",
		},
		Status: "ok",
	}
	
	data, err := json.Marshal(reply)
	if err != nil {
		t.Fatalf("Failed to marshal KernelInfoReply: %v", err)
	}
	
	var decoded KernelInfoReply
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("Failed to unmarshal KernelInfoReply: %v", err)
	}
	
	if decoded.ProtocolVersion != reply.ProtocolVersion {
		t.Errorf("Expected protocol_version %q, got %q", reply.ProtocolVersion, decoded.ProtocolVersion)
	}
	
	if decoded.LanguageInfo.Name != reply.LanguageInfo.Name {
		t.Errorf("Expected language name %q, got %q", reply.LanguageInfo.Name, decoded.LanguageInfo.Name)
	}
}

func TestJupyterMessageJSON(t *testing.T) {
	header := NewMessageHeader("test_message", "session-1")
	
	content := map[string]string{
		"test": "value",
	}
	contentJSON, _ := json.Marshal(content)
	
	msg := JupyterMessage{
		Header:  header,
		Content: contentJSON,
		Metadata: make(map[string]interface{}),
	}
	
	data, err := json.Marshal(msg)
	if err != nil {
		t.Fatalf("Failed to marshal JupyterMessage: %v", err)
	}
	
	var decoded JupyterMessage
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("Failed to unmarshal JupyterMessage: %v", err)
	}
	
	if decoded.Header.MsgType != msg.Header.MsgType {
		t.Errorf("Expected msg_type %q, got %q", msg.Header.MsgType, decoded.Header.MsgType)
	}
}

func TestRandomHex(t *testing.T) {
	// Test that randomHex generates the correct length
	hex1 := randomHex(8)
	if len(hex1) != 8 {
		t.Errorf("Expected length 8, got %d", len(hex1))
	}
	
	hex2 := randomHex(16)
	if len(hex2) != 16 {
		t.Errorf("Expected length 16, got %d", len(hex2))
	}
	
	// Test that generated hex strings are valid hex
	for _, c := range hex1 {
		if !((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f')) {
			t.Errorf("Invalid hex character: %c", c)
		}
	}
	
	// Test that consecutive calls generate different values (usually)
	hex3 := randomHex(8)
	if hex1 == hex3 {
		t.Log("Warning: randomHex generated identical values (may be rare but valid)")
	}
}


// pkg/kernel/jupyter/server.go
// Jupyter kernel server implementation

package jupyter

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/sruja-ai/sruja/pkg/kernel"
)

// Server implements a Jupyter kernel server.
type Server struct {
	kernel    *kernel.Kernel
	session   string
	execCount int
	connFile  string // Connection file path (JSON with ports, keys)
}

// NewServer creates a new Jupyter kernel server.
func NewServer(k *kernel.Kernel, sessionID string) *Server {
	return &Server{
		kernel:    k,
		session:   sessionID,
		execCount: 0,
	}
}

// SetConnectionFile sets the connection file path.
// The connection file contains ZeroMQ ports and keys for communication.
func (s *Server) SetConnectionFile(path string) {
	s.connFile = path
}

// Serve starts the kernel server.
//
// If connection file is set, uses ZeroMQ transport.
// Otherwise, uses stdio transport (for VSCode/JupyterLite).
func (s *Server) Serve() error {
	// Check if connection file is provided (ZeroMQ mode)
	if s.connFile != "" {
		return s.serveZMQ()
	}

	// Default to stdio transport
	return s.serveStdio()
}

// serveStdio starts the server in stdio mode.
func (s *Server) serveStdio() error {
	// For stdio transport, read from stdin and write to stdout
	reader := bufio.NewReader(os.Stdin)

	for {
		// Read a line (JSON message)
		line, err := reader.ReadString('\n')
		if err != nil {
			if err == io.EOF {
				return nil
			}
			return fmt.Errorf("failed to read from stdin: %w", err)
		}

		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		// Parse Jupyter message
		var msg JupyterMessage
		if err := json.Unmarshal([]byte(line), &msg); err != nil {
			// Send error reply
			s.sendError(msg.Header.MsgID, "parse_error", fmt.Sprintf("Failed to parse message: %v", err), nil)
			continue
		}

		// Route message by type
		if err := s.handleMessage(&msg); err != nil {
			s.sendError(msg.Header.MsgID, "internal_error", err.Error(), &msg.Header)
		}
	}
}

// serveZMQ starts the server in ZeroMQ mode.
func (s *Server) serveZMQ() error {
	// Parse connection file
	connInfo, err := ParseConnectionFile(s.connFile)
	if err != nil {
		return fmt.Errorf("failed to parse connection file: %w", err)
	}

	// Create ZeroMQ transport
	transport, err := NewZMQTransport(connInfo, s.kernel, s.session)
	if err != nil {
		return fmt.Errorf("failed to create ZeroMQ transport: %w", err)
	}
	defer transport.Close()

	// Start serving
	return transport.Serve()
}

// handleMessage routes a message to the appropriate handler.
func (s *Server) handleMessage(msg *JupyterMessage) error {
	switch msg.Header.MsgType {
	case "kernel_info_request":
		return s.handleKernelInfo(msg)
	case "execute_request":
		return s.handleExecute(msg)
	case "complete_request":
		return s.handleComplete(msg)
	case "inspect_request":
		return s.handleInspect(msg)
	case "is_complete_request":
		return s.handleIsComplete(msg)
	case "shutdown_request":
		return s.handleShutdown(msg)
	default:
		return fmt.Errorf("unsupported message type: %s", msg.Header.MsgType)
	}
}

// handleKernelInfo responds to kernel_info_request.
func (s *Server) handleKernelInfo(msg *JupyterMessage) error {
	var req KernelInfoRequest
	if len(msg.Content) > 0 {
		if err := json.Unmarshal(msg.Content, &req); err != nil {
			return err
		}
	}

	reply := KernelInfoReply{
		ProtocolVersion:       "5.3",
		Implementation:        "sruja-kernel",
		ImplementationVersion: "0.1.0",
		LanguageInfo: LanguageInfo{
			Name:          "sruja",
			Version:       "1.0",
			MIMEType:      "text/x-sruja",
			FileExtension: ".sruja",
		},
		Banner:     "Sruja Architecture Kernel - Interactive architecture design and validation",
		Status:     "ok",
		WSProtocol: "v1", // WebSocket protocol version for JupyterLab 4.x
	}

	return s.sendReply(msg.Header.MsgID, "kernel_info_reply", reply, &msg.Header)
}

// handleExecute handles execute_request messages.
func (s *Server) handleExecute(msg *JupyterMessage) error {
	var req ExecuteRequest
	if err := json.Unmarshal(msg.Content, &req); err != nil {
		return fmt.Errorf("failed to parse execute_request: %w", err)
	}

	s.execCount++

	// Determine cell type from code content
	cellType := s.detectCellType(req.Code)

	// Execute using kernel
	cellID := kernel.CellID(fmt.Sprintf("cell-%d", s.execCount))
	result, err := s.kernel.ExecuteCell(cellID, cellType, req.Code)
	if err != nil {
		// Send error via iopub
		s.sendIOPub("error", ExecuteError{
			ExecutionCount: s.execCount,
			Ename:          "ExecutionError",
			Evalue:         err.Error(),
			Traceback:      []string{err.Error()},
		}, &msg.Header)

		// Send error reply
		return s.sendReply(msg.Header.MsgID, "execute_reply", ExecuteReply{
			Status:         "error",
			ExecutionCount: &s.execCount,
		}, &msg.Header)
	}

	// Send outputs as display_data messages
	for _, output := range result.Outputs {
		data := make(map[string]interface{})
		data[output.OutputType] = output.Data

		displayData := CreateDisplayData(data, nil)
		s.sendIOPub("display_data", displayData, &msg.Header)
	}

	// Send diagnostics if any
	if len(result.Diagnostics) > 0 {
		diagData := make(map[string]interface{})
		diagJSON, _ := json.Marshal(result.Diagnostics)
		diagData["application/sruja-diagnostics+json"] = string(diagJSON)

		// Format as text
		var diagText strings.Builder
		for _, diag := range result.Diagnostics {
			diagText.WriteString(fmt.Sprintf("[%s] %s\n", strings.ToUpper(diag.Severity), diag.Message))
		}
		diagData["text/plain"] = diagText.String()

		displayData := CreateDisplayData(diagData, nil)
		s.sendIOPub("display_data", displayData, &msg.Header)
	}

	// Send execute_reply
	status := "ok"
	if !result.Success {
		status = "error"
	}

	execReply := ExecuteReply{
		Status:         status,
		ExecutionCount: &s.execCount,
	}

	return s.sendReply(msg.Header.MsgID, "execute_reply", execReply, &msg.Header)
}

// detectCellType determines the cell type from code content.
func (s *Server) detectCellType(code string) kernel.CellType {
	code = strings.TrimSpace(code)

	// Check for simulation commands
	if strings.HasPrefix(strings.ToLower(code), "simulate") {
		return kernel.CellTypeSimulation
	}

	// Check for magic commands
	if strings.HasPrefix(code, "%") {
		return kernel.CellTypeDSL // Magic commands execute in DSL cells
	}

	// Check for diagram commands
	if strings.HasPrefix(code, "diagram") {
		return kernel.CellTypeDiagram
	}

	// Check for validation commands
	if strings.HasPrefix(code, "validate") {
		return kernel.CellTypeValidation
	}

	// Check for query (SrujaQL)
	if strings.HasPrefix(strings.ToLower(code), "find ") || strings.HasPrefix(strings.ToLower(code), "select ") {
		return kernel.CellTypeQuery
	}

	// Default to DSL
	return kernel.CellTypeDSL
}

// handleComplete handles complete_request messages (autocomplete).
func (s *Server) handleComplete(msg *JupyterMessage) error {
	var req CompleteRequest
	if err := json.Unmarshal(msg.Content, &req); err != nil {
		return fmt.Errorf("failed to parse complete_request: %w", err)
	}

	// Use kernel's LSP capabilities for completion
	// For now, return basic completions
	matches := s.getCompletions(req.Code, req.CursorPos)

	// Simple cursor position calculation
	cursorStart := req.CursorPos
	cursorEnd := req.CursorPos

	reply := CompleteReply{
		Status:      "ok",
		Matches:     matches,
		CursorStart: cursorStart,
		CursorEnd:   cursorEnd,
	}

	return s.sendReply(msg.Header.MsgID, "complete_reply", reply, &msg.Header)
}

// getCompletions returns completion suggestions.
func (s *Server) getCompletions(code string, cursorPos int) []string {
	// Basic keyword completions
	keywords := []string{
		"system", "container", "component",
		"domain", "entity", "event",
		"architecture", "workspace",
		"diagram", "validate", "find", "select",
	}

	// Get partial word at cursor
	beforeCursor := code[:cursorPos]
	words := strings.Fields(beforeCursor)
	if len(words) == 0 {
		return keywords
	}

	partial := words[len(words)-1]

	var matches []string
	for _, kw := range keywords {
		if strings.HasPrefix(kw, partial) {
			matches = append(matches, kw)
		}
	}

	return matches
}

// handleInspect handles inspect_request messages (hover).
func (s *Server) handleInspect(msg *JupyterMessage) error {
	var req InspectRequest
	if err := json.Unmarshal(msg.Content, &req); err != nil {
		return fmt.Errorf("failed to parse inspect_request: %w", err)
	}

	// Use kernel's LSP capabilities for hover
	// For now, return basic info
	reply := InspectReply{
		Status: "ok",
		Found:  false,
		Data:   make(map[string]interface{}),
	}

	return s.sendReply(msg.Header.MsgID, "inspect_reply", reply, &msg.Header)
}

// handleIsComplete handles is_complete_request messages (syntax checking).
func (s *Server) handleIsComplete(msg *JupyterMessage) error {
	var req IsCompleteRequest
	if err := json.Unmarshal(msg.Content, &req); err != nil {
		return fmt.Errorf("failed to parse is_complete_request: %w", err)
	}

	// Simple check: count braces
	openBraces := strings.Count(req.Code, "{")
	closeBraces := strings.Count(req.Code, "}")

	status := "complete"
	if openBraces > closeBraces {
		status = "incomplete"
	}

	reply := IsCompleteReply{
		Status: status,
	}

	return s.sendReply(msg.Header.MsgID, "is_complete_reply", reply, &msg.Header)
}

// handleShutdown handles shutdown_request messages.
func (s *Server) handleShutdown(msg *JupyterMessage) error {
	var req ShutdownRequest
	if err := json.Unmarshal(msg.Content, &req); err != nil {
		return fmt.Errorf("failed to parse shutdown_request: %w", err)
	}

	reply := ShutdownReply{
		Restart: req.Restart,
		Status:  "ok",
	}

	s.sendReply(msg.Header.MsgID, "shutdown_reply", reply, &msg.Header)

	// Exit if not restarting
	if !req.Restart {
		os.Exit(0)
	}

	return nil
}

// sendReply sends a reply message.
func (s *Server) sendReply(msgID, msgType string, content interface{}, parentHeader *MessageHeader) error {
	header := NewMessageHeader(msgType, s.session)
	// Reply gets a new unique msg_id, parent_header contains the original msg_id

	contentJSON, err := json.Marshal(content)
	if err != nil {
		return fmt.Errorf("failed to marshal content: %w", err)
	}

	msg := JupyterMessage{
		Header:       header,
		ParentHeader: parentHeader,
		Content:      contentJSON,
		Metadata:     make(map[string]interface{}),
	}

	return s.writeMessage(&msg)
}

// sendIOPub sends an iopub message (outputs).
func (s *Server) sendIOPub(msgType string, content interface{}, parentHeader *MessageHeader) error {
	header := NewMessageHeader(msgType, s.session)

	contentJSON, err := json.Marshal(content)
	if err != nil {
		return fmt.Errorf("failed to marshal content: %w", err)
	}

	msg := JupyterMessage{
		Header:       header,
		ParentHeader: parentHeader,
		Content:      contentJSON,
		Metadata:     make(map[string]interface{}),
	}

	return s.writeMessage(&msg)
}

// sendError sends an error message.
func (s *Server) sendError(msgID, ename, evalue string, parentHeader *MessageHeader) {
	errMsg := ExecuteError{
		Ename:     ename,
		Evalue:    evalue,
		Traceback: []string{evalue},
	}

	// Send as iopub error
	s.sendIOPub("error", errMsg, parentHeader)
}

// writeMessage writes a message to stdout.
func (s *Server) writeMessage(msg *JupyterMessage) error {
	data, err := json.Marshal(msg)
	if err != nil {
		return fmt.Errorf("failed to marshal message: %w", err)
	}

	fmt.Println(string(data))
	return nil
}

// createReply creates a reply message for a given request (used by ZMQ transport).
func (s *Server) createReply(msg *JupyterMessage) *JupyterMessage {
	// Handle the message and create appropriate reply
	var replyContent interface{}
	var replyType string

	switch msg.Header.MsgType {
	case "kernel_info_request":
		replyType = "kernel_info_reply"
		replyContent = KernelInfoReply{
			ProtocolVersion:       "5.3",
			Implementation:        "sruja-kernel",
			ImplementationVersion: "0.1.0",
			WSProtocol:            "v1", // WebSocket protocol version for JupyterLab 4.x
			LanguageInfo: LanguageInfo{
				Name:          "sruja",
				Version:       "1.0",
				MIMEType:      "text/x-sruja",
				FileExtension: ".sruja",
			},
			Banner: "Sruja Architecture Kernel - Interactive architecture design and validation",
			Status: "ok",
		}
	case "execute_request":
		var req ExecuteRequest
		if err := json.Unmarshal(msg.Content, &req); err == nil {
			s.execCount++
			cellType := s.detectCellType(req.Code)
			cellID := kernel.CellID(fmt.Sprintf("cell-%d", s.execCount))
			result, err := s.kernel.ExecuteCell(cellID, cellType, req.Code)

			if err != nil {
				replyType = "execute_reply"
				replyContent = ExecuteReply{
					Status:         "error",
					ExecutionCount: &s.execCount,
				}
			} else {
				// Send outputs via iopub (handled separately in ZMQ)
				replyType = "execute_reply"
				status := "ok"
				if !result.Success {
					status = "error"
				}
				replyContent = ExecuteReply{
					Status:         status,
					ExecutionCount: &s.execCount,
				}
			}
		}
	case "complete_request":
		var req CompleteRequest
		if err := json.Unmarshal(msg.Content, &req); err == nil {
			matches := s.getCompletions(req.Code, req.CursorPos)
			replyType = "complete_reply"
			replyContent = CompleteReply{
				Status:      "ok",
				Matches:     matches,
				CursorStart: req.CursorPos,
				CursorEnd:   req.CursorPos,
			}
		}
	case "inspect_request":
		replyType = "inspect_reply"
		replyContent = InspectReply{
			Status: "ok",
			Found:  false,
			Data:   make(map[string]interface{}),
		}
	case "is_complete_request":
		var req IsCompleteRequest
		if err := json.Unmarshal(msg.Content, &req); err == nil {
			openBraces := strings.Count(req.Code, "{")
			closeBraces := strings.Count(req.Code, "}")
			status := "complete"
			if openBraces > closeBraces {
				status = "incomplete"
			}
			replyType = "is_complete_reply"
			replyContent = IsCompleteReply{
				Status: status,
			}
		}
	default:
		replyType = "error"
		replyContent = map[string]interface{}{
			"error": fmt.Sprintf("unsupported message type: %s", msg.Header.MsgType),
		}
	}

	// Create reply message
	header := NewMessageHeader(replyType, s.session)
	// Reply gets a new unique msg_id, parent_header contains the original msg_id

	contentJSON, _ := json.Marshal(replyContent)

	return &JupyterMessage{
		Header:       header,
		ParentHeader: &msg.Header,
		Content:      contentJSON,
	}
}

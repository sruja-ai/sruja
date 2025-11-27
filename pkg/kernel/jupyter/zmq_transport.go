// pkg/kernel/jupyter/zmq_transport.go
// ZeroMQ transport implementation for Jupyter kernel

package jupyter

import (
	"context"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"strings"
	"sync"
	"time"

	"github.com/go-zeromq/zmq4"
	"github.com/sruja-ai/sruja/pkg/kernel"
)

// min returns the minimum of two integers
func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// Jupyter protocol constants
const (
	// DELIM is the Jupyter message delimiter
	DELIM = "<IDS|MSG>"
)

// ZMQTransport implements ZeroMQ transport for Jupyter kernel communication.
type ZMQTransport struct {
	connInfo  *ConnectionInfo
	kernel    *kernel.Kernel
	session   string
	execCount int

	// ZeroMQ sockets
	shellSock   zmq4.Socket
	iopubSock   zmq4.Socket
	stdinSock   zmq4.Socket
	controlSock zmq4.Socket
	hbSock      zmq4.Socket

	// Context for cancellation
	ctx    context.Context
	cancel context.CancelFunc
	wg     sync.WaitGroup
}

// NewZMQTransport creates a new ZeroMQ transport.
func NewZMQTransport(connInfo *ConnectionInfo, k *kernel.Kernel, sessionID string) (*ZMQTransport, error) {
	ctx, cancel := context.WithCancel(context.Background())

	transport := &ZMQTransport{
		connInfo:  connInfo,
		kernel:    k,
		session:   sessionID,
		execCount: 0,
		ctx:       ctx,
		cancel:    cancel,
	}

	// Create ZeroMQ sockets
	if err := transport.createSockets(); err != nil {
		cancel()
		return nil, fmt.Errorf("failed to create sockets: %w", err)
	}

	return transport, nil
}

// createSockets creates all required ZeroMQ sockets.
func (z *ZMQTransport) createSockets() error {
	// Shell socket (ROUTER for request/reply)
	shellSock := zmq4.NewRouter(z.ctx, zmq4.WithID(zmq4.SocketIdentity("shell")))
	if err := shellSock.Listen(z.connInfo.GetShellAddress()); err != nil {
		return fmt.Errorf("failed to listen on shell socket: %w", err)
	}
	z.shellSock = shellSock

	// IOPub socket (PUB for broadcasting)
	iopubSock := zmq4.NewPub(z.ctx, zmq4.WithID(zmq4.SocketIdentity("iopub")))
	if err := iopubSock.Listen(z.connInfo.GetIOPubAddress()); err != nil {
		return fmt.Errorf("failed to listen on iopub socket: %w", err)
	}
	z.iopubSock = iopubSock

	// Stdin socket (ROUTER for user input)
	stdinSock := zmq4.NewRouter(z.ctx, zmq4.WithID(zmq4.SocketIdentity("stdin")))
	if err := stdinSock.Listen(z.connInfo.GetStdinAddress()); err != nil {
		return fmt.Errorf("failed to listen on stdin socket: %w", err)
	}
	z.stdinSock = stdinSock

	// Control socket (ROUTER for control messages)
	controlSock := zmq4.NewRouter(z.ctx, zmq4.WithID(zmq4.SocketIdentity("control")))
	if err := controlSock.Listen(z.connInfo.GetControlAddress()); err != nil {
		return fmt.Errorf("failed to listen on control socket: %w", err)
	}
	z.controlSock = controlSock

	// Heartbeat socket (REP for heartbeat)
	hbSock := zmq4.NewRep(z.ctx, zmq4.WithID(zmq4.SocketIdentity("hb")))
	if err := hbSock.Listen(z.connInfo.GetHBAddress()); err != nil {
		return fmt.Errorf("failed to listen on heartbeat socket: %w", err)
	}
	z.hbSock = hbSock

	return nil
}

// Serve starts the ZeroMQ transport server.
func (z *ZMQTransport) Serve() error {
	log.Printf("Starting ZeroMQ transport on %s", z.connInfo.IP)
	log.Printf("  Shell: %d, IOPub: %d, Stdin: %d, Control: %d, HB: %d",
		z.connInfo.ShellPort, z.connInfo.IOPubPort, z.connInfo.StdinPort,
		z.connInfo.ControlPort, z.connInfo.HBPort)

	// Start goroutines for each channel
	z.wg.Add(5)

	go z.handleShell()
	go z.handleIOPub()
	go z.handleStdin()
	go z.handleControl()
	go z.handleHeartbeat()

	// Log that transport is ready
	log.Printf("ZeroMQ transport started successfully, waiting for messages...")

	// Wait for context cancellation (kernel shutdown)
	// This blocks until the kernel is shut down
	<-z.ctx.Done()

	log.Printf("ZeroMQ transport shutting down...")

	// Wait for all goroutines to finish
	z.wg.Wait()

	return z.ctx.Err()
}

// handleShell handles messages on the shell channel.
func (z *ZMQTransport) handleShell() {
	defer z.wg.Done()

	for {
		select {
		case <-z.ctx.Done():
			return
		default:
			// Receive message (ROUTER receives [identity, DELIM, signature, header, parent_header, metadata, content])
			msg, err := z.shellSock.Recv()
			if err != nil {
				if z.ctx.Err() != nil {
					return
				}
				log.Printf("Error receiving shell message: %v", err)
				time.Sleep(100 * time.Millisecond) // Brief pause before retry
				continue
			}

			// Parse message: [identity, DELIM, signature, header, parent_header, metadata, content]
			// Note: Some clients may send fewer frames, so we handle both cases
			log.Printf("Received message with %d frames", len(msg.Frames))
			if len(msg.Frames) < 3 {
				log.Printf("Invalid shell message format: expected at least 3 frames, got %d", len(msg.Frames))
				continue
			}

			// Find DELIM frame
			delimIdx := -1
			for i, frame := range msg.Frames {
				if string(frame) == DELIM {
					delimIdx = i
					break
				}
			}

			if delimIdx < 0 || delimIdx+4 >= len(msg.Frames) {
				log.Printf("Invalid message format: DELIM not found or insufficient frames (got %d frames, delimIdx: %d)", len(msg.Frames), delimIdx)
				// Log frame previews for debugging
				for i, frame := range msg.Frames {
					preview := string(frame)
					if len(preview) > 50 {
						preview = preview[:50] + "..."
					}
					log.Printf("  Frame %d (%d bytes): %q", i, len(frame), preview)
				}
				continue
			}

			log.Printf("Found DELIM at index %d, signature at %d, header at %d", delimIdx, delimIdx+1, delimIdx+2)

			// Parse Jupyter message from frames after DELIM
			// Format: [identity, DELIM, signature, header, parent_header, metadata, content]
			var jupyterMsg JupyterMessage

			// Skip identity (frame 0) and DELIM (frame delimIdx), signature is at delimIdx+1
			signatureIdx := delimIdx + 1
			headerIdx := delimIdx + 2 // Header comes after DELIM and signature

			if len(msg.Frames) <= headerIdx {
				log.Printf("Invalid message format: missing header frame (got %d frames, need at least %d)", len(msg.Frames), headerIdx+1)
				continue
			}

			// Extract signature
			receivedSignature := string(msg.Frames[signatureIdx])
			log.Printf("Received signature: %s (length: %d)", receivedSignature[:min(16, len(receivedSignature))], len(receivedSignature))

			// Parse header (should be JSON)
			if err := json.Unmarshal(msg.Frames[headerIdx], &jupyterMsg.Header); err != nil {
				log.Printf("Failed to parse header: %v (frame %d, length: %d, preview: %s)", err, headerIdx, len(msg.Frames[headerIdx]), string(msg.Frames[headerIdx][:min(100, len(msg.Frames[headerIdx]))]))
				continue
			}

			// Parse optional parent header
			var parentHeaderJSON []byte
			if len(msg.Frames) > headerIdx+1 && len(msg.Frames[headerIdx+1]) > 0 && string(msg.Frames[headerIdx+1]) != "{}" {
				parentHeaderJSON = msg.Frames[headerIdx+1]
				jupyterMsg.ParentHeader = &MessageHeader{}
				json.Unmarshal(parentHeaderJSON, jupyterMsg.ParentHeader)
			} else {
				parentHeaderJSON = []byte("{}")
			}

			// Parse optional metadata
			var metadataJSON []byte
			if len(msg.Frames) > headerIdx+2 && len(msg.Frames[headerIdx+2]) > 0 && string(msg.Frames[headerIdx+2]) != "{}" {
				metadataJSON = msg.Frames[headerIdx+2]
				jupyterMsg.Metadata = make(map[string]interface{})
				json.Unmarshal(metadataJSON, &jupyterMsg.Metadata)
			} else {
				metadataJSON = []byte("{}")
			}

			// Parse content (last frame)
			var contentJSON []byte
			if len(msg.Frames) > headerIdx+3 {
				contentJSON = msg.Frames[headerIdx+3]
				jupyterMsg.Content = contentJSON
			} else {
				contentJSON = []byte("{}")
			}

			// Verify signature
			if !z.verifySignature(receivedSignature, msg.Frames[headerIdx], parentHeaderJSON, metadataJSON, contentJSON) {
				log.Printf("Invalid signature for message %s, ignoring", jupyterMsg.Header.MsgID)
				continue
			}
			log.Printf("Signature verified successfully for message %s", jupyterMsg.Header.MsgID)

			// Handle message
			log.Printf("Received shell message: %s (msg_id: %s)", jupyterMsg.Header.MsgType, jupyterMsg.Header.MsgID)
			reply := z.handleMessage(&jupyterMsg)

			// Send outputs via iopub if this was an execute_request
			if jupyterMsg.Header.MsgType == "execute_request" {
				z.sendExecuteOutputs(&jupyterMsg)
			}

			// Send signed reply
			log.Printf("Sending shell reply: %s (msg_id: %s, parent_msg_id: %s)",
				reply.Header.MsgType, reply.Header.MsgID,
				func() string {
					if reply.ParentHeader != nil {
						return reply.ParentHeader.MsgID
					}
					return "none"
				}())
			if err := z.sendSignedMessage(z.shellSock, msg.Frames[0], reply); err != nil {
				log.Printf("Error sending shell reply: %v", err)
			} else {
				log.Printf("Successfully sent shell reply")
			}
		}
	}
}

// handleIOPub handles iopub channel (for now, just a placeholder).
func (z *ZMQTransport) handleIOPub() {
	defer z.wg.Done()
	// IOPub is write-only (PUB socket), so we just wait for context cancellation
	<-z.ctx.Done()
}

// handleStdin handles stdin channel messages.
func (z *ZMQTransport) handleStdin() {
	defer z.wg.Done()

	for {
		select {
		case <-z.ctx.Done():
			return
		default:
			msg, err := z.stdinSock.Recv()
			if err != nil {
				if z.ctx.Err() != nil {
					return
				}
				log.Printf("Error receiving stdin message: %v", err)
				continue
			}

			// Handle stdin input (for now, just log it)
			if len(msg.Frames) >= 3 {
				log.Printf("Stdin input received: %s", string(msg.Frames[len(msg.Frames)-1]))
			}
		}
	}
}

// handleControl handles control channel messages.
func (z *ZMQTransport) handleControl() {
	defer z.wg.Done()

	for {
		select {
		case <-z.ctx.Done():
			return
		default:
			msg, err := z.controlSock.Recv()
			if err != nil {
				if z.ctx.Err() != nil {
					return
				}
				log.Printf("Error receiving control message: %v", err)
				continue
			}

			// Parse message: [identity, DELIM, signature, header, parent_header, metadata, content]
			// Find DELIM frame
			delimIdx := -1
			for i, frame := range msg.Frames {
				if string(frame) == DELIM {
					delimIdx = i
					break
				}
			}

			if delimIdx < 0 || delimIdx+4 >= len(msg.Frames) {
				continue
			}

			var jupyterMsg JupyterMessage
			headerIdx := delimIdx + 2 // Header comes after DELIM and signature

			if len(msg.Frames) <= headerIdx {
				continue
			}

			if err := json.Unmarshal(msg.Frames[headerIdx], &jupyterMsg.Header); err != nil {
				log.Printf("Failed to parse control header: %v", err)
				continue
			}
			if len(msg.Frames) > headerIdx+1 && len(msg.Frames[headerIdx+1]) > 0 && string(msg.Frames[headerIdx+1]) != "{}" {
				jupyterMsg.ParentHeader = &MessageHeader{}
				json.Unmarshal(msg.Frames[headerIdx+1], jupyterMsg.ParentHeader)
			}
			if len(msg.Frames) > headerIdx+2 && len(msg.Frames[headerIdx+2]) > 0 && string(msg.Frames[headerIdx+2]) != "{}" {
				jupyterMsg.Metadata = make(map[string]interface{})
				json.Unmarshal(msg.Frames[headerIdx+2], &jupyterMsg.Metadata)
			}
			if len(msg.Frames) > headerIdx+3 {
				jupyterMsg.Content = msg.Frames[headerIdx+3]
			}

			// Handle control messages (interrupt, shutdown, etc.)
			reply := z.handleControlMessage(&jupyterMsg)

			// Send signed reply
			if err := z.sendSignedMessage(z.controlSock, msg.Frames[0], reply); err != nil {
				log.Printf("Error sending control reply: %v", err)
			}
		}
	}
}

// handleHeartbeat handles heartbeat messages.
// Heartbeat uses REQ/REP pattern - just echo back what we receive.
func (z *ZMQTransport) handleHeartbeat() {
	defer z.wg.Done()
	log.Printf("Heartbeat handler started")

	for {
		select {
		case <-z.ctx.Done():
			log.Printf("Heartbeat handler stopping")
			return
		default:
			msg, err := z.hbSock.Recv()
			if err != nil {
				if z.ctx.Err() != nil {
					return
				}
				// Heartbeat errors are usually not critical, just log and continue
				time.Sleep(100 * time.Millisecond)
				continue
			}

			// Echo back the heartbeat immediately (REP socket pattern)
			// Heartbeat messages are just raw bytes, no parsing needed
			if err := z.hbSock.Send(msg); err != nil {
				log.Printf("Error sending heartbeat reply: %v", err)
			}
		}
	}
}

// handleMessage processes a Jupyter message and returns a reply.
func (z *ZMQTransport) handleMessage(msg *JupyterMessage) *JupyterMessage {
	// Validate message
	if msg.Header.MsgType == "" {
		log.Printf("Received message with empty msg_type")
		return &JupyterMessage{
			Header: MessageHeader{
				MsgID:    generateMsgID(),
				MsgType:  "error",
				Username: "sruja",
				Session:  z.session,
				Version:  "5.3",
				Date:     time.Now(),
			},
			ParentHeader: &msg.Header,
			Content:      []byte(`{"error": "Invalid message: empty msg_type"}`),
		}
	}

	// Create a temporary server instance to reuse message handling logic
	tempServer := &Server{
		kernel:    z.kernel,
		session:   z.session,
		execCount: z.execCount,
	}
	z.execCount++

	// Handle the message and get reply
	reply := tempServer.createReply(msg)
	return reply
}

// handleControlMessage handles control channel messages.
func (z *ZMQTransport) handleControlMessage(msg *JupyterMessage) *JupyterMessage {
	// Handle interrupt, shutdown, etc.
	switch msg.Header.MsgType {
	case "shutdown_request":
		// Graceful shutdown
		z.cancel()
		return &JupyterMessage{
			Header: MessageHeader{
				MsgID:    generateMsgID(),
				Username: "sruja",
				Session:  z.session,
				MsgType:  "shutdown_reply",
				Version:  "5.3",
				Date:     time.Now(),
			},
			ParentHeader: &msg.Header,
		}
	case "interrupt_request":
		// Interrupt execution (for now, just acknowledge)
		return &JupyterMessage{
			Header: MessageHeader{
				MsgID:    generateMsgID(),
				Username: "sruja",
				Session:  z.session,
				MsgType:  "interrupt_reply",
				Version:  "5.3",
				Date:     time.Now(),
			},
			ParentHeader: &msg.Header,
		}
	default:
		// Unknown control message
		return &JupyterMessage{
			Header: MessageHeader{
				MsgID:    generateMsgID(),
				Username: "sruja",
				Session:  z.session,
				MsgType:  "error",
				Version:  "5.3",
				Date:     time.Now(),
			},
			ParentHeader: &msg.Header,
		}
	}
}

// sendExecuteOutputs sends execution outputs via iopub channel.
func (z *ZMQTransport) sendExecuteOutputs(msg *JupyterMessage) {
	var req ExecuteRequest
	if err := json.Unmarshal(msg.Content, &req); err != nil {
		return
	}

	// Execute using kernel (reuse server's detectCellType)
	tempServer := &Server{
		kernel:  z.kernel,
		session: z.session,
	}
	cellType := tempServer.detectCellType(req.Code)
	cellID := kernel.CellID(fmt.Sprintf("cell-%d", z.execCount))
	result, err := z.kernel.ExecuteCell(cellID, cellType, req.Code)
	if err != nil {
		// Send error via iopub
		errorMsg := &JupyterMessage{
			Header: MessageHeader{
				MsgID:    generateMsgID(),
				Username: "sruja",
				Session:  z.session,
				MsgType:  "error",
				Version:  "5.3",
				Date:     time.Now(),
			},
			ParentHeader: &msg.Header,
		}
		errorContent, _ := json.Marshal(ExecuteError{
			ExecutionCount: z.execCount,
			Ename:          "ExecutionError",
			Evalue:         err.Error(),
			Traceback:      []string{err.Error()},
		})
		errorMsg.Content = errorContent
		z.SendIOPub(errorMsg)
		return
	}

	// Send outputs as display_data messages
	for _, output := range result.Outputs {
		data := make(map[string]interface{})
		data[output.OutputType] = output.Data

		displayData := CreateDisplayData(data, nil)
		displayMsg := &JupyterMessage{
			Header: MessageHeader{
				MsgID:    generateMsgID(),
				Username: "sruja",
				Session:  z.session,
				MsgType:  "display_data",
				Version:  "5.3",
				Date:     time.Now(),
			},
			ParentHeader: &msg.Header,
		}
		displayContent, _ := json.Marshal(displayData)
		displayMsg.Content = displayContent
		z.SendIOPub(displayMsg)
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
		displayMsg := &JupyterMessage{
			Header: MessageHeader{
				MsgID:    generateMsgID(),
				Username: "sruja",
				Session:  z.session,
				MsgType:  "display_data",
				Version:  "5.3",
				Date:     time.Now(),
			},
			ParentHeader: &msg.Header,
		}
		displayContent, _ := json.Marshal(displayData)
		displayMsg.Content = displayContent
		z.SendIOPub(displayMsg)
	}
}

// computeSignature computes HMAC-SHA256 signature for a Jupyter message.
func (z *ZMQTransport) computeSignature(header, parentHeader, metadata, content []byte) string {
	if z.connInfo.Key == "" {
		return "" // No signing if no key
	}

	// Signature is computed over: header + parent_header + metadata + content
	// Note: Jupyter concatenates the JSON strings directly (not as bytes, but as the JSON strings)
	mac := hmac.New(sha256.New, []byte(z.connInfo.Key))
	mac.Write(header)
	mac.Write(parentHeader)
	mac.Write(metadata)
	mac.Write(content)
	return hex.EncodeToString(mac.Sum(nil))
}

// verifySignature verifies the HMAC-SHA256 signature of an incoming message.
func (z *ZMQTransport) verifySignature(signature string, header, parentHeader, metadata, content []byte) bool {
	if z.connInfo.Key == "" {
		return signature == "" // If no key, signature should be empty
	}

	expected := z.computeSignature(header, parentHeader, metadata, content)
	isValid := hmac.Equal([]byte(signature), []byte(expected))

	if !isValid {
		log.Printf("Signature verification failed: expected %s, got %s", expected[:min(16, len(expected))], signature[:min(16, len(signature))])
	}

	return isValid
}

// sendSignedMessage sends a signed Jupyter message over ZeroMQ.
func (z *ZMQTransport) sendSignedMessage(sock zmq4.Socket, identity []byte, msg *JupyterMessage) error {
	// Marshal message parts - must match exactly what Jupyter expects
	headerJSON, _ := json.Marshal(msg.Header)

	// Parent header must be exact JSON, not empty object if it exists
	var parentHeaderJSON []byte
	if msg.ParentHeader != nil {
		parentHeaderJSON, _ = json.Marshal(msg.ParentHeader)
	} else {
		parentHeaderJSON = []byte("{}")
	}

	// Metadata must be exact JSON
	var metadataJSON []byte
	if msg.Metadata != nil && len(msg.Metadata) > 0 {
		metadataJSON, _ = json.Marshal(msg.Metadata)
	} else {
		metadataJSON = []byte("{}")
	}

	// Content must be exact JSON
	contentJSON := msg.Content
	if contentJSON == nil || len(contentJSON) == 0 {
		contentJSON = []byte("{}")
	}

	// Compute signature - Jupyter uses HMAC-SHA256 over the concatenated JSON strings
	signature := z.computeSignature(headerJSON, parentHeaderJSON, metadataJSON, contentJSON)

	if signature == "" && z.connInfo.Key != "" {
		log.Printf("Warning: No signature computed but key exists")
	}

	// Build message frames: [identity, DELIM, signature, header, parent_header, metadata, content]
	zmqMsg := zmq4.NewMsgFrom(
		identity,
		[]byte(DELIM),
		[]byte(signature),
		headerJSON,
		parentHeaderJSON,
		metadataJSON,
		contentJSON,
	)

	return sock.Send(zmqMsg)
}

// SendIOPub sends a message on the iopub channel.
func (z *ZMQTransport) SendIOPub(msg *JupyterMessage) error {
	// IOPub doesn't need identity (PUB socket)
	// But we still need signature, header, parent_header, metadata, content
	headerJSON, _ := json.Marshal(msg.Header)
	parentHeaderJSON := []byte("{}")
	if msg.ParentHeader != nil {
		parentHeaderJSON, _ = json.Marshal(msg.ParentHeader)
	}
	metadataJSON := []byte("{}")
	if msg.Metadata != nil {
		metadataJSON, _ = json.Marshal(msg.Metadata)
	}
	contentJSON := msg.Content
	if contentJSON == nil {
		contentJSON = []byte("{}")
	}

	signature := z.computeSignature(headerJSON, parentHeaderJSON, metadataJSON, contentJSON)

	zmqMsg := zmq4.NewMsgFrom(
		[]byte(DELIM),
		[]byte(signature),
		headerJSON,
		parentHeaderJSON,
		metadataJSON,
		contentJSON,
	)

	return z.iopubSock.Send(zmqMsg)
}

// detectCellType determines the cell type from code content.
func (z *ZMQTransport) detectCellType(code string) kernel.CellType {
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

// Close closes all sockets and cleans up.
func (z *ZMQTransport) Close() error {
	z.cancel()
	z.wg.Wait()

	// Close sockets
	z.shellSock.Close()
	z.iopubSock.Close()
	z.stdinSock.Close()
	z.controlSock.Close()
	z.hbSock.Close()

	return nil
}

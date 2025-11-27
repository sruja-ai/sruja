package mcp

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"sync"

	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

type Server struct {
	mu    sync.Mutex
	tools map[string]func(map[string]interface{}) (*CallToolResult, error)
}

func NewServer() *Server {
	s := &Server{
		tools: make(map[string]func(map[string]interface{}) (*CallToolResult, error)),
	}
	s.registerCoreTools()
	return s
}

func (s *Server) registerCoreTools() {
	s.RegisterTool("validate", s.handleValidateTool)
	s.RegisterTool("compile", s.handleCompileTool)
}

func (s *Server) handleValidateTool(args map[string]interface{}) (*CallToolResult, error) {
	code, ok := args["code"].(string)
	if !ok {
		return nil, fmt.Errorf("missing 'code' argument")
	}

	// Parse
	parser, err := language.NewParser()
	if err != nil {
		return &CallToolResult{
			Content: []Content{{Type: "text", Text: fmt.Sprintf("Parser initialization error: %v", err)}},
			IsError: true,
		}, nil
	}
	program, err := parser.Parse("validate.sruja", code)
	if err != nil {
		return &CallToolResult{
			Content: []Content{{Type: "text", Text: fmt.Sprintf("Parser Errors:\n%v", err)}},
			IsError: true,
		}, nil
	}

	// Validate
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	validator.RegisterRule(&engine.OrphanDetectionRule{})
	validator.RegisterRule(&engine.ExternalBestPracticeRule{})
	validationErrors := validator.Validate(program)

	if len(validationErrors) > 0 {
		var errMsgs []string
		for _, err := range validationErrors {
			errMsgs = append(errMsgs, err.String())
		}
		return &CallToolResult{
			Content: []Content{{Type: "text", Text: fmt.Sprintf("Validation Errors:\n%s", strings.Join(errMsgs, "\n"))}},
			IsError: true,
		}, nil
	}

	return &CallToolResult{
		Content: []Content{{Type: "text", Text: "Validation successful. No errors found."}},
	}, nil
}

func (s *Server) handleCompileTool(args map[string]interface{}) (*CallToolResult, error) {
	code, ok := args["code"].(string)
	if !ok {
		return nil, fmt.Errorf("missing 'code' argument")
	}

	// Parse
	parser, err := language.NewParser()
	if err != nil {
		return &CallToolResult{
			Content: []Content{{Type: "text", Text: fmt.Sprintf("Parser initialization error: %v", err)}},
			IsError: true,
		}, nil
	}
	program, err := parser.Parse("validate.sruja", code)
	if err != nil {
		return &CallToolResult{
			Content: []Content{{Type: "text", Text: fmt.Sprintf("Parser Errors:\n%v", err)}},
			IsError: true,
		}, nil
	}

	// Validate (optional, but good practice before compile)
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	validator.RegisterRule(&engine.OrphanDetectionRule{})
	validator.RegisterRule(&engine.ExternalBestPracticeRule{})
	validationErrors := validator.Validate(program)

	if len(validationErrors) > 0 {
		var errMsgs []string
		for _, err := range validationErrors {
			errMsgs = append(errMsgs, err.String())
		}
		return &CallToolResult{
			Content: []Content{{Type: "text", Text: fmt.Sprintf("Validation Errors:\n%s", strings.Join(errMsgs, "\n"))}},
			IsError: true,
		}, nil
	}

	// Compile
	c := compiler.NewD2Compiler()
	mermaidCode, err := c.Compile(program)
	if err != nil {
		return &CallToolResult{
			Content: []Content{{Type: "text", Text: fmt.Sprintf("Compilation error: %v", err)}},
			IsError: true,
		}, nil
	}

	return &CallToolResult{
		Content: []Content{{Type: "text", Text: mermaidCode}},
	}, nil
}

func (s *Server) RegisterTool(name string, handler func(map[string]interface{}) (*CallToolResult, error)) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.tools[name] = handler
}

func (s *Server) Serve() {
	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		line := scanner.Bytes()
		s.handleMessage(line)
	}
}

func (s *Server) handleMessage(data []byte) {
	var req JSONRPCRequest
	if err := json.Unmarshal(data, &req); err != nil {
		s.sendError(nil, -32700, "Parse error")
		return
	}

	switch req.Method {
	case "initialize":
		s.handleInitialize(req.ID)
	case "tools/list":
		s.handleToolsList(req.ID)
	case "tools/call":
		s.handleToolsCall(req, req.ID)
	case "resources/list":
		s.handleResourcesList(req.ID)
	case "resources/read":
		s.handleResourcesRead(req, req.ID)
	case "notifications/initialized":
		// No response needed
	default:
		// For now, ignore unknown methods or return method not found
		// s.sendError(req.ID, -32601, "Method not found")
	}
}

func (s *Server) handleInitialize(id interface{}) {
	result := InitializeResult{
		ProtocolVersion: "2025-06-18",
		Capabilities: Capabilities{
			Tools:     &ToolsCapability{},
			Resources: &ResourcesCapability{},
		},
		ServerInfo: ServerInfo{
			Name:    "sruja-mcp",
			Version: "0.1.0",
		},
	}
	s.sendResult(id, result)
}

func (s *Server) handleResourcesList(id interface{}) {
	resources := []Resource{}

	// Walk current directory to find .sruja files
	// In a real implementation, we might want to respect .gitignore or have a configured root
	cwd, err := os.Getwd()
	if err != nil {
		s.sendError(id, -32000, fmt.Sprintf("Failed to get cwd: %v", err))
		return
	}

	// Simple recursive walk
	var walk func(path string)
	walk = func(path string) {
		entries, err := os.ReadDir(path)
		if err != nil {
			return
		}
		for _, entry := range entries {
			fullPath := fmt.Sprintf("%s/%s", path, entry.Name())
			if entry.IsDir() {
				if entry.Name() == ".git" || entry.Name() == "node_modules" {
					continue
				}
				walk(fullPath)
			} else {
				if strings.HasSuffix(entry.Name(), ".sruja") {
					// Create a file URI
					uri := fmt.Sprintf("file://%s", fullPath)
					resources = append(resources, Resource{
						URI:      uri,
						Name:     entry.Name(),
						MIMEType: "text/x-sruja",
					})
				}
			}
		}
	}
	walk(cwd)

	s.sendResult(id, ListResourcesResult{Resources: resources})
}

func (s *Server) handleResourcesRead(req JSONRPCRequest, id interface{}) {
	var params ReadResourceParams
	if err := json.Unmarshal(req.Params, &params); err != nil {
		s.sendError(id, -32602, "Invalid params")
		return
	}

	// Basic validation of URI
	if !strings.HasPrefix(params.URI, "file://") {
		s.sendError(id, -32002, "Only file:// URIs are supported")
		return
	}

	path := strings.TrimPrefix(params.URI, "file://")
	content, err := os.ReadFile(path)
	if err != nil {
		s.sendError(id, -32002, fmt.Sprintf("Failed to read file: %v", err))
		return
	}

	s.sendResult(id, ReadResourceResult{
		Contents: []ResourceContent{
			{
				URI:      params.URI,
				MIMEType: "text/x-sruja",
				Text:     string(content),
			},
		},
	})
}

func (s *Server) handleToolsList(id interface{}) {
	// For now, we'll hardcode the list based on what we plan to implement
	// In a real implementation, this would be dynamic based on registered tools
	tools := []Tool{
		{
			Name:        "validate",
			Description: "Validate a Sruja architecture file",
			InputSchema: json.RawMessage(`{
				"type": "object",
				"properties": {
					"code": { "type": "string", "description": "Sruja source code" }
				},
				"required": ["code"]
			}`),
		},
		{
			Name:        "compile",
			Description: "Compile Sruja code to Mermaid",
			InputSchema: json.RawMessage(`{
				"type": "object",
				"properties": {
					"code": { "type": "string", "description": "Sruja source code" }
				},
				"required": ["code"]
			}`),
		},
	}

	s.sendResult(id, map[string]interface{}{
		"tools": tools,
	})
}

func (s *Server) handleToolsCall(req JSONRPCRequest, id interface{}) {
	var params CallToolParams
	if err := json.Unmarshal(req.Params, &params); err != nil {
		s.sendError(id, -32602, "Invalid params")
		return
	}

	s.mu.Lock()
	handler, ok := s.tools[params.Name]
	s.mu.Unlock()

	if !ok {
		s.sendError(id, -32601, fmt.Sprintf("Tool not found: %s", params.Name))
		return
	}

	result, err := handler(params.Arguments)
	if err != nil {
		s.sendResult(id, CallToolResult{
			Content: []Content{{Type: "text", Text: err.Error()}},
			IsError: true,
		})
		return
	}

	s.sendResult(id, result)
}

func (s *Server) sendResult(id interface{}, result interface{}) {
	resp := JSONRPCResponse{
		JSONRPC: "2.0",
		ID:      id,
		Result:  result,
	}
	s.writeResponse(resp)
}

func (s *Server) sendError(id interface{}, code int, message string) {
	resp := JSONRPCResponse{
		JSONRPC: "2.0",
		ID:      id,
		Error: &JSONRPCError{
			Code:    code,
			Message: message,
		},
	}
	s.writeResponse(resp)
}

func (s *Server) writeResponse(resp JSONRPCResponse) {
	bytes, _ := json.Marshal(resp)
	fmt.Fprintf(os.Stdout, "%s\n", string(bytes))
}

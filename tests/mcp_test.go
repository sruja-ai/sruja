//go:build legacy

package tests

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

type JSONRPCRequest struct {
	JSONRPC string      `json:"jsonrpc"`
	Method  string      `json:"method"`
	Params  interface{} `json:"params,omitempty"`
	ID      int         `json:"id"`
}

type JSONRPCResponse struct {
	JSONRPC string          `json:"jsonrpc"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   interface{}     `json:"error,omitempty"`
	ID      int             `json:"id"`
}

func TestMCPIntegration(t *testing.T) {
	// Build the binary first to ensure we are testing the latest code
	cwd, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get cwd: %v", err)
	}
	projectRoot := filepath.Dir(cwd)
	binPath := filepath.Join(projectRoot, "sruja_test_bin")

	buildCmd := exec.Command("go", "build", "-o", binPath, "./apps/cli/cmd")
	buildCmd.Dir = projectRoot
	if err := buildCmd.Run(); err != nil {
		t.Fatalf("Failed to build sruja binary: %v", err)
	}
	// Cleanup binary after test
	defer exec.Command("rm", binPath).Run()

	// Start the MCP server
	cmd := exec.Command(binPath, "mcp")
	cmd.Dir = projectRoot // Run in project root so it finds examples/
	stdin, err := cmd.StdinPipe()
	if err != nil {
		t.Fatalf("Failed to get stdin pipe: %v", err)
	}
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		t.Fatalf("Failed to get stdout pipe: %v", err)
	}

	if err := cmd.Start(); err != nil {
		t.Fatalf("Failed to start sruja mcp: %v", err)
	}
	defer cmd.Process.Kill()

	reader := bufio.NewReader(stdout)

	// Helper to send request and get response
	sendRequest := func(req JSONRPCRequest) JSONRPCResponse {
		bytes, _ := json.Marshal(req)
		fmt.Fprintf(stdin, "%s\n", string(bytes))

		line, err := reader.ReadString('\n')
		if err != nil {
			t.Fatalf("Failed to read response: %v", err)
		}

		var resp JSONRPCResponse
		if err := json.Unmarshal([]byte(line), &resp); err != nil {
			t.Fatalf("Failed to unmarshal response: %v", err)
		}
		return resp
	}

	// 1. Initialize
	initReq := JSONRPCRequest{
		JSONRPC: "2.0",
		Method:  "initialize",
		Params: map[string]interface{}{
			"protocolVersion": "2025-06-18",
			"capabilities":    map[string]interface{}{},
			"clientInfo":      map[string]string{"name": "test-client", "version": "1.0"},
		},
		ID: 1,
	}
	resp := sendRequest(initReq)
	if resp.Error != nil {
		t.Fatalf("Initialize failed: %v", resp.Error)
	}

	// 2. List Tools
	listReq := JSONRPCRequest{
		JSONRPC: "2.0",
		Method:  "tools/list",
		ID:      2,
	}
	resp = sendRequest(listReq)
	var listResult struct {
		Tools []struct {
			Name string `json:"name"`
		} `json:"tools"`
	}
	json.Unmarshal(resp.Result, &listResult)

	hasValidate := false
	hasCompile := false
	for _, tool := range listResult.Tools {
		if tool.Name == "validate" {
			hasValidate = true
		}
		if tool.Name == "compile" {
			hasCompile = true
		}
	}
	if !hasValidate || !hasCompile {
		t.Errorf("Missing tools. Got: %v", listResult.Tools)
	}

	// 3. Call Validate (Valid)
	validCode := `
	workspace {
		model {
			system User "User"
			system System "System"
			User -> System "uses"
		}
	}
	`
	validateReq := JSONRPCRequest{
		JSONRPC: "2.0",
		Method:  "tools/call",
		Params: map[string]interface{}{
			"name":      "validate",
			"arguments": map[string]string{"code": validCode},
		},
		ID: 3,
	}
	resp = sendRequest(validateReq)
	var validateResult struct {
		Content []struct {
			Text string `json:"text"`
		} `json:"content"`
		IsError bool `json:"isError"`
	}
	json.Unmarshal(resp.Result, &validateResult)
	if validateResult.IsError {
		t.Errorf("Expected valid code to pass validation, got error: %v", validateResult.Content)
	}

	// 4. Call Validate (Invalid)
	invalidCode := `
	workspace {
		model {
			system User "User"
			system User "User"
		}
	}
	`
	validateInvalidReq := JSONRPCRequest{
		JSONRPC: "2.0",
		Method:  "tools/call",
		Params: map[string]interface{}{
			"name":      "validate",
			"arguments": map[string]string{"code": invalidCode},
		},
		ID: 4,
	}
	resp = sendRequest(validateInvalidReq)
	json.Unmarshal(resp.Result, &validateResult)
	if !validateResult.IsError {
		t.Errorf("Expected invalid code to fail validation")
	}
	if !strings.Contains(validateResult.Content[0].Text, "Duplicate identifier") {
		t.Errorf("Expected 'Duplicate identifier' error, got: %s", validateResult.Content[0].Text)
	}

	// 5. List Resources
	listResourcesReq := JSONRPCRequest{
		JSONRPC: "2.0",
		Method:  "resources/list",
		ID:      5,
	}
	resp = sendRequest(listResourcesReq)
	var listResourcesResult struct {
		Resources []struct {
			URI  string `json:"uri"`
			Name string `json:"name"`
		} `json:"resources"`
	}
	json.Unmarshal(resp.Result, &listResourcesResult)

	// We expect at least example.sruja and full_features.sruja from previous steps
	foundExample := false
	var exampleURI string
	for _, res := range listResourcesResult.Resources {
		if res.Name == "example.sruja" {
			foundExample = true
			exampleURI = res.URI
		}
	}
	if !foundExample {
		t.Errorf("Expected to find example.sruja in resources, got: %v", listResourcesResult.Resources)
	}

	// 6. Read Resource
	if foundExample {
		readResourceReq := JSONRPCRequest{
			JSONRPC: "2.0",
			Method:  "resources/read",
			Params: map[string]string{
				"uri": exampleURI,
			},
			ID: 6,
		}
		resp = sendRequest(readResourceReq)
		var readResourceResult struct {
			Contents []struct {
				Text string `json:"text"`
			} `json:"contents"`
		}
		json.Unmarshal(resp.Result, &readResourceResult)

		if len(readResourceResult.Contents) == 0 {
			t.Errorf("Expected content for resource %s", exampleURI)
		} else {
			content := readResourceResult.Contents[0].Text
			if !strings.Contains(content, "workspace") {
				t.Errorf("Expected content to contain 'workspace', got: %s", content)
			}
		}
	}
}

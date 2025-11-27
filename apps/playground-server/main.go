// apps/playground-server/main.go
package main

import (
	"encoding/json"
	"log"
	"net/http"
	"os"

	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

type CompileRequest struct {
	Code string `json:"code"`
}

type CompileResponse struct {
	D2       string   `json:"d2,omitempty"`
	SVG      string   `json:"svg,omitempty"` // Rendered SVG for D2
	Errors   []string `json:"errors,omitempty"`
	Warnings []string `json:"warnings,omitempty"`
	Valid    bool     `json:"valid"`
}

type ValidateRequest struct {
	Code string `json:"code"`
}

type ValidateResponse struct {
	Valid    bool     `json:"valid"`
	Errors   []string `json:"errors,omitempty"`
	Warnings []string `json:"warnings,omitempty"`
}

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	http.HandleFunc("/api/compile", handleCompile)
	http.HandleFunc("/api/validate", handleValidate)
	http.HandleFunc("/health", handleHealth)

	log.Printf("Playground server starting on :%s", port)
	log.Fatal(http.ListenAndServe(":"+port, nil))
}

func handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
}

func handleCompile(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req CompileRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	response := compileCode(req.Code)
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")
	json.NewEncoder(w).Encode(response)
}

func handleValidate(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req ValidateRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	response := validateCode(req.Code)
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")
	json.NewEncoder(w).Encode(response)
}

func compileCode(code string) CompileResponse {
	parser, err := language.NewParser()
	if err != nil {
		return CompileResponse{
			Valid:    false,
			Errors:   []string{err.Error()},
			Warnings: []string{},
		}
	}

	program, err := parser.Parse("playground.sruja", code)
	if err != nil {
		return CompileResponse{
			Valid:    false,
			Errors:   []string{err.Error()},
			Warnings: []string{},
		}
	}

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
		return CompileResponse{
			Valid:    false,
			Errors:   errMsgs,
			Warnings: []string{},
		}
	}

	// Compile to D2
	d2Compiler := compiler.NewD2Compiler()
	output, compileErr := d2Compiler.Compile(program)
	if compileErr != nil {
		return CompileResponse{
			Valid:    false,
			Errors:   []string{compileErr.Error()},
			Warnings: []string{},
		}
	}

	// Try to render to SVG (may fail if D2 libraries not available)
	var svgOutput string
	renderer := compiler.NewD2Renderer()
	svgBytes, svgErr := renderer.RenderToSVG(output)
	if svgErr == nil {
		svgOutput = string(svgBytes)
	}
	// If SVG rendering fails, we still return D2 code

	return CompileResponse{
		Valid:    true,
		D2:       output,
		SVG:      svgOutput,
		Errors:   []string{},
		Warnings: []string{},
	}
}

func validateCode(code string) ValidateResponse {
	parser, err := language.NewParser()
	if err != nil {
		return ValidateResponse{
			Valid:    false,
			Errors:   []string{err.Error()},
			Warnings: []string{},
		}
	}

	program, err := parser.Parse("playground.sruja", code)
	if err != nil {
		return ValidateResponse{
			Valid:    false,
			Errors:   []string{err.Error()},
			Warnings: []string{},
		}
	}

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
		return ValidateResponse{
			Valid:    false,
			Errors:   errMsgs,
			Warnings: []string{},
		}
	}

	return ValidateResponse{
		Valid:    true,
		Errors:   []string{},
		Warnings: []string{},
	}
}

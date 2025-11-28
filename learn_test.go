// learn_test.go
// Tests for Sruja code compilation in playground and course content
package main

import (
	"context"
	"log/slog"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
	"oss.terrastruct.com/d2/d2graph"
	"oss.terrastruct.com/d2/d2layouts/d2dagrelayout"
	"oss.terrastruct.com/d2/d2lib"
	"oss.terrastruct.com/d2/d2renderers/d2svg"
	d2log "oss.terrastruct.com/d2/lib/log"
	"oss.terrastruct.com/d2/lib/textmeasure"
)

// extractPlaygroundExamples extracts Sruja code examples from playground.html
func extractPlaygroundExamples() (map[string]string, error) {
	content, err := os.ReadFile("learn/layouts/shortcodes/playground.html")
	if err != nil {
		return nil, err
	}

	examples := make(map[string]string)

	// Match: "Example Name": `code here`
	re := regexp.MustCompile(`"([^"]+)":\s*` + "`([^`]+)`")
	matches := re.FindAllStringSubmatch(string(content), -1)

	for _, match := range matches {
		if len(match) == 3 {
			name := match[1]
			code := strings.ReplaceAll(match[2], "\\n", "\n")
			examples[name] = code
		}
	}

	return examples, nil
}

// isSyntaxExample checks if code is a syntax example (not runnable)
func isSyntaxExample(code string) bool {
	if hasPlaceholderSyntax(code) {
		return true
	}
	if isStandaloneTags(code) {
		return true
	}
	if isStandaloneElement(code) {
		return true
	}
	if isRelationWithoutDefinitions(code) {
		return true
	}
	return false
}

// hasPlaceholderSyntax checks for ... placeholder syntax
func hasPlaceholderSyntax(code string) bool {
	return strings.Contains(code, "...")
}

// isStandaloneTags checks if code is just standalone tags
func isStandaloneTags(code string) bool {
	return strings.HasPrefix(strings.TrimSpace(code), "tags [")
}

// isStandaloneElement checks if code is a standalone element without architecture wrapper
func isStandaloneElement(code string) bool {
	trimmed := strings.TrimSpace(code)
	standalonePatterns := []string{
		"^container ", "^component ", "^datastore ", "^queue ",
		"^person ", "^system ", "^deployment ", "^node ",
		"^containerInstance ", "^dynamic ",
	}
	for _, pattern := range standalonePatterns {
		if matched, _ := regexp.MatchString(pattern, trimmed); matched {
			if !strings.Contains(code, "architecture") {
				return true
			}
		}
	}
	return false
}

// isRelationWithoutDefinitions checks if code has relations but no element definitions
func isRelationWithoutDefinitions(code string) bool {
	trimmed := strings.TrimSpace(code)
	if !strings.Contains(trimmed, "->") {
		return false
	}
	hasDefinitions := strings.Contains(trimmed, "architecture") ||
		strings.Contains(trimmed, "system") ||
		strings.Contains(trimmed, "person") ||
		strings.Contains(trimmed, "container") ||
		strings.Contains(trimmed, "component") ||
		strings.Contains(trimmed, "datastore")
	return !hasDefinitions
}

// extractCodeBlocks extracts Sruja code blocks from markdown files
func extractCodeBlocks(rootDir string) (map[string]string, error) {
	codeBlocks := make(map[string]string)

	err := filepath.Walk(rootDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if !strings.HasSuffix(path, ".md") {
			return nil
		}

		content, err := os.ReadFile(path)
		if err != nil {
			return err
		}

		// Match ```sruja ... ``` blocks
		re := regexp.MustCompile("```sruja\\s*\n([\\s\\S]*?)```")
		matches := re.FindAllStringSubmatch(string(content), -1)

		for i, match := range matches {
			if len(match) >= 2 {
				code := strings.TrimSpace(match[1])
				if code != "" && !isSyntaxExample(code) {
					blockName := filepath.Base(path) + "#block" + string(rune(i+1))
					codeBlocks[blockName] = code
				}
			}
		}

		return nil
	})

	return codeBlocks, err
}

// compileCode compiles Sruja code and returns error if compilation fails
// skipOrphanCheck: if true, skips orphan detection (useful for course examples)
// skipSVGRender: if true, skips SVG rendering (only tests parse/validate/export)
func compileCode(name, code string, skipOrphanCheck bool, skipSVGRender bool) error {
	program, err := parseSrujaCode(name, code)
	if err != nil {
		return err
	}

	if err := validateSrujaCode(program, skipOrphanCheck); err != nil {
		return err
	}

	d2Script, err := exportToD2(program)
	if err != nil {
		return err
	}

	if skipSVGRender {
		return nil
	}

	return renderD2ToSVG(d2Script)
}

// parseSrujaCode parses Sruja code into an AST
func parseSrujaCode(name, code string) (*language.Program, error) {
	parser, err := language.NewParser()
	if err != nil {
		return nil, err
	}
	return parser.Parse(name, code)
}

// validateSrujaCode validates the parsed architecture
func validateSrujaCode(program *language.Program, skipOrphanCheck bool) error {
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	if !skipOrphanCheck {
		validator.RegisterRule(&engine.OrphanDetectionRule{})
	}

	errors := validator.Validate(program)
	if len(errors) == 0 {
		return nil
	}

	var msgs []string
	for _, e := range errors {
		msgs = append(msgs, e.Error())
	}
	return &CompilationError{Message: strings.Join(msgs, "; ")}
}

// exportToD2 exports the architecture to D2 script
func exportToD2(program *language.Program) (string, error) {
	exporter := d2.NewExporter()
	return exporter.Export(program.Architecture)
}

// renderD2ToSVG compiles and renders D2 script to SVG
func renderD2ToSVG(d2Script string) error {
	logger := slog.New(slog.NewTextHandler(nil, &slog.HandlerOptions{Level: slog.LevelError}))
	ctx := d2log.With(context.Background(), logger)

	ruler, err := textmeasure.NewRuler()
	if err != nil {
		return err
	}

	layout := "dagre"
	diagram, _, err := d2lib.Compile(ctx, d2Script, &d2lib.CompileOptions{
		Ruler:  ruler,
		Layout: &layout,
		LayoutResolver: func(engine string) (d2graph.LayoutGraph, error) {
			return func(ctx context.Context, g *d2graph.Graph) error {
				return d2dagrelayout.Layout(ctx, g, nil)
			}, nil
		},
	}, nil)
	if err != nil {
		return err
	}

	pad := int64(d2svg.DEFAULT_PADDING)
	renderOpts := &d2svg.RenderOpts{Pad: &pad}
	_, err = d2svg.Render(diagram, renderOpts)
	return err
}

type CompilationError struct {
	Message string
}

func (e *CompilationError) Error() string {
	return e.Message
}

func TestPlaygroundExamples(t *testing.T) {
	examples, err := extractPlaygroundExamples()
	if err != nil {
		t.Fatalf("Failed to extract playground examples: %v", err)
	}

	if len(examples) == 0 {
		t.Fatal("No playground examples found")
	}

	t.Logf("Found %d playground examples", len(examples))

	for name, code := range examples {
		t.Run(name, func(t *testing.T) {
			// Playground examples must generate SVG (full pipeline test)
			if err := compileCode("playground-"+name, code, false, false); err != nil {
				t.Errorf("Playground example '%s' failed to compile: %v\nCode:\n%s", name, err, code)
			}
		})
	}
}

func TestCourseCodeBlocks(t *testing.T) {
	courseDir := "learn/content/courses"

	// Check if directory exists
	if _, err := os.Stat(courseDir); os.IsNotExist(err) {
		t.Skipf("Course directory %s does not exist, skipping", courseDir)
	}

	codeBlocks, err := extractCodeBlocks(courseDir)
	if err != nil {
		t.Fatalf("Failed to extract code blocks: %v", err)
	}

	if len(codeBlocks) == 0 {
		t.Skip("No code blocks found in course files")
	}

	t.Logf("Found %d code blocks in course files", len(codeBlocks))

	for name, code := range codeBlocks {
		t.Run(name, func(t *testing.T) {
			// Skip orphan check and SVG rendering for course examples
			// They may be intentionally standalone and some features (like journeys) may have D2 rendering limitations
			if err := compileCode(name, code, true, true); err != nil {
				t.Errorf("Course code block '%s' failed to compile: %v\nCode:\n%s", name, err, code)
			}
		})
	}
}

func TestDocsCodeBlocks(t *testing.T) {
	docsDir := "learn/content/docs"

	// Check if directory exists
	if _, err := os.Stat(docsDir); os.IsNotExist(err) {
		t.Skipf("Docs directory %s does not exist, skipping", docsDir)
	}

	codeBlocks, err := extractCodeBlocks(docsDir)
	if err != nil {
		t.Fatalf("Failed to extract code blocks: %v", err)
	}

	if len(codeBlocks) == 0 {
		t.Skip("No code blocks found in docs files")
	}

	t.Logf("Found %d code blocks in docs files", len(codeBlocks))

	for name, code := range codeBlocks {
		t.Run(name, func(t *testing.T) {
			// Skip orphan check and SVG rendering for docs examples
			// They may be intentionally standalone and some features may have D2 rendering limitations
			if err := compileCode(name, code, true, true); err != nil {
				t.Errorf("Docs code block '%s' failed to compile: %v\nCode:\n%s", name, err, code)
			}
		})
	}
}

// website_test.go
// Tests for Sruja code compilation in playground and course content
package tests

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
	// D2 and SVG export removed - Studio will provide visualization
)

// ExampleMetadata holds metadata about an example from manifest
type ExampleMetadata struct {
	SkipTest        bool
	SkipOrphanCheck bool
	SkipSVGRender   bool
	ExpectedFailure string
	Reason          string
}

// ManifestEntry represents an entry in the examples manifest
type ManifestEntry struct {
	File            string `json:"file"`
	Name            string `json:"name"`
	Order           int    `json:"order"`
	Category        string `json:"category,omitempty"`
	Description     string `json:"description,omitempty"`
	SkipPlayground  bool   `json:"skipPlayground,omitempty"`
	SkipOrphanCheck bool   `json:"skipOrphanCheck,omitempty"`
	ExpectedFailure string `json:"expectedFailure,omitempty"`
}

// Manifest represents the examples manifest
type Manifest struct {
	Examples []ManifestEntry `json:"examples"`
}

// extractPlaygroundExamples extracts Sruja code examples from examples/ directory
// Uses examples/manifest/examples.json as the source of truth for metadata
func extractPlaygroundExamples() (examples map[string]string, metadata map[string]ExampleMetadata, err error) {
	manifestPath := "../examples/manifest.json"
	examplesDir := "../examples"
	examples = make(map[string]string)
	metadata = make(map[string]ExampleMetadata)

	// Read manifest
	manifestData, err := os.ReadFile(manifestPath)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to read manifest: %w", err)
	}

	var manifest Manifest
	if err := json.Unmarshal(manifestData, &manifest); err != nil {
		return nil, nil, fmt.Errorf("failed to parse manifest: %w", err)
	}

	// Build map of files to metadata
	manifestMap := make(map[string]ManifestEntry)
	for _, entry := range manifest.Examples {
		manifestMap[entry.File] = entry
	}

	// Read all .sruja files and apply metadata from manifest
	err = filepath.Walk(examplesDir, buildExampleWalker(examples, metadata, manifestMap))

	return examples, metadata, err
}

// buildExampleWalker creates a filepath.WalkFunc that processes .sruja files
func buildExampleWalker(examples map[string]string, metadata map[string]ExampleMetadata, manifestMap map[string]ManifestEntry) filepath.WalkFunc {
	return func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if info.IsDir() {
			if filepath.Base(path) == "course" {
				return filepath.SkipDir
			}
			return nil
		}

		if !strings.HasSuffix(path, ".sruja") {
			return nil
		}

		return processExampleFile(path, examples, metadata, manifestMap)
	}
}

// processExampleFile processes a single .sruja file and adds it to examples/metadata maps
func processExampleFile(path string, examples map[string]string, metadata map[string]ExampleMetadata, manifestMap map[string]ManifestEntry) error {
	filename := filepath.Base(path)
	entry, hasManifest := manifestMap[filename]

	if hasManifest && entry.SkipPlayground {
		return nil
	}

	content, err := os.ReadFile(path)
	if err != nil {
		return err
	}

	contentStr := string(content)
	name := strings.TrimSuffix(filename, ".sruja")
	meta := buildMetadata(&entry, hasManifest, contentStr)

	if meta.SkipTest {
		return nil
	}

	examples[name] = contentStr
	metadata[name] = meta
	return nil
}

// buildMetadata constructs ExampleMetadata from manifest entry and legacy comments
func buildMetadata(entry *ManifestEntry, hasManifest bool, content string) ExampleMetadata {
	meta := ExampleMetadata{}
	if hasManifest {
		meta.SkipOrphanCheck = entry.SkipOrphanCheck
		meta.ExpectedFailure = entry.ExpectedFailure
		if entry.ExpectedFailure != "" {
			meta.Reason = entry.ExpectedFailure
		}
	}

	legacyMeta := parseLegacyComments(content)
	if legacyMeta.SkipTest {
		meta.SkipTest = true
	}
	if legacyMeta.SkipOrphanCheck {
		meta.SkipOrphanCheck = true
	}
	if legacyMeta.ExpectedFailure != "" && meta.ExpectedFailure == "" {
		meta.ExpectedFailure = legacyMeta.ExpectedFailure
	}

	return meta
}

// parseLegacyComments extracts metadata from special comment tags (for backward compatibility)
func parseLegacyComments(content string) ExampleMetadata {
	meta := ExampleMetadata{}
	lines := strings.Split(content, "\n")

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		checkLegacyComment(trimmed, &meta)
	}

	return meta
}

// checkLegacyComment checks a single line for legacy comment tags
func checkLegacyComment(trimmed string, meta *ExampleMetadata) {
	switch {
	case strings.HasPrefix(trimmed, "// SKIP_TEST:") || trimmed == "// SKIP_TEST":
		meta.SkipTest = true
	case strings.HasPrefix(trimmed, "// SKIP_ORPHAN_CHECK:") || trimmed == "// SKIP_ORPHAN_CHECK":
		meta.SkipOrphanCheck = true
	case strings.HasPrefix(trimmed, "// SKIP_SVG_RENDER:") || trimmed == "// SKIP_SVG_RENDER":
		meta.SkipSVGRender = true
	case strings.HasPrefix(trimmed, "// EXPECTED_FAILURE:"):
		meta.ExpectedFailure = strings.TrimSpace(strings.TrimPrefix(trimmed, "// EXPECTED_FAILURE:"))
	}
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

// removeComments removes line comments from code
func removeComments(code string) string {
	lines := strings.Split(code, "\n")
	var result []string
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if !strings.HasPrefix(trimmed, "//") {
			result = append(result, line)
		}
	}
	return strings.Join(result, "\n")
}

// isStandaloneTags checks if code is just standalone tags
func isStandaloneTags(code string) bool {
	return strings.HasPrefix(strings.TrimSpace(removeComments(code)), "tags [")
}

// isStandaloneElement checks if code is a standalone element without architecture wrapper
func isStandaloneElement(code string) bool {
	trimmed := strings.TrimSpace(removeComments(code))
	standalonePatterns := []string{
		"^container ", "^component ", "^datastore ", "^queue ",
		"^person ", "^system ", "^deployment ", "^node ",
		"^containerInstance ", "^scenario ",
		"^adr ", "^requirement ", "^policy ",
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
	trimmed := strings.TrimSpace(removeComments(code))
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

// compile regex once for performance
var (
	systemRegexStr    = regexp.MustCompile(`system\s+\w+`)
	systemDetailRegex = regexp.MustCompile(`system\s+\w+\s+"[^"]+"\s+"[^"]+"\s+{`)
	relationRegex     = regexp.MustCompile(`\b\w+\.\w+\b`)
	propertyRegex     = regexp.MustCompile(`\w+\s*:\s*"`)
)

// CodeBlockMetadata holds metadata extracted from code blocks
type CodeBlockMetadata struct {
	ExpectedFailure string
	SkipOrphanCheck bool
}

// extractCodeBlocks extracts Sruja code blocks from markdown files
// Returns both code blocks and their metadata
func extractCodeBlocks(rootDir string) (codeBlocks map[string]string, metadata map[string]CodeBlockMetadata, err error) {
	codeBlocks = make(map[string]string)
	metadata = make(map[string]CodeBlockMetadata)

	err = filepath.Walk(rootDir, func(path string, _ os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Check for both .md and .mdx files
		if !strings.HasSuffix(path, ".md") && !strings.HasSuffix(path, ".mdx") {
			return nil
		}

		content, err := os.ReadFile(path)
		if err != nil {
			return err
		}

		// Match ```sruja ... ``` blocks
		// Pattern matches: ```sruja (optional whitespace/newline) (content) ```
		// The (?s) flag makes . match newlines
		re := regexp.MustCompile("(?s)```sruja\\s*\n([\\s\\S]*?)```")
		matches := re.FindAllStringSubmatch(string(content), -1)

		for i, match := range matches {
			if len(match) >= 2 {
				code := strings.TrimSpace(match[1])
				if code != "" && !isSyntaxExample(code) {
					blockName := fmt.Sprintf("%s#block%d", filepath.Base(path), i+1)
					codeBlocks[blockName] = code
					// Extract metadata from code block comments
					metadata[blockName] = extractCodeBlockMetadata(code)
				}
			}
		}

		return nil
	})

	return codeBlocks, metadata, err
}

// extractCodeBlockMetadata extracts metadata from code block comments
func extractCodeBlockMetadata(code string) CodeBlockMetadata {
	meta := CodeBlockMetadata{}
	lines := strings.Split(code, "\n")

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, "// EXPECTED_FAILURE:") {
			meta.ExpectedFailure = strings.TrimSpace(strings.TrimPrefix(trimmed, "// EXPECTED_FAILURE:"))
		}
		if strings.HasPrefix(trimmed, "// SKIP_ORPHAN_CHECK:") || trimmed == "// SKIP_ORPHAN_CHECK" {
			meta.SkipOrphanCheck = true
		}
	}

	return meta
}

// usesDeferredFeature is deprecated - all examples should use only supported features
// This function now always returns false - no skipping should occur
// All content should be updated to use only supported DSL syntax
func usesDeferredFeature(code string) bool {
	// All examples should use only supported features - no skipping logic
	// Content should be updated to remove any unsupported syntax
	return false
}

// compileCode compiles Sruja code and returns error if compilation fails
// skipOrphanCheck: if true, skips orphan detection (useful for course examples)
// skipSVGRender: deprecated - D2/SVG export removed, Studio will provide visualization
func compileCode(name, code string, skipOrphanCheck, _ bool) error {
	parser, err := language.NewParser()
	if err != nil {
		return err
	}
	program, _, err := parser.Parse(name, code) // Changed to use name and code parameters
	if err != nil {
		return err
	}

	if err := validateSrujaCode(program, skipOrphanCheck); err != nil {
		// Filter out informational cycle messages (cycles are valid in many architectures)
		if errStr := err.Error(); !strings.Contains(errStr, "Cycle detected") || !strings.Contains(errStr, "valid") {
			return err
		}
		// Cycles are valid - skip informational messages
		// Continue with compilation
	}

	// D2 and SVG export removed - Studio will provide visualization
	// Tests now only verify parsing and validation
	// Export functionality will be tested separately when Studio is implemented
	return nil
}

// validateSrujaCode validates the parsed architecture
func validateSrujaCode(program *language.Program, skipOrphanCheck bool) error {
	validator := engine.NewValidator()
	validator.RegisterRule(&engine.UniqueIDRule{})
	validator.RegisterRule(&engine.ValidReferenceRule{})
	validator.RegisterRule(&engine.CycleDetectionRule{})
	validator.RegisterRule(&engine.ExternalDependencyRule{})
	validator.RegisterRule(&engine.SimplicityRule{}) // Guide users toward appropriate syntax
	validator.RegisterRule(&engine.LayerViolationRule{})
	if !skipOrphanCheck {
		validator.RegisterRule(&engine.OrphanDetectionRule{})
	}

	diags := validator.Validate(program)
	if len(diags) == 0 {
		return nil
	}

	// Filter out informational messages (cycles are valid, simplicity warnings are guidance)
	var blockingErrors []diagnostics.Diagnostic
	for i := range diags {
		d := &diags[i]
		// Skip informational cycle detection messages (cycles are valid patterns)
		if d.Code == diagnostics.CodeCycleDetected && d.Severity == diagnostics.SeverityInfo {
			continue // Cycles are valid - skip informational messages
		}
		// Skip simplicity guidance warnings (they're suggestions, not errors)
		// SimplicityRule messages contain "Consider using" and are guidance, not blocking
		if strings.Contains(d.Message, "Consider using") {
			continue // Simplicity warnings are guidance, not blocking errors
		}
		// Only consider Errors as blocking, unless we want to fail on Warnings too
		// For now, let's treat Errors as blocking
		if d.Severity == diagnostics.SeverityError {
			blockingErrors = append(blockingErrors, *d)
		}
	}

	if len(blockingErrors) == 0 {
		return nil
	}

	msgs := make([]string, 0, len(blockingErrors))
	for i := range blockingErrors {
		d := &blockingErrors[i]
		msgs = append(msgs, diagnostics.FormatDiagnostic(*d))
	}
	return &CompilationError{Message: strings.Join(msgs, "; ")}
}

// D2 and SVG export functions removed - Studio will provide visualization
// Tests now only verify parsing and validation

type CompilationError struct {
	Message string
}

func (e *CompilationError) Error() string {
	return e.Message
}

func TestPlaygroundExamples(t *testing.T) {
	examples, metadata, err := extractPlaygroundExamples()
	if err != nil {
		t.Fatalf("Failed to extract playground examples: %v", err)
	}

	if len(examples) == 0 {
		t.Fatal("No playground examples found")
	}

	t.Logf("Found %d playground examples", len(examples))

	for name, code := range examples {
		meta := metadata[name]
		t.Run(name, func(t *testing.T) {
			// All examples should use only supported features - no skipping

			// Tests now only verify parsing and validation (D2/SVG export removed)
			skipOrphan := meta.SkipOrphanCheck
			skipSVGRender := true // Always skip - D2/SVG export removed

			err := compileCode("playground-"+name, code, skipOrphan, skipSVGRender)

			if meta.ExpectedFailure != "" {
				// This example is expected to fail
				if err == nil {
					t.Errorf("Playground example '%s' was expected to fail (%s) but compiled successfully", name, meta.ExpectedFailure)
				} else {
					t.Logf("Playground example '%s' failed as expected: %s - %v", name, meta.ExpectedFailure, err)
				}
			} else {
				// This example should compile successfully
				if err != nil {
					t.Errorf("Playground example '%s' failed to compile: %v\nCode:\n%s", name, err, code)
				}
			}
		})
	}
}

func TestCourseCodeBlocks(t *testing.T) {
	courseDir := "../apps/website/src/content/courses"

	// Check if directory exists
	if _, err := os.Stat(courseDir); os.IsNotExist(err) {
		t.Skipf("Course directory %s does not exist, skipping", courseDir)
	}

	codeBlocks, metadata, err := extractCodeBlocks(courseDir)
	if err != nil {
		t.Fatalf("Failed to extract code blocks: %v", err)
	}

	if len(codeBlocks) == 0 {
		t.Skip("No code blocks found in course files")
	}

	t.Logf("Found %d code blocks in course files", len(codeBlocks))

	for name, code := range codeBlocks {
		meta := metadata[name]
		t.Run(name, func(t *testing.T) {
			// All examples should use only supported features - no skipping

			// Use regular compiler with optional orphan check skip
			skipOrphan := meta.SkipOrphanCheck
			err := compileCode("course-"+name, code, skipOrphan, true)

			if meta.ExpectedFailure != "" {
				// This code block is expected to fail
				if err == nil {
					t.Errorf("Course code block '%s' was expected to fail (%s) but compiled successfully", name, meta.ExpectedFailure)
				} else {
					t.Logf("Course code block '%s' failed as expected: %s - %v", name, meta.ExpectedFailure, err)
				}
			} else {
				// This code block should compile successfully
				if err != nil {
					t.Errorf("Course code block '%s' failed to compile: %v\nCode:\n%s", name, err, code)
				}
			}
		})
	}
}

func TestDocsCodeBlocks(t *testing.T) {
	docsDir := "../apps/website/src/content/docs"

	// Check if directory exists
	if _, err := os.Stat(docsDir); os.IsNotExist(err) {
		t.Skipf("Docs directory %s does not exist, skipping", docsDir)
	}

	codeBlocks, metadata, err := extractCodeBlocks(docsDir)
	if err != nil {
		t.Fatalf("Failed to extract code blocks: %v", err)
	}

	if len(codeBlocks) == 0 {
		t.Skip("No code blocks found in docs files")
	}

	t.Logf("Found %d code blocks in docs files", len(codeBlocks))

	for name, code := range codeBlocks {
		meta := metadata[name]
		t.Run(name, func(t *testing.T) {
			// All examples should use only supported features - no skipping
			// Use regular compiler with optional orphan check skip
			skipOrphan := meta.SkipOrphanCheck
			err := compileCode("docs-"+name, code, skipOrphan, true)

			if meta.ExpectedFailure != "" {
				// This code block is expected to fail
				if err == nil {
					t.Errorf("Docs code block '%s' was expected to fail (%s) but compiled successfully", name, meta.ExpectedFailure)
				} else {
					t.Logf("Docs code block '%s' failed as expected: %s - %v", name, meta.ExpectedFailure, err)
				}
			} else {
				// This code block should compile successfully
				if err != nil {
					t.Errorf("Docs code block '%s' failed to compile: %v\nCode:\n%s", name, err, code)
				}
			}
		})
	}
}

func TestTutorialCodeBlocks(t *testing.T) {
	tutorialsDir := "../apps/website/src/content/tutorials"

	// Check if directory exists
	if _, err := os.Stat(tutorialsDir); os.IsNotExist(err) {
		t.Skipf("Tutorials directory %s does not exist, skipping", tutorialsDir)
	}

	codeBlocks, metadata, err := extractCodeBlocks(tutorialsDir)
	if err != nil {
		t.Fatalf("Failed to extract code blocks: %v", err)
	}

	if len(codeBlocks) == 0 {
		t.Skip("No code blocks found in tutorial files")
	}

	t.Logf("Found %d code blocks in tutorial files", len(codeBlocks))

	for name, code := range codeBlocks {
		meta := metadata[name]
		t.Run(name, func(t *testing.T) {
			// All examples should use only supported features - no skipping
			// Use regular compiler with optional orphan check skip
			skipOrphan := meta.SkipOrphanCheck
			err := compileCode("tutorial-"+name, code, skipOrphan, true)

			if meta.ExpectedFailure != "" {
				// This code block is expected to fail
				if err == nil {
					t.Errorf("Tutorial code block '%s' was expected to fail (%s) but compiled successfully", name, meta.ExpectedFailure)
				} else {
					t.Logf("Tutorial code block '%s' failed as expected: %s - %v", name, meta.ExpectedFailure, err)
				}
			} else {
				// This code block should compile successfully
				if err != nil {
					t.Errorf("Tutorial code block '%s' failed to compile: %v\nCode:\n%s", name, err, code)
				}
			}
		})
	}
}

func TestBlogCodeBlocks(t *testing.T) {
	blogDir := "../apps/website/src/content/blog"

	// Check if directory exists
	if _, err := os.Stat(blogDir); os.IsNotExist(err) {
		t.Skipf("Blog directory %s does not exist, skipping", blogDir)
	}

	codeBlocks, metadata, err := extractCodeBlocks(blogDir)
	if err != nil {
		t.Fatalf("Failed to extract code blocks: %v", err)
	}

	if len(codeBlocks) == 0 {
		t.Skip("No code blocks found in blog files")
	}

	t.Logf("Found %d code blocks in blog files", len(codeBlocks))

	for name, code := range codeBlocks {
		meta := metadata[name]
		t.Run(name, func(t *testing.T) {
			// All examples should use only supported features - no skipping
			// Use regular compiler with optional orphan check skip
			skipOrphan := meta.SkipOrphanCheck
			err := compileCode("blog-"+name, code, skipOrphan, true)

			if meta.ExpectedFailure != "" {
				// This code block is expected to fail
				if err == nil {
					t.Errorf("Blog code block '%s' was expected to fail (%s) but compiled successfully", name, meta.ExpectedFailure)
				} else {
					t.Logf("Blog code block '%s' failed as expected: %s - %v", name, meta.ExpectedFailure, err)
				}
			} else {
				// This code block should compile successfully
				if err != nil {
					t.Errorf("Blog code block '%s' failed to compile: %v\nCode:\n%s", name, err, code)
				}
			}
		})
	}
}

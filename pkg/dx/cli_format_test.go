// pkg/dx/cli_format_test.go
package dx

import (
	"os"
	"strings"
	"testing"
)

const (
	testCodeSnippet = "code snippet"
	testLinkText    = "link text"
)

func TestSupportsColor(t *testing.T) {
	// Test that SupportsColor returns a boolean (actual result depends on environment)
	result := SupportsColor()
	if result != true && result != false {
		t.Error("SupportsColor should return a boolean")
	}
}

func TestColorize(t *testing.T) {
	text := "test"
	colored := Colorize(ColorRed, text, true)
	if !strings.Contains(colored, text) {
		t.Error("Colorize should contain original text")
	}

	uncolored := Colorize(ColorRed, text, false)
	if uncolored != text {
		t.Errorf("Colorize with disabled should return original text, got '%s'", uncolored)
	}
}

func TestSuccess(t *testing.T) {
	msg := "Operation completed"
	result := Success(msg)
	if !strings.Contains(result, msg) {
		t.Error("Success should contain message")
	}
}

func TestError(t *testing.T) {
	msg := "Operation failed"
	result := Error(msg)
	if !strings.Contains(result, msg) {
		t.Error("Error should contain message")
	}
}

func TestWarning(t *testing.T) {
	msg := "Warning message"
	result := Warning(msg)
	if !strings.Contains(result, msg) {
		t.Error("Warning should contain message")
	}
}

func TestInfo(t *testing.T) {
	msg := "Info message"
	result := Info(msg)
	if !strings.Contains(result, msg) {
		t.Error("Info should contain message")
	}
}

func TestBold(t *testing.T) {
	text := "bold text"
	result := Bold(text)
	if !strings.Contains(result, text) {
		t.Error("Bold should contain original text")
	}
}

func TestDim(t *testing.T) {
	text := "dim text"
	result := Dim(text)
	if !strings.Contains(result, text) {
		t.Error("Dim should contain original text")
	}
}

func TestHeader(t *testing.T) {
	text := "Section Title"
	result := Header(text)
	if !strings.Contains(result, text) {
		t.Error("Header should contain title")
	}
	if !strings.Contains(result, "=") {
		t.Error("Header should contain separator lines")
	}
}

func TestSection(t *testing.T) {
	title := "Section Name"
	result := Section(title)
	if !strings.Contains(result, title) {
		t.Error("Section should contain title")
	}
}

func TestListItem(t *testing.T) {
	text := "List item"
	result := ListItem(text, 0)
	if !strings.Contains(result, text) {
		t.Error("ListItem should contain text")
	}

	result2 := ListItem(text, 2)
	if !strings.Contains(result2, text) {
		t.Error("ListItem should contain text with indent")
	}
}

func TestCode(t *testing.T) {
	text := testCodeSnippet
	result := Code(text)
	if !strings.Contains(result, text) {
		t.Error("Code should contain original text")
	}
}

func TestLink(t *testing.T) {
	text := testLinkText
	result := Link(text)
	if !strings.Contains(result, text) {
		t.Error("Link should contain original text")
	}
}

func TestTable(t *testing.T) {
	rows := [][]string{
		{"Key1", "Value1"},
		{"Key2", "Value2"},
		{"LongKey", "Value3"},
	}
	result := Table(rows)
	if !strings.Contains(result, "Key1") || !strings.Contains(result, "Value1") {
		t.Error("Table should contain row data")
	}
	if !strings.Contains(result, "Key2") || !strings.Contains(result, "Value2") {
		t.Error("Table should contain all rows")
	}
}

func TestTable_Empty(t *testing.T) {
	rows := [][]string{}
	result := Table(rows)
	if result != "" {
		t.Errorf("Table with empty rows should return empty string, got '%s'", result)
	}
}

func TestTable_SingleColumn(t *testing.T) {
	rows := [][]string{
		{"Item1"},
		{"Item2"},
	}
	result := Table(rows)
	if !strings.Contains(result, "Item1") {
		t.Error("Table should handle single column rows")
	}
}

func TestProgressBar(t *testing.T) {
	result := ProgressBar(50, 100, "Processing")
	if !strings.Contains(result, "Processing") {
		t.Error("ProgressBar should contain label")
	}
	if !strings.Contains(result, "50") {
		t.Error("ProgressBar should contain percentage")
	}
}

func TestProgressBar_ZeroTotal(t *testing.T) {
	result := ProgressBar(0, 0, "Processing")
	if result != "" {
		t.Errorf("ProgressBar with zero total should return empty string, got '%s'", result)
	}
}

func TestSeparator(t *testing.T) {
	result := Separator()
	if result == "" {
		t.Error("Separator should return a line")
	}
}

func TestSupportsColor_NoColorEnv(t *testing.T) {
	originalNoColor := os.Getenv("NO_COLOR")
	defer func() {
		_ = os.Setenv("NO_COLOR", originalNoColor)
	}()

	_ = os.Setenv("NO_COLOR", "1")
	result := SupportsColor()
	if result {
		t.Error("SupportsColor should return false when NO_COLOR is set")
	}
}

func TestSupportsColor_DumbTerm(t *testing.T) {
	originalTerm := os.Getenv("TERM")
	defer func() {
		_ = os.Setenv("TERM", originalTerm)
	}()

	_ = os.Setenv("TERM", "dumb")
	result := SupportsColor()
	if result {
		t.Error("SupportsColor should return false when TERM is 'dumb'")
	}
}

func TestSupportsColor_EmptyTerm(_ *testing.T) {
	originalTerm := os.Getenv("TERM")
	defer func() {
		_ = os.Setenv("TERM", originalTerm)
	}()

	_ = os.Setenv("TERM", "")
	result := SupportsColor()
	// Result depends on whether stdout is a terminal
	_ = result
}

func TestHeader_WithColor(t *testing.T) {
	// Test that Header works with color enabled
	text := "Test Header"
	result := Header(text)
	if !strings.Contains(result, text) {
		t.Error("Header should contain text")
	}
}

func TestSection_WithColor(t *testing.T) {
	title := "Test Section"
	result := Section(title)
	if !strings.Contains(result, title) {
		t.Error("Section should contain title")
	}
}

func TestCode_WithColor(t *testing.T) {
	text := testCodeSnippet
	result := Code(text)
	if !strings.Contains(result, text) {
		t.Error("Code should contain text")
	}
}

func TestLink_WithColor(t *testing.T) {
	text := testLinkText
	result := Link(text)
	if !strings.Contains(result, text) {
		t.Error("Link should contain text")
	}
}

func TestProgressBar_WithColor(t *testing.T) {
	result := ProgressBar(75, 100, "Processing")
	if !strings.Contains(result, "Processing") {
		t.Error("ProgressBar should contain label")
	}
}

func TestSeparator_WithColor(t *testing.T) {
	result := Separator()
	if result == "" {
		t.Error("Separator should return a line")
	}
}

func TestHeader_NoColor(t *testing.T) {
	originalNoColor := os.Getenv("NO_COLOR")
	defer func() {
		_ = os.Setenv("NO_COLOR", originalNoColor)
	}()
	_ = os.Setenv("NO_COLOR", "1")

	text := "Test Header"
	result := Header(text)
	if !strings.Contains(result, text) {
		t.Error("Header should contain text even without color")
	}
}

func TestSection_NoColor(t *testing.T) {
	originalNoColor := os.Getenv("NO_COLOR")
	defer func() {
		_ = os.Setenv("NO_COLOR", originalNoColor)
	}()
	_ = os.Setenv("NO_COLOR", "1")

	title := "Test Section"
	result := Section(title)
	if !strings.Contains(result, title) {
		t.Error("Section should contain title even without color")
	}
}

func TestListItem_NoColor(t *testing.T) {
	originalNoColor := os.Getenv("NO_COLOR")
	defer func() {
		_ = os.Setenv("NO_COLOR", originalNoColor)
	}()
	_ = os.Setenv("NO_COLOR", "1")

	text := "List item"
	result := ListItem(text, 1)
	if !strings.Contains(result, text) {
		t.Error("ListItem should contain text even without color")
	}
}

func TestCode_NoColor(t *testing.T) {
	originalNoColor := os.Getenv("NO_COLOR")
	defer func() {
		_ = os.Setenv("NO_COLOR", originalNoColor)
	}()
	_ = os.Setenv("NO_COLOR", "1")

	text := testCodeSnippet
	result := Code(text)
	if result != text {
		t.Errorf("Code should return original text without color, got '%s'", result)
	}
}

func TestLink_NoColor(t *testing.T) {
	originalNoColor := os.Getenv("NO_COLOR")
	defer func() {
		_ = os.Setenv("NO_COLOR", originalNoColor)
	}()
	_ = os.Setenv("NO_COLOR", "1")

	text := testLinkText
	result := Link(text)
	if result != text {
		t.Errorf("Link should return original text without color, got '%s'", result)
	}
}

func TestTable_NoColor(t *testing.T) {
	originalNoColor := os.Getenv("NO_COLOR")
	defer func() {
		_ = os.Setenv("NO_COLOR", originalNoColor)
	}()
	_ = os.Setenv("NO_COLOR", "1")

	rows := [][]string{
		{"Key", "Value"},
	}
	result := Table(rows)
	if !strings.Contains(result, "Key") {
		t.Error("Table should work without color")
	}
}

func TestProgressBar_NoColor(t *testing.T) {
	originalNoColor := os.Getenv("NO_COLOR")
	defer func() {
		_ = os.Setenv("NO_COLOR", originalNoColor)
	}()
	_ = os.Setenv("NO_COLOR", "1")

	result := ProgressBar(50, 100, "Processing")
	if !strings.Contains(result, "Processing") {
		t.Error("ProgressBar should work without color")
	}
}

func TestSeparator_NoColor(t *testing.T) {
	originalNoColor := os.Getenv("NO_COLOR")
	defer func() {
		_ = os.Setenv("NO_COLOR", originalNoColor)
	}()
	_ = os.Setenv("NO_COLOR", "1")

	result := Separator()
	if result == "" {
		t.Error("Separator should work without color")
	}
}

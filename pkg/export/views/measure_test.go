package views

import (
	"testing"
)

func TestDefaultFontMetrics(t *testing.T) {
	metrics := DefaultFontMetrics()

	if metrics.FontSize != 12.0 {
		t.Errorf("Expected FontSize 12.0, got %v", metrics.FontSize)
	}
	if metrics.FontWeight != "normal" {
		t.Errorf("Expected FontWeight 'normal', got %v", metrics.FontWeight)
	}
	if metrics.FontFamily != "Arial" {
		t.Errorf("Expected FontFamily 'Arial', got %v", metrics.FontFamily)
	}
	if metrics.LineHeight != 1.2 {
		t.Errorf("Expected LineHeight 1.2, got %v", metrics.LineHeight)
	}
}

func TestTitleFontMetrics(t *testing.T) {
	metrics := TitleFontMetrics()

	if metrics.FontSize != 14.0 {
		t.Errorf("Expected FontSize 14.0, got %v", metrics.FontSize)
	}
	if metrics.FontWeight != "bold" {
		t.Errorf("Expected FontWeight 'bold', got %v", metrics.FontWeight)
	}
	if metrics.FontFamily != "Arial" {
		t.Errorf("Expected FontFamily 'Arial', got %v", metrics.FontFamily)
	}
	if metrics.LineHeight != 1.2 {
		t.Errorf("Expected LineHeight 1.2, got %v", metrics.LineHeight)
	}
}

func TestMeasureText_Empty(t *testing.T) {
	metrics := DefaultFontMetrics()
	width := MeasureText("", metrics)

	if width != 0 {
		t.Errorf("Expected width 0 for empty text, got %v", width)
	}
}

func TestMeasureText_NarrowChars(t *testing.T) {
	metrics := DefaultFontMetrics()
	width := MeasureText("ilI1|", metrics)

	if width <= 0 {
		t.Errorf("Expected positive width for narrow chars, got %v", width)
	}
}

func TestMeasureText_WideChars(t *testing.T) {
	metrics := DefaultFontMetrics()
	width := MeasureText("mwMW", metrics)

	if width <= 0 {
		t.Errorf("Expected positive width for wide chars, got %v", width)
	}

	narrowWidth := MeasureText("il", metrics)
	if width <= narrowWidth {
		t.Errorf("Expected wide chars to have larger width than narrow chars")
	}
}

func TestMeasureText_WithSpaces(t *testing.T) {
	metrics := DefaultFontMetrics()
	width := MeasureText("hello world", metrics)

	if width <= 0 {
		t.Errorf("Expected positive width for text with spaces, got %v", width)
	}
}

func TestMeasureText_DifferentFontSize(t *testing.T) {
	defaultMetrics := DefaultFontMetrics()
	titleMetrics := TitleFontMetrics()

	text := "hello"
	defaultWidth := MeasureText(text, defaultMetrics)
	titleWidth := MeasureText(text, titleMetrics)

	if titleWidth <= defaultWidth {
		t.Errorf("Expected title font size to have larger width than default")
	}
}

func TestWrapText_Empty(t *testing.T) {
	metrics := DefaultFontMetrics()
	lines, width, height := WrapText("", 100, metrics)

	if len(lines) != 0 {
		t.Errorf("Expected 0 lines for empty text, got %d", len(lines))
	}
	if width != 0 {
		t.Errorf("Expected 0 width for empty text, got %v", width)
	}
	if height != 0 {
		t.Errorf("Expected 0 height for empty text, got %v", height)
	}
}

func TestWrapText_ShortText(t *testing.T) {
	metrics := DefaultFontMetrics()
	text := "hello"
	lines, width, height := WrapText(text, 100, metrics)

	if len(lines) != 1 {
		t.Errorf("Expected 1 line for short text, got %d", len(lines))
	}
	if len(lines) == 0 || lines[0] != text {
		t.Errorf("Expected line '%s', got %v", text, lines)
	}
	if width <= 0 {
		t.Errorf("Expected positive width, got %v", width)
	}
	if height <= 0 {
		t.Errorf("Expected positive height, got %v", height)
	}
}

func TestWrapText_LongText(t *testing.T) {
	metrics := DefaultFontMetrics()
	text := "This is a very long text that should be wrapped into multiple lines because it exceeds the maximum width"
	maxWidth := 100.0

	lines, width, height := WrapText(text, maxWidth, metrics)

	if len(lines) < 2 {
		t.Errorf("Expected at least 2 lines for long text, got %d", len(lines))
	}
	if width <= 0 {
		t.Errorf("Expected positive width, got %v", width)
	}
	if height <= 0 {
		t.Errorf("Expected positive height, got %v", height)
	}
	if width > maxWidth*1.5 {
		t.Errorf("Expected width to be close to maxWidth, got %v vs maxWidth %v", width, maxWidth)
	}
}

func TestWrapText_SingleLongWord(t *testing.T) {
	metrics := DefaultFontMetrics()
	text := "supercalifragilisticexpialidocious"
	maxWidth := 50.0

	lines, width, _ := WrapText(text, maxWidth, metrics)

	if len(lines) != 1 {
		t.Errorf("Expected 1 line for single long word, got %d", len(lines))
	}
	if len(lines) == 0 || lines[0] != text {
		t.Errorf("Expected line to contain the full word")
	}
	if width <= maxWidth {
		t.Errorf("Expected width to exceed maxWidth for single long word, got %v", width)
	}
}

func TestWrapText_MultipleSpaces(t *testing.T) {
	metrics := DefaultFontMetrics()
	text := "hello    world"
	maxWidth := 100.0

	lines, _, _ := WrapText(text, maxWidth, metrics)

	if len(lines) != 1 {
		t.Errorf("Expected 1 line for text with multiple spaces, got %d", len(lines))
	}
}

func TestMeasureNodeContent_Person(t *testing.T) {
	elem := &Element{
		Kind:        "person",
		Title:       "End User",
		Technology:  "Web",
		Description: "A person who uses the system",
	}

	width, height := MeasureNodeContent(elem)

	if width < 200 {
		t.Errorf("Expected width >= 200 for person, got %v", width)
	}
	if height < 180 {
		t.Errorf("Expected height >= 180 for person, got %v", height)
	}
}

func TestMeasureNodeContent_System(t *testing.T) {
	elem := &Element{
		Kind:        "system",
		Title:       "My System",
		Technology:  "Cloud",
		Description: "Main system",
	}

	width, height := MeasureNodeContent(elem)

	if width < 220 {
		t.Errorf("Expected width >= 220 for system, got %v", width)
	}
	if height < 140 {
		t.Errorf("Expected height >= 140 for system, got %v", height)
	}
}

func TestMeasureNodeContent_Component(t *testing.T) {
	elem := &Element{
		Kind:        "component",
		Title:       "API Service",
		Technology:  "Go",
		Description: "REST API",
	}

	width, height := MeasureNodeContent(elem)

	if width < 180 {
		t.Errorf("Expected width >= 180 for component, got %v", width)
	}
	if height < 100 {
		t.Errorf("Expected height >= 100 for component, got %v", height)
	}
}

func TestMeasureNodeContent_DataStore(t *testing.T) {
	elem := &Element{
		Kind:        "datastore",
		Title:       "Database",
		Technology:  "PostgreSQL",
		Description: "Main DB",
	}

	width, height := MeasureNodeContent(elem)

	if width < 200 {
		t.Errorf("Expected width >= 200 for datastore, got %v", width)
	}
	if height < 100 {
		t.Errorf("Expected height >= 100 for datastore, got %v", height)
	}
}

func TestMeasureNodeContent_Queue(t *testing.T) {
	elem := &Element{
		Kind:        "queue",
		Title:       "Event Queue",
		Technology:  "RabbitMQ",
		Description: "Message queue",
	}

	width, height := MeasureNodeContent(elem)

	if width < 200 {
		t.Errorf("Expected width >= 200 for queue, got %v", width)
	}
	if height < 100 {
		t.Errorf("Expected height >= 100 for queue, got %v", height)
	}
}

func TestMeasureNodeContent_MinimumBounds(t *testing.T) {
	elem := &Element{
		Kind:        "component",
		Title:       "X",
		Technology:  "",
		Description: "",
	}

	width, height := MeasureNodeContent(elem)

	if width < 180 {
		t.Errorf("Expected minimum width to be enforced, got %v", width)
	}
	if height < 100 {
		t.Errorf("Expected minimum height to be enforced, got %v", height)
	}
}

func TestMeasureNodeContent_MaximumBounds(t *testing.T) {
	longDesc := "This is a very long description that would normally cause the node to exceed the maximum height limit"
	elem := &Element{
		Kind:        "component",
		Title:       longDesc,
		Technology:  longDesc,
		Description: longDesc,
	}

	width, height := MeasureNodeContent(elem)

	if width > 500 {
		t.Errorf("Expected width <= 500, got %v", width)
	}
	if height > 300 {
		t.Errorf("Expected height <= 300, got %v", height)
	}
}

func TestMeasureNodeContent_NoTechnology(t *testing.T) {
	elem := &Element{
		Kind:        "component",
		Title:       "Service",
		Technology:  "",
		Description: "No technology specified",
	}

	width, height := MeasureNodeContent(elem)

	if width <= 0 || height <= 0 {
		t.Errorf("Expected positive width and height, got %v, %v", width, height)
	}
}

func TestMeasureNodeContent_NoDescription(t *testing.T) {
	elem := &Element{
		Kind:        "component",
		Title:       "Service",
		Technology:  "Go",
		Description: "",
	}

	width, height := MeasureNodeContent(elem)

	if width <= 0 || height <= 0 {
		t.Errorf("Expected positive width and height, got %v, %v", width, height)
	}
}

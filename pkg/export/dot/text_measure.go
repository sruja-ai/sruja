// Package dot provides text measurement utilities for accurate node sizing.
//
// This implements FAANG-level text measurement using font metrics.

package dot

import (
	"math"
	"strings"
)

// FontMetrics holds font measurement configuration.
type FontMetrics struct {
	FontSize   float64 // Font size in points
	FontWeight string  // "normal", "bold"
	FontFamily string  // "Arial", etc.
	LineHeight float64 // Line height multiplier (typically 1.2)
}

// DefaultFontMetrics returns default font metrics for node labels.
func DefaultFontMetrics() FontMetrics {
	return FontMetrics{
		FontSize:   12.0,
		FontWeight: "normal",
		FontFamily: "Arial",
		LineHeight: 1.2,
	}
}

// TitleFontMetrics returns font metrics for node titles (bold, larger).
func TitleFontMetrics() FontMetrics {
	return FontMetrics{
		FontSize:   14.0,
		FontWeight: "bold",
		FontFamily: "Arial",
		LineHeight: 1.2,
	}
}

// MeasureText measures text width using font metrics.
// This is a simplified implementation using character-based estimation.
// For production, consider using a proper font rendering library.
func MeasureText(text string, metrics FontMetrics) float64 {
	if text == "" {
		return 0
	}

	// Character width estimation based on Arial metrics
	// Arial 12pt: average char width ~7.2px, bold ~8.0px
	// Arial 14pt: average char width ~8.4px, bold ~9.6px
	charWidth := 7.2
	if metrics.FontSize == 14.0 {
		charWidth = 8.4
		if metrics.FontWeight == "bold" {
			charWidth = 9.6
		}
	} else if metrics.FontWeight == "bold" {
		charWidth = 8.0
	}

	// Scale by font size
	charWidth *= metrics.FontSize / 12.0

	// Measure actual text (account for variable character widths)
	width := 0.0
	for _, r := range text {
		// Adjust for character width variations
		switch r {
		case 'i', 'l', 'I', '1', '|':
			width += charWidth * 0.4
		case 'm', 'w', 'M', 'W':
			width += charWidth * 1.3
		case ' ', '\t':
			width += charWidth * 0.5
		default:
			width += charWidth
		}
	}

	return width
}

// WrapText wraps text to fit within maxWidth, returning lines and dimensions.
func WrapText(text string, maxWidth float64, metrics FontMetrics) (lines []string, width, height float64) {
	if text == "" {
		return []string{}, 0, 0
	}

	words := strings.Fields(text)
	if len(words) == 0 {
		return []string{}, 0, 0
	}

	var currentLine strings.Builder
	currentLine.WriteString(words[0])
	maxLineWidth := MeasureText(words[0], metrics)

	for i := 1; i < len(words); i++ {
		word := words[i]
		testLine := currentLine.String() + " " + word
		testWidth := MeasureText(testLine, metrics)

		if testWidth > maxWidth && currentLine.Len() > 0 {
			// Current line is full, start new line
			lines = append(lines, currentLine.String())
			if testWidth > maxLineWidth {
				maxLineWidth = testWidth
			}
			currentLine.Reset()
			currentLine.WriteString(word)
		} else {
			// Add word to current line
			if currentLine.Len() > 0 {
				currentLine.WriteString(" ")
			}
			currentLine.WriteString(word)
			if testWidth > maxLineWidth {
				maxLineWidth = testWidth
			}
		}
	}

	// Add last line
	if currentLine.Len() > 0 {
		lines = append(lines, currentLine.String())
	}

	// Calculate dimensions
	width = maxLineWidth
	lineHeight := metrics.FontSize * metrics.LineHeight
	height = float64(len(lines)) * lineHeight

	return lines, width, height
}

// MeasureNodeContent measures all content in a node (title, technology, description).
func MeasureNodeContent(elem *Element) (width, height float64) {
	// Measure title (bold, 14pt)
	titleMetrics := TitleFontMetrics()
	titleWidth := MeasureText(elem.Title, titleMetrics)
	titleHeight := titleMetrics.FontSize * titleMetrics.LineHeight

	// Measure technology tag (normal, 12pt)
	var techWidth float64
	var techHeight float64
	if elem.Technology != "" {
		techText := "[" + elem.Technology + "]"
		techMetrics := DefaultFontMetrics()
		techWidth = MeasureText(techText, techMetrics)
		techHeight = techMetrics.FontSize * techMetrics.LineHeight
	}

	// Measure description (wrapped, 12pt)
	var descWidth, descHeight float64
	if elem.Description != "" {
		// Wrap description to reasonable width (max 300px)
		descMetrics := DefaultFontMetrics()
		_, descWidth, descHeight = WrapText(elem.Description, 300, descMetrics)
	}

	// Calculate total width (max of title, tech, desc)
	width = math.Max(titleWidth, math.Max(techWidth, descWidth))

	// Add padding (20px horizontal, 15px vertical)
	width += 40                                         // 20px each side
	height = titleHeight + techHeight + descHeight + 30 // 15px top + 15px bottom

	// Apply min/max constraints
	minWidth, maxWidth := 180.0, 500.0
	minHeight, maxHeight := 100.0, 300.0

	// Adjust based on kind
	switch elem.Kind {
	case "person":
		minWidth, minHeight = 200.0, 180.0
	case "system":
		minWidth, minHeight = 220.0, 140.0
	case "container":
		minWidth, minHeight = 200.0, 120.0
	case "component":
		minWidth, minHeight = 180.0, 100.0
	case "datastore", "queue":
		minWidth, minHeight = 200.0, 100.0
	}

	if width < minWidth {
		width = minWidth
	}
	if width > maxWidth {
		width = maxWidth
	}
	if height < minHeight {
		height = minHeight
	}
	if height > maxHeight {
		height = maxHeight
	}

	return width, height
}

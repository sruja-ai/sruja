// pkg/dx/cli_format.go
package dx

import (
	"fmt"
	"os"
	"strings"
)

// Colors for terminal output
const (
	ColorReset  = "\033[0m"
	ColorRed    = "\033[31m"
	ColorGreen  = "\033[32m"
	ColorYellow = "\033[33m"
	ColorBlue   = "\033[34m"
	ColorPurple = "\033[35m"
	ColorCyan   = "\033[36m"
	ColorGray   = "\033[90m"
	ColorBold   = "\033[1m"
)

// SupportsColor checks if the terminal supports color output.
func SupportsColor() bool {
	// Check if output is a terminal and not being piped
	if fileInfo, err := os.Stdout.Stat(); err == nil {
		mode := fileInfo.Mode()
		// Check if it's a character device (terminal)
		if mode&os.ModeCharDevice != 0 {
			// Check TERM environment variable
			term := os.Getenv("TERM")
			if term != "" && term != "dumb" {
				// Check NO_COLOR environment variable (respect user preference)
				if os.Getenv("NO_COLOR") == "" {
					return true
				}
			}
		}
	}
	return false
}

// Colorize applies color to text if color support is enabled.
func Colorize(color, text string, enabled bool) string {
	if enabled {
		return color + text + ColorReset
	}
	return text
}

// Success formats a success message.
func Success(message string) string {
	useColor := SupportsColor()
	return Colorize(ColorGreen, "✓", useColor) + " " + message
}

// Error formats an error message.
func Error(message string) string {
	useColor := SupportsColor()
	return Colorize(ColorRed, "✗", useColor) + " " + message
}

// Warning formats a warning message.
func Warning(message string) string {
	useColor := SupportsColor()
	return Colorize(ColorYellow, "⚠", useColor) + " " + message
}

// Info formats an info message.
func Info(message string) string {
	useColor := SupportsColor()
	return Colorize(ColorBlue, "ℹ", useColor) + " " + message
}

// Bold makes text bold.
func Bold(text string) string {
	useColor := SupportsColor()
	return Colorize(ColorBold, text, useColor)
}

// Dim makes text dim/gray.
func Dim(text string) string {
	useColor := SupportsColor()
	return Colorize(ColorGray, text, useColor)
}

// Header formats a section header.
func Header(text string) string {
	useColor := SupportsColor()
	line := strings.Repeat("=", len(text))
	if useColor {
		return fmt.Sprintf("\n%s\n%s\n%s\n", line, Colorize(ColorBold+ColorCyan, text, useColor), line)
	}
	return fmt.Sprintf("\n%s\n%s\n%s\n", line, text, line)
}

// Section formats a section with a title.
func Section(title string) string {
	useColor := SupportsColor()
	if useColor {
		return fmt.Sprintf("\n%s %s\n", Colorize(ColorCyan+ColorBold, "→", useColor), Bold(title))
	}
	return fmt.Sprintf("\n→ %s\n", title)
}

// ListItem formats a list item.
func ListItem(text string, indent int) string {
	useColor := SupportsColor()
	indentStr := strings.Repeat("  ", indent)
	bullet := "•"
	if useColor {
		bullet = Colorize(ColorCyan, bullet, useColor)
	}
	return fmt.Sprintf("%s%s %s", indentStr, bullet, text)
}

// Code formats code/text in a monospace style.
func Code(text string) string {
	useColor := SupportsColor()
	if useColor {
		return Colorize(ColorYellow, text, useColor)
	}
	return text
}

// Link formats a link-style text.
func Link(text string) string {
	useColor := SupportsColor()
	if useColor {
		return Colorize(ColorBlue+ColorBold, text, useColor)
	}
	return text
}

// Table formats a simple two-column table.
func Table(rows [][]string) string {
	if len(rows) == 0 {
		return ""
	}

	useColor := SupportsColor()
	var sb strings.Builder

	// Find max width for first column
	maxWidth := 0
	for _, row := range rows {
		if len(row) > 0 && len(row[0]) > maxWidth {
			maxWidth = len(row[0])
		}
	}

	// Print rows
	for _, row := range rows {
		if len(row) == 0 {
			continue
		}
		first := row[0]
		if len(row) > 1 {
			second := row[1]
			padding := strings.Repeat(" ", maxWidth-len(first))

			if useColor {
				sb.WriteString(fmt.Sprintf("%s%s %s%s\n",
					Colorize(ColorCyan, first, useColor),
					padding,
					Dim("│"),
					second))
			} else {
				sb.WriteString(fmt.Sprintf("%s%s │ %s\n", first, padding, second))
			}
		} else {
			sb.WriteString(first + "\n")
		}
	}

	return sb.String()
}

// ProgressBar creates a simple progress indicator (not a real animated bar).
func ProgressBar(current, total int, label string) string {
	if total == 0 {
		return ""
	}

	useColor := SupportsColor()
	percentage := int((float64(current) / float64(total)) * 100)

	if useColor {
		return fmt.Sprintf("%s [%d%%] %s", Colorize(ColorCyan, "⏳", useColor), percentage, label)
	}
	return fmt.Sprintf("[%d%%] %s", percentage, label)
}

// Separator prints a visual separator line.
func Separator() string {
	useColor := SupportsColor()
	line := strings.Repeat("─", 60)
	if useColor {
		return Colorize(ColorGray, line, useColor)
	}
	return line
}

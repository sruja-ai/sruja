// pkg/export/svg/templates.go
package svg

import (
	"bytes"
	"embed"
	"fmt"
	"html"
	"text/template"
)

//go:embed templates/*.tmpl
var templateFS embed.FS

// TemplateData holds all data needed for SVG template rendering
type TemplateData struct {
	Width      int
	Height     int
	Background string
	Theme      *Theme
	Metadata   map[string]string
	EmbedFonts bool

	// Grid
	ShowGrid  bool
	GridColor string
	GridSize  int

	// Title
	ShowTitle bool
	Title     string
	TitleX    int
	TitleY    int

	// Definitions
	Defs string

	// Content
	GridLines []GridLine
	Edges     []Edge
	Nodes     []Node
	Groups    []Group
	Legend    Legend
}

type GridLine struct {
	X1, Y1, X2, Y2 int
	Color          string
	Opacity        float64
}

type Edge struct {
	X1, Y1, X2, Y2     int
	Curved             bool
	CX1, CY1, CX2, CY2 int
	Color              string
	Width              int
	Label              string
	LabelX, LabelY     int
	LabelBg            bool
	Title              string
	Description        string
}

type Node struct {
	X, Y            int
	Width, Height   int
	Fill            string
	Stroke          string
	StrokeWidth     float64
	Gradient        string
	Filter          string
	Dash            string
	Icon            string
	IconX, IconY    int
	IconScale       float64
	IconW, IconH    int
	IconStrokeWidth float64
	Label           string
	LabelX, LabelY  int
	LabelColor      string
	LabelSize       int
	WrappedLines    []string
	LineHeight      int
	Title           string
	Description     string
}

type Group struct {
	X, Y          int
	Width, Height int
	Title         string
	Stroke        string
	Fill          string
	TitleColor    string
}

type Legend struct {
	Show  bool
	X, Y  int
	Items []LegendItem
	Title string
}

type LegendItem struct {
	Icon   string
	Label  string
	Stroke string
	Fill   string
}

// TemplateFuncMap provides helper functions for templates
func templateFuncMap() template.FuncMap {
	return template.FuncMap{
		"add": func(a, b int) int { return a + b },
		"sub": func(a, b int) int { return a - b },
		"mul": func(a, b int) int { return a * b },
		"div": func(a, b int) int {
			if b == 0 {
				return 0
			}
			return a / b
		},
		"fdiv": func(a, b int) float64 {
			if b == 0 {
				return 0
			}
			return float64(a) / float64(b)
		},
		"lighten": lightenColor,
		"xml":     escapeXML, // Escape XML special characters for user content
	}
}

// loadTemplate loads and parses a template from the embedded filesystem
func loadTemplate(name string) (*template.Template, error) {
	tmplContent, err := templateFS.ReadFile("templates/" + name + ".tmpl")
	if err != nil {
		return nil, fmt.Errorf("failed to read template %s: %w", name, err)
	}

	tmpl, err := template.New(name).Funcs(templateFuncMap()).Parse(string(tmplContent))
	if err != nil {
		return nil, fmt.Errorf("failed to parse template %s: %w", name, err)
	}

	return tmpl, nil
}

// escapeXML escapes XML special characters (but not HTML entities)
func escapeXML(s string) string {
	return html.EscapeString(s)
}

// renderSVG renders the SVG using templates
func renderSVG(data *TemplateData) (string, error) {
	tmpl, err := loadTemplate("svg")
	if err != nil {
		return "", err
	}

	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, data); err != nil {
		return "", fmt.Errorf("failed to execute SVG template: %w", err)
	}

	return buf.String(), nil
}

// renderDefs renders the defs section using templates
func renderDefs(theme *Theme, icons []IconDef) (string, error) {
	type DefsData struct {
		*Theme
		Icons []IconDef
	}

	data := DefsData{
		Theme: theme,
		Icons: icons,
	}

	tmpl, err := loadTemplate("defs")
	if err != nil {
		return "", err
	}

	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, data); err != nil {
		return "", fmt.Errorf("failed to execute defs template: %w", err)
	}

	return buf.String(), nil
}

type IconDef struct {
	Name    string
	Content string
}

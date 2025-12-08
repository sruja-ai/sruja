// pkg/export/html/html.go
// Package html provides HTML export with embedded Sruja Viewer.
package html

import (
	"bytes"
	"embed"
	"encoding/json"
	"fmt"
	htmlpkg "html"
	"html/template"
	"os"
	"path/filepath"
	"strings"

	"github.com/tdewolff/minify/v2"
	"github.com/tdewolff/minify/v2/css"
	minifyhtml "github.com/tdewolff/minify/v2/html"
	"github.com/tdewolff/minify/v2/js"

	jexport "github.com/sruja-ai/sruja/pkg/export/json"
	"github.com/sruja-ai/sruja/pkg/export/svg"
	"github.com/sruja-ai/sruja/pkg/language"
)

//go:embed template_single.html template_svg.html template_svg_styles.html template_svg_sidebar.html template_svg_views.html template_svg_scripts.html template_json_path.html template_v2.html
var templateFS embed.FS

// embedViewerFS contains embedded assets for HTML export:
//   - embed/embed-viewer.js: Full viewer bundle for ModeSingleFile (legacy, ~500KB+)
//   - embed/embed-viewer.css: CSS for the full viewer bundle
//   - embed/components/sruja-components.js: Web components bundle for ModeSVG (lightweight, ~12KB)
//
//go:embed embed/embed-viewer.js embed/embed-viewer.css embed/components/sruja-components.js embed/components/sruja-v2-components.js embed/template_v2_styles.css embed/template_v2_scripts.js embed/vendor/*
var embedViewerFS embed.FS

// ExportMode represents the HTML export mode.
//
// There are two export modes:
//
//   - ModeSVG (recommended): Lightweight mode that embeds SVG diagrams directly.
//     Uses small web components (~12KB) for interactivity. Produces standalone HTML
//     files that don't require external dependencies. This is the default mode.
//
//   - ModeSingleFile: Legacy mode that embeds the full viewer bundle (~500KB+).
//     Includes everything in a single file but is much larger. Maintained for
//     backward compatibility with older exports.
type ExportMode int

const (
	ModeSVG        ExportMode = iota // SVG-based mode (lightweight, no viewer bundle) - recommended
	ModeSingleFile                   // Single-file mode (everything inlined) - for backward compatibility
	ModeV2                           // V2 mode: Pure JS + elkjs + Web Components (experimental)
)

type Exporter struct {
	JSONPath  string     // Path to JSON file (relative or absolute)
	Mode      ExportMode // Export mode (SVG, SingleFile)
	EmbedJSON bool       // Embed JSON data directly in HTML
	OutputDir string     // Output directory for local mode
	Minify    bool       // Whether to minify HTML output (default: false for readability, true for production)
}

type templateData struct {
	ArchName          string
	JSONData          string
	DSLContent        string            // DSL content for DSL tab
	SVGContent        string            // SVG content for "all" view (full architecture)
	SystemSVGs        map[string]string // Map of system ID -> SVG content
	ContainerSVGs     map[string]string // Map of "systemID.containerID" -> SVG content
	ScenarioSVGs      map[string]string // Map of scenario ID -> SVG content
	FlowSVGs          map[string]string // Map of flow ID -> SVG content
	ViewerCoreJS      string            // Inlined JS for single-file mode
	ViewerAppJS       string            // Inlined JS for single-file mode
	ViewerCSS         string            // Inlined CSS for single-file mode
	EmbedViewerJS     string            // Embedded IIFE bundle
	EmbedViewerCSS    string            // Embedded CSS
	ComponentsJS      string            // Embedded web components bundle
	VendorCytoscapeJS string            // Embedded Cytoscape JS
	VendorElkJS       string            // Embedded Elk JS
	VendorCyElkJS     string            // Embedded Cytoscape-Elk JS
}

func NewExporter() *Exporter {
	return &Exporter{
		Mode:      ModeSVG, // Default to SVG mode (lightweight, no viewer bundle)
		EmbedJSON: true,    // Always embed JSON for standalone HTML
		Minify:    false,   // Default to readable output (set to true for production)
	}
}

// ExportFromArchitecture converts an Architecture AST to HTML format.
// This is a convenience method that first exports to JSON, then generates HTML.
//
//nolint:gocyclo // Complexity is acceptable for main export logic
func (e *Exporter) ExportFromArchitecture(arch *language.Architecture) (string, error) {
	// Parse JSON to get architecture name
	jsonExporter := jexport.NewExporter()
	jsonData, err := jsonExporter.Export(arch)
	if err != nil {
		return "", fmt.Errorf("failed to export to JSON: %w", err)
	}

	var archJSON jexport.ArchitectureJSON
	if err := json.Unmarshal([]byte(jsonData), &archJSON); err != nil {
		return "", fmt.Errorf("failed to parse JSON: %w", err)
	}

	// Check for V2 mode
	if e.Mode == ModeV2 {
		return e.generateHTMLWithV2(archJSON.Metadata.Name, jsonData)
	}

	// For SVG mode, generate SVG and use it
	if e.Mode == ModeSVG {
		svgExporter := svg.NewExporter()
		svgExporter.EmbedFonts = true

		// Generate full architecture SVG
		svgContent, err := svgExporter.ExportAll(arch)
		if err != nil {
			return "", fmt.Errorf("failed to export to SVG: %w", err)
		}

		// Generate per-system SVGs
		systemSVGs := make(map[string]string)
		for _, sys := range arch.Systems {
			sysSVG, err := svgExporter.ExportSystemContainer(arch, sys)
			if err != nil {
				// Log error but continue - not all systems may be exportable
				continue
			}
			systemSVGs[sys.ID] = sysSVG
		}

		// Generate per-container SVGs
		containerSVGs := make(map[string]string)
		for _, sys := range arch.Systems {
			for _, cont := range sys.Containers {
				contSVG, err := svgExporter.ExportContainerComponent(arch, sys, cont)
				if err != nil {
					// Log error but continue
					continue
				}
				// Use "systemID.containerID" as key
				containerSVGs[sys.ID+"."+cont.ID] = contSVG
			}
		}

		// Generate per-scenario SVGs
		scenarioSVGs := make(map[string]string)
		for _, scenario := range arch.Scenarios {
			scenarioSVG, err := svgExporter.ExportScenario(arch, scenario)
			if err != nil {
				// Log error but continue
				continue
			}
			scenarioID := scenario.ID
			if scenarioID == "" {
				scenarioID = scenario.Title
			}
			scenarioSVGs[scenarioID] = scenarioSVG
		}

		// Generate per-flow SVGs
		flowSVGs := make(map[string]string)
		for _, flow := range arch.Flows {
			flowSVG, err := svgExporter.ExportFlow(arch, flow)
			if err != nil {
				// Log error but continue
				continue
			}
			flowSVGs[flow.ID] = flowSVG
		}

		// Generate DSL from architecture
		printer := language.NewPrinter()
		dslContent := printer.Print(&language.Program{Architecture: arch})

		return e.generateHTMLWithSVG(archJSON.Metadata.Name, svgContent, jsonData, dslContent, systemSVGs, containerSVGs, scenarioSVGs, flowSVGs)
	}

	// Generate HTML
	if e.EmbedJSON {
		return e.generateHTMLWithEmbeddedJSON(archJSON.Metadata.Name, jsonData)
	}
	return e.generateHTMLWithJSONPath(archJSON.Metadata.Name, e.JSONPath)
}

// ExportFromJSON generates HTML from JSON data.
func (e *Exporter) ExportFromJSON(jsonData []byte) (string, error) {
	// Parse JSON to get architecture name
	var archJSON jexport.ArchitectureJSON
	if err := json.Unmarshal(jsonData, &archJSON); err != nil {
		return "", fmt.Errorf("failed to parse JSON: %w", err)
	}

	// Generate HTML
	if e.EmbedJSON {
		return e.generateHTMLWithEmbeddedJSON(archJSON.Metadata.Name, string(jsonData))
	}
	return e.generateHTMLWithJSONPath(archJSON.Metadata.Name, e.JSONPath)
}

// generateHTMLWithJSONPath generates HTML that references an external JSON file.
func (e *Exporter) generateHTMLWithJSONPath(archName, jsonPath string) (string, error) {
	// Determine JSON file path (relative to HTML file)
	jsonFile := jsonPath
	if jsonFile == "" {
		jsonFile = "architecture.json"
	}
	// Make path relative if it's absolute
	if filepath.IsAbs(jsonFile) {
		jsonFile = filepath.Base(jsonFile)
	}

	// Load template
	tmplContent, err := templateFS.ReadFile("template_json_path.html")
	if err != nil {
		return "", fmt.Errorf("failed to read template_json_path.html: %w", err)
	}

	// Create template with custom functions
	funcMap := template.FuncMap{
		"escapeHTML": htmlpkg.EscapeString,
	}

	tmpl, err := template.New("json_path").Funcs(funcMap).Parse(string(tmplContent))
	if err != nil {
		return "", fmt.Errorf("failed to parse template: %w", err)
	}

	// Prepare template data
	data := struct {
		ArchName string
		JSONPath string
	}{
		ArchName: archName,
		JSONPath: jsonFile,
	}

	// Execute template
	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, data); err != nil {
		return "", fmt.Errorf("failed to execute template: %w", err)
	}

	return buf.String(), nil
}

// parseSVGTemplates parses all SVG mode templates and returns a configured template.
// This helper reduces duplication between generateHTMLWithEmbeddedJSON and generateHTMLWithSVG.
func parseSVGTemplates(funcMap template.FuncMap) (*template.Template, error) {
	// Load main template
	mainTmpl, err := templateFS.ReadFile("template_svg.html")
	if err != nil {
		return nil, fmt.Errorf("failed to read template_svg.html: %w", err)
	}

	// Parse main template first
	tmpl, err := template.New("main").Funcs(funcMap).Parse(string(mainTmpl))
	if err != nil {
		return nil, fmt.Errorf("failed to parse main template: %w", err)
	}

	// Parse all split templates
	templateFiles := map[string]string{
		"styles":  "template_svg_styles.html",
		"sidebar": "template_svg_sidebar.html",
		"views":   "template_svg_views.html",
		"scripts": "template_svg_scripts.html",
	}

	for name, filename := range templateFiles {
		content, err := templateFS.ReadFile(filename)
		if err != nil {
			return nil, fmt.Errorf("failed to read %s: %w", filename, err)
		}
		if _, err = tmpl.New(name).Parse(string(content)); err != nil {
			return nil, fmt.Errorf("failed to parse %s template: %w", name, err)
		}
	}

	return tmpl, nil
}

// getSVGTemplateFuncMap returns the function map for SVG mode templates.
func getSVGTemplateFuncMap() template.FuncMap {
	return template.FuncMap{
		"safeHTML": func(s string) template.HTML {
			// For SVG content, we trust it since it's generated by our own exporter
			return template.HTML(s) // #nosec G203 // trust generated SVG content
		},
		"safeJS": func(s string) template.JS {
			// Escape </script> to prevent XSS
			safe := strings.ReplaceAll(s, "</script>", "<\\/script>")
			return template.JS(safe) // #nosec G203 // safe substitution
		},
		"escapeHTML": htmlpkg.EscapeString,
		"rawDSL": func(s string) template.HTML {
			// For DSL content, return as-is without escaping (it's in a <pre> tag)
			return template.HTML(s) // #nosec G203 // trust content in pre tag
		},
	}
}

// generateHTMLWithEmbeddedJSON generates HTML with JSON data embedded inline.
// This function handles both ModeSVG (lightweight) and ModeSingleFile (legacy) modes.
// For ModeSVG, it uses the split template structure with web components.
// For ModeSingleFile, it embeds the full viewer bundle.
//
//nolint:funlen,gocyclo,gosec // HTML generation complexity; G306 handled in file write
func (e *Exporter) generateHTMLWithEmbeddedJSON(archName, jsonData string) (string, error) {
	// Select template based on mode
	var templateName string
	switch e.Mode {
	case ModeSingleFile:
		templateName = "template_single.html"
	case ModeSVG:
		templateName = "template_svg.html"
	default:
		templateName = "template_svg.html" // Default to lightweight SVG mode
	}

	// Load template
	tmplContent, err := templateFS.ReadFile(templateName)
	if err != nil {
		return "", fmt.Errorf("failed to read template %s: %w", templateName, err)
	}

	// Create template with custom functions
	funcMap := template.FuncMap{
		"safeJS": func(s string) template.JS {
			// Escape </script> to prevent XSS
			safe := strings.ReplaceAll(s, "</script>", "<\\/script>")
			return template.JS(safe) // #nosec G203 // safe substitution
		},
		"safeCSS": func(s string) template.CSS {
			return template.CSS(s) // #nosec G203 // trust generated CSS
		},
		"safeHTML": func(s string) template.HTML {
			// For SVG content, we trust it since it's generated by our own exporter
			return template.HTML(s) // #nosec G203 // trust generated SVG content
		},
	}
	// Add escapeHTML and rawDSL for SVG mode (needed by views template)
	if e.Mode == ModeSVG {
		funcMap["escapeHTML"] = htmlpkg.EscapeString
		funcMap["rawDSL"] = func(s string) template.HTML {
			// For DSL content, return as-is without escaping (it's in a <pre> tag)
			return template.HTML(s) // #nosec G203 // trust content in pre tag
		}
	}

	// Parse template - for SVG mode, use helper function; for single-file mode, parse directly
	var tmpl *template.Template
	if e.Mode == ModeSVG {
		// Use helper function to parse all SVG templates
		tmpl, err = parseSVGTemplates(funcMap)
		if err != nil {
			return "", err
		}
	} else {
		// For single-file mode, use simple template parsing
		tmpl, err = template.New("html").Funcs(funcMap).Parse(string(tmplContent))
		if err != nil {
			return "", fmt.Errorf("failed to parse template: %w", err)
		}
	}

	// Prepare template data
	data := templateData{
		ArchName:      archName,
		JSONData:      jsonData, // Will be escaped by safeJS function
		DSLContent:    "",       // Empty for JSON-only export (no DSL available)
		SVGContent:    "",       // Empty for JSON-only export (no SVG available)
		SystemSVGs:    make(map[string]string),
		ContainerSVGs: make(map[string]string),
		ScenarioSVGs:  make(map[string]string),
		FlowSVGs:      make(map[string]string),
	}

	// For single-file mode, use embedded IIFE bundle
	if e.Mode == ModeSingleFile {
		// Read embedded IIFE bundle
		embedJS, err := embedViewerFS.ReadFile("embed/embed-viewer.js")
		if err != nil {
			return "", fmt.Errorf("failed to read embedded viewer JS: %w (run 'make build-embed-viewer' first)", err)
		}
		embedJSStr := strings.TrimSpace(string(embedJS))
		// Check if file is just a placeholder (contains build instruction comment)
		if embedJSStr == "" || strings.Contains(embedJSStr, "This file is replaced during build") {
			return "", fmt.Errorf("embedded viewer JS is not built (placeholder file found). Run 'make build-embed-viewer' to build the IIFE bundle")
		}
		data.EmbedViewerJS = embedJSStr

		// Read embedded CSS
		embedCSS, err := embedViewerFS.ReadFile("embed/embed-viewer.css")
		if err == nil {
			embedCSSStr := strings.TrimSpace(string(embedCSS))
			// Only use CSS if it's not a placeholder
			if embedCSSStr != "" && !strings.Contains(embedCSSStr, "This file is replaced during build") {
				data.EmbedViewerCSS = embedCSSStr
			}
		}
	}

	// Execute template
	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, data); err != nil {
		return "", fmt.Errorf("failed to execute template: %w", err)
	}

	return buf.String(), nil
}

// generateHTMLWithSVG generates HTML with embedded SVG content (lightweight mode).
// This is the recommended mode: it embeds SVG diagrams directly and uses lightweight
// web components (~12KB) instead of the full viewer bundle (~500KB+).
// generateHTMLWithSVG generates HTML with embedded SVG content (lightweight mode).
// This is the recommended mode: it embeds SVG diagrams directly and uses lightweight
// web components (~12KB) instead of the full viewer bundle (~500KB+).
func (e *Exporter) generateHTMLWithSVG(archName, svgContent, jsonData, dslContent string, systemSVGs, containerSVGs, scenarioSVGs, flowSVGs map[string]string) (string, error) {
	// Use helper function to parse all SVG templates
	funcMap := getSVGTemplateFuncMap()
	tmpl, err := parseSVGTemplates(funcMap)
	if err != nil {
		return "", err
	}

	// Load web components bundle (optional - may not exist if build-html-viewer hasn't run)
	componentsJS, err := embedViewerFS.ReadFile("embed/components/sruja-components.js")
	componentsJSStr := ""
	if err == nil {
		componentsJSStr = strings.TrimSpace(string(componentsJS))
	}

	// Prepare template data
	data := templateData{
		ArchName:      archName,
		SVGContent:    svgContent,
		SystemSVGs:    systemSVGs,
		ContainerSVGs: containerSVGs,
		ScenarioSVGs:  scenarioSVGs,
		FlowSVGs:      flowSVGs,
		JSONData:      jsonData,   // Embed JSON for interactive features (search, filter, tabs)
		DSLContent:    dslContent, // Embed DSL for DSL tab
		ComponentsJS:  componentsJSStr,
	}

	// Execute template
	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, data); err != nil {
		return "", fmt.Errorf("failed to execute template: %w", err)
	}

	htmlOutput := buf.String()

	// Minify if enabled
	if e.Minify {
		m := minify.New()
		m.AddFunc("text/html", minifyhtml.Minify)
		m.AddFunc("text/css", css.Minify)
		m.AddFunc("text/javascript", js.Minify)
		m.AddFunc("application/javascript", js.Minify)

		minified, err := m.String("text/html", htmlOutput)
		if err != nil {
			// If minification fails, return original (don't fail the export)
			return htmlOutput, nil
		}
		return minified, nil
	}

	return htmlOutput, nil
}

// readViewerBundle reads a viewer bundle file from the dist directory

// Export writes HTML to a file.
func (e *Exporter) Export(arch *language.Architecture, outputPath string) error {
	html, err := e.ExportFromArchitecture(arch)
	if err != nil {
		return err
	}

	// If JSONPath is not set and we're not embedding, set it based on output path
	if !e.EmbedJSON && e.JSONPath == "" {
		outputDir := filepath.Dir(outputPath)
		outputBase := filepath.Base(outputPath)
		jsonName := strings.TrimSuffix(outputBase, filepath.Ext(outputBase)) + ".json"
		e.JSONPath = filepath.Join(outputDir, jsonName)
	}

	return os.WriteFile(outputPath, []byte(html), 0o644) //nolint:gosec // standard permission for generated file
}

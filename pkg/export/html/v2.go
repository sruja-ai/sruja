package html

import (
	"bytes"
	"fmt"
	"html/template"
	"strings"

	"github.com/tdewolff/minify/v2"
	"github.com/tdewolff/minify/v2/css"
	minifyhtml "github.com/tdewolff/minify/v2/html"
	"github.com/tdewolff/minify/v2/js"
)

// generateHTMLWithV2 generates HTML for the V2 mode (elkjs + pure JS).
func (e *Exporter) generateHTMLWithV2(archName, jsonData string) (string, error) {
	// Load template
	tmplContent, err := templateFS.ReadFile("template_v2.html")
	if err != nil {
		return "", fmt.Errorf("failed to read template_v2.html: %w", err)
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
	}

	tmpl, err := template.New("html_v2").Funcs(funcMap).Parse(string(tmplContent))
	if err != nil {
		return "", fmt.Errorf("failed to parse template: %w", err)
	}

	// Load V2 CSS
	v2CSS, err := embedViewerFS.ReadFile("embed/template_v2_styles.css")
	v2CSSStr := ""
	if err == nil {
		v2CSSStr = strings.TrimSpace(string(v2CSS))
	}

	// Load V2 JS
	v2JS, err := embedViewerFS.ReadFile("embed/template_v2_scripts.js")
	v2JSStr := ""
	if err == nil {
		v2JSStr = strings.TrimSpace(string(v2JS))
	}

	// Load vendor scripts
	readVendor := func(name string) string {
		content, err := embedViewerFS.ReadFile("embed/vendor/" + name)
		if err != nil {
			return "" // Should not happen if correctly embedded
		}
		return string(content)
	}

	// Prepare template data
	data := struct {
		ArchName          string
		JSONData          string
		V2CSS             string
		V2JS              string
		VendorCytoscapeJS string
		VendorDagreJS     string
		VendorCyDagreJS   string
		VendorElkJS       string
		VendorCyElkJS     string
	}{
		ArchName:          archName,
		JSONData:          jsonData,
		V2CSS:             v2CSSStr,
		V2JS:              v2JSStr,
		VendorCytoscapeJS: readVendor("cytoscape.min.js"),
		VendorDagreJS:     readVendor("dagre.min.js"),
		VendorCyDagreJS:   readVendor("cytoscape-dagre.js"),
		VendorElkJS:       readVendor("elk.bundled.js"),
		VendorCyElkJS:     readVendor("cytoscape-elk.min.js"),
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
			return htmlOutput, nil
		}
		return minified, nil
	}

	return htmlOutput, nil
}

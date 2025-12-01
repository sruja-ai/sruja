// pkg/export/svg/ui.go
// UI components (styles, buttons, panels)
package svg

import (
	"fmt"
	"html"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

func (e *Exporter) writeStyles(sb *strings.Builder) {
	sb.WriteString("    <!-- Styles -->\n")
	sb.WriteString("    <style type=\"text/css\">\n")
	sb.WriteString("        text { font-family: 'Segoe UI', 'Arial', sans-serif; fill: #2c3e50; }\n")
	sb.WriteString("        .title { font-size: 42px; font-weight: bold; fill: #2c3e50; }\n")
	sb.WriteString("        .subtitle { font-size: 20px; fill: #7f8c8d; }\n")
	sb.WriteString("        .section-header { font-size: 26px; font-weight: bold; fill: #34495e; text-decoration: underline; }\n")
	sb.WriteString("        .doc-text { font-size: 16px; line-height: 1.4; }\n")
	sb.WriteString("        .doc-list-item { font-size: 15px; line-height: 1.5; }\n")
	sb.WriteString("        .system-boundary { fill: none; stroke: #95a5a6; stroke-width: 3; stroke-dasharray: 10 5; }\n")
	sb.WriteString("        .container-boundary { fill: none; stroke: #bdc3c7; stroke-width: 2; stroke-dasharray: 5 3; }\n")
	sb.WriteString("        .system-box { fill: #3498db; stroke: #2980b9; stroke-width: 4; cursor: pointer; }\n")
	sb.WriteString("        .container-box { fill: #f1c40f; stroke: #f39c12; stroke-width: 3; cursor: pointer; }\n")
	sb.WriteString("        .component-box { fill: #9b59b6; stroke: #8e44ad; stroke-width: 2; cursor: pointer; }\n")
	sb.WriteString("        .person-box { fill: #2ecc71; stroke: #27ae60; stroke-width: 4; cursor: pointer; }\n")
	sb.WriteString("        .db-box { fill: #e74c3c; stroke: #c0392b; stroke-width: 3; cursor: pointer; }\n")
	sb.WriteString("        .queue-box { fill: #e67e22; stroke: #d35400; stroke-width: 3; cursor: pointer; }\n")
	sb.WriteString("        .external-box { fill: #95a5a6; stroke: #7f8c8d; stroke-width: 3; cursor: pointer; }\n")
	sb.WriteString("        .interactive:hover { opacity: 0.85; transform: scale(1.02); }\n")
	sb.WriteString("        .selected { stroke-width: 6 !important; opacity: 0.95; filter: drop-shadow(0 0 8px rgba(52, 152, 219, 0.6)); }\n")
	sb.WriteString("        .button { fill: #ecf0f1; stroke: #bdc3c7; stroke-width: 2; cursor: pointer; rx: 5; }\n")
	sb.WriteString("        .button:hover { fill: #dfe6e9; stroke: #3498db; stroke-width: 2.5; }\n")
	sb.WriteString("        .button-active { fill: #3498db; stroke: #2980b9; }\n")
	sb.WriteString("        .button-text { font-size: 16px; font-weight: bold; text-anchor: middle; alignment-baseline: central; fill: #2c3e50; pointer-events: none; }\n")
	sb.WriteString("        .button-text-active { fill: #ffffff; }\n")
	sb.WriteString("        #docPanel { fill: #ffffff; stroke: #bdc3c7; stroke-width: 2; rx: 8; }\n")
	sb.WriteString("        .filter-button { fill: #ecf0f1; stroke: #bdc3c7; stroke-width: 1.5; cursor: pointer; rx: 3; }\n")
	sb.WriteString("        .filter-button:hover { fill: #d5dbdb; }\n")
	sb.WriteString("        .filter-button-active { fill: #3498db; stroke: #2980b9; }\n")
	sb.WriteString("        .filter-text { font-size: 13px; text-anchor: middle; alignment-baseline: central; fill: #2c3e50; pointer-events: none; }\n")
	sb.WriteString("        .filter-text-active { fill: #ffffff; }\n")
	sb.WriteString("    </style>\n\n")
}

func (e *Exporter) writeHeader(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("    <!-- Header -->\n")
	sb.WriteString("    <rect x=\"0\" y=\"0\" width=\"2400\" height=\"100\" fill=\"#ecf0f1\" stroke=\"#bdc3c7\" stroke-width=\"2\"/>\n")
	fmt.Fprintf(sb, "    <text x=\"1200\" y=\"40\" class=\"title\" text-anchor=\"middle\">%s Architecture</text>\n", html.EscapeString(arch.Name))
	sb.WriteString("    <text x=\"1200\" y=\"70\" class=\"subtitle\" text-anchor=\"middle\">Interactive C4 Model with Requirements, ADRs &amp; Technology Choices</text>\n\n")
}

func (e *Exporter) writeLevelButtons(sb *strings.Builder) {
	sb.WriteString("    <!-- C4 Level Navigation -->\n")
	sb.WriteString("    <g id=\"levelButtons\">\n")
	sb.WriteString("        <rect id=\"btnLevel1\" x=\"50\" y=\"120\" width=\"180\" height=\"40\" class=\"button button-active\" data-level=\"1\"/>\n")
	sb.WriteString("        <text x=\"140\" y=\"145\" class=\"button-text button-text-active\">Level 1: System Context</text>\n")
	sb.WriteString("        <rect id=\"btnLevel2\" x=\"250\" y=\"120\" width=\"180\" height=\"40\" class=\"button\" data-level=\"2\"/>\n")
	sb.WriteString("        <text x=\"340\" y=\"145\" class=\"button-text\">Level 2: Containers</text>\n")
	sb.WriteString("        <rect id=\"btnLevel3\" x=\"450\" y=\"120\" width=\"180\" height=\"40\" class=\"button\" data-level=\"3\"/>\n")
	sb.WriteString("        <text x=\"540\" y=\"145\" class=\"button-text\">Level 3: Components</text>\n")
	sb.WriteString("    </g>\n\n")
}

func (e *Exporter) writeFilterButtons(sb *strings.Builder) {
	sb.WriteString("    <!-- Filter Buttons -->\n")
	sb.WriteString("    <g id=\"filterButtons\">\n")
	sb.WriteString("        <rect id=\"btnAll\" x=\"700\" y=\"120\" width=\"100\" height=\"30\" class=\"filter-button filter-button-active\" data-filter=\"all\"/>\n")
	sb.WriteString("        <text x=\"750\" y=\"138\" class=\"filter-text filter-text-active\">All</text>\n")
	sb.WriteString("        <rect id=\"btnRequirements\" x=\"820\" y=\"120\" width=\"120\" height=\"30\" class=\"filter-button\" data-filter=\"requirements\"/>\n")
	sb.WriteString("        <text x=\"880\" y=\"138\" class=\"filter-text\">Requirements</text>\n")
	sb.WriteString("        <rect id=\"btnADRs\" x=\"960\" y=\"120\" width=\"80\" height=\"30\" class=\"filter-button\" data-filter=\"adrs\"/>\n")
	sb.WriteString("        <text x=\"1000\" y=\"138\" class=\"filter-text\">ADRs</text>\n")
	sb.WriteString("        <rect id=\"btnTech\" x=\"1060\" y=\"120\" width=\"100\" height=\"30\" class=\"filter-button\" data-filter=\"tech\"/>\n")
	sb.WriteString("        <text x=\"1110\" y=\"138\" class=\"filter-text\">Technology</text>\n")
	sb.WriteString("    </g>\n\n")

	// Zoom/Pan Controls
	sb.WriteString("    <!-- Zoom Controls -->\n")
	sb.WriteString("    <g id=\"zoomControls\" transform=\"translate(20, 20)\">\n")
	sb.WriteString("        <rect id=\"btnZoomIn\" x=\"0\" y=\"0\" width=\"30\" height=\"30\" class=\"button\"/>\n")
	sb.WriteString("        <text x=\"15\" y=\"15\" class=\"button-text\" font-size=\"20\">+</text>\n")
	sb.WriteString("        <rect id=\"btnZoomOut\" x=\"40\" y=\"0\" width=\"30\" height=\"30\" class=\"button\"/>\n")
	sb.WriteString("        <text x=\"55\" y=\"15\" class=\"button-text\" font-size=\"20\">-</text>\n")
	sb.WriteString("        <rect id=\"btnReset\" x=\"80\" y=\"0\" width=\"60\" height=\"30\" class=\"button\"/>\n")
	sb.WriteString("        <text x=\"110\" y=\"15\" class=\"button-text\" font-size=\"14\">Reset</text>\n")
	sb.WriteString("    </g>\n\n")

	// Search Input
	sb.WriteString("    <!-- Search Input -->\n")
	sb.WriteString("    <foreignObject x=\"200\" y=\"20\" width=\"300\" height=\"40\">\n")
	sb.WriteString("        <body xmlns=\"http://www.w3.org/1999/xhtml\">\n")
	sb.WriteString("            <input type=\"text\" id=\"searchInput\" placeholder=\"Search components...\" style=\"width: 100%; height: 30px; border-radius: 5px; border: 1px solid #bdc3c7; padding: 5px; font-family: sans-serif;\"/>\n")
	sb.WriteString("        </body>\n")
	sb.WriteString("    </foreignObject>\n\n")
}

func (e *Exporter) writeDocPanel(sb *strings.Builder, arch *language.Architecture) {
	sb.WriteString("    <!-- Documentation Panel -->\n")
	sb.WriteString("    <g id=\"docPanelContainer\">\n")
	sb.WriteString("        <rect id=\"docPanel\" x=\"1200\" y=\"180\" width=\"1150\" height=\"1200\" rx=\"8\" ry=\"8\"/>\n\n")

	// Quick Access Buttons
	sb.WriteString("        <g id=\"quickAccess\">\n")
	if len(arch.Requirements) > 0 || e.hasSystemRequirements(arch) {
		sb.WriteString("            <rect id=\"btnQuickReq\" x=\"1250\" y=\"200\" width=\"140\" height=\"35\" class=\"filter-button\" data-content-id=\"requirements_summary\"/>\n")
		sb.WriteString("            <text x=\"1320\" y=\"220\" class=\"filter-text\" text-anchor=\"middle\">All Requirements</text>\n")
	}
	if len(arch.ADRs) > 0 || e.hasSystemADRs(arch) {
		sb.WriteString("            <rect id=\"btnQuickADR\" x=\"1410\" y=\"200\" width=\"120\" height=\"35\" class=\"filter-button\" data-content-id=\"adrs_summary\"/>\n")
		sb.WriteString("            <text x=\"1470\" y=\"220\" class=\"filter-text\" text-anchor=\"middle\">All ADRs</text>\n")
	}
	if e.hasTechnology(arch) {
		sb.WriteString("            <rect id=\"btnQuickTech\" x=\"1550\" y=\"200\" width=\"140\" height=\"35\" class=\"filter-button\" data-content-id=\"technology_summary\"/>\n")
		sb.WriteString("            <text x=\"1620\" y=\"220\" class=\"filter-text\" text-anchor=\"middle\">Technology Stack</text>\n")
	}
	sb.WriteString("        </g>\n\n")

	// Initial Placeholder
	sb.WriteString("        <g id=\"initialText\">\n")
	sb.WriteString("            <text x=\"1775\" y=\"650\" class=\"title\" text-anchor=\"middle\" fill=\"#bdc3c7\">Click a component</text>\n")
	sb.WriteString("            <text x=\"1775\" y=\"700\" class=\"subtitle\" text-anchor=\"middle\">to view its documentation</text>\n")
	sb.WriteString("            <text x=\"1775\" y=\"750\" class=\"subtitle\" text-anchor=\"middle\">or use quick access buttons above</text>\n")
	sb.WriteString("        </g>\n\n")

	// Content Injection Area
	sb.WriteString("        <g id=\"contentInjection\" transform=\"translate(1250, 260)\">\n")
	sb.WriteString("        </g>\n")
	sb.WriteString("    </g>\n\n")
}

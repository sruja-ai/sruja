package dot

import (
	"fmt"
	"strings"
)

// buildNodeHTML generates the HTML label for a node.
// It creates a table with rows for Title, Technology, and Description.
func buildNodeHTML(title, kind, technology, description string) string {
	var sb strings.Builder

	// Main table container
	// CELLBORDER="0" ensures no internal grid lines unless specified
	// CELLSPACING="0" allows us to control spacing via padding
	// CELLPADDING matches LikeC4's effective padding (~10-14px internal)
	sb.WriteString("<<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"0\" CELLPADDING=\"0\">")

	// 1. Title Row
	sb.WriteString("<TR><TD ALIGN=\"TEXT\" BALIGN=\"CENTER\">")
	sb.WriteString("<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"0\" CELLPADDING=\"4\">")
	sb.WriteString("<TR><TD>")
	sb.WriteString(fmt.Sprintf("<FONT POINT-SIZE=\"14\"><B>%s</B></FONT>", escapeHTML(title)))
	sb.WriteString("</TD></TR>")
	sb.WriteString("</TABLE>")
	sb.WriteString("</TD></TR>")

	// 2. Metadata Section (Kind / Technology)
	// Only add if there is content to show
	hasMeta := kind != "" || technology != ""
	if hasMeta {
		sb.WriteString("<TR><TD ALIGN=\"TEXT\" BALIGN=\"CENTER\">")
		sb.WriteString("<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"0\" CELLPADDING=\"2\">")

		if kind != "" {
			sb.WriteString("<TR><TD>")
			sb.WriteString(fmt.Sprintf("<FONT POINT-SIZE=\"10\" COLOR=\"#596980\">[%s]</FONT>", escapeHTML(kind)))
			sb.WriteString("</TD></TR>")
		}

		if technology != "" {
			sb.WriteString("<TR><TD>")
			sb.WriteString(fmt.Sprintf("<FONT POINT-SIZE=\"10\" COLOR=\"#596980\">%s</FONT>", escapeHTML(technology)))
			sb.WriteString("</TD></TR>")
		}

		sb.WriteString("</TABLE>")
		sb.WriteString("</TD></TR>")
	}

	sb.WriteString("</TABLE>>")
	return sb.String()
}

// escapeHTML escapes special characters for Graphviz HTML labels.
func escapeHTML(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, "\"", "&quot;")
	return s
}

package svg

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

type node struct {
	ID    string
	Label string
	Kind  string
}

type point struct{ X, Y int }

func relationLabel(rel *language.Relation) string {
	if rel == nil {
		return ""
	}
	var parts []string
	if len(rel.Tags) > 0 {
		parts = append(parts, strings.Join(rel.Tags, ", "))
	}
	if rel.Verb != nil {
		parts = append(parts, *rel.Verb)
	}
	if rel.Label != nil {
		parts = append(parts, *rel.Label)
	}
	return strings.Join(parts, " ")
}

func labelOrIDSystem(s *language.System) string {
	if s.Label != "" {
		return s.Label
	}
	return s.ID
}

//nolint:unparam // Return value is for future compatibility
func resolveOverviewNode(arch *language.Architecture, contToSys map[string]string, endpoint string) (string, bool) {
	base := lastSegment(endpoint)
	for _, p := range arch.Persons {
		if p.ID == base {
			return p.ID, true
		}
	}
	for _, s := range arch.Systems {
		if s.ID == base {
			return s.ID, true
		}
	}
	if sys, ok := contToSys[base]; ok {
		return sys, true
	}
	// Fallback: treat as external/unknown top-level node
	return base, true
}

func buildContainerSystemMap(arch *language.Architecture) map[string]string {
	m := make(map[string]string)
	for _, s := range arch.Systems {
		for _, c := range s.Containers {
			m[c.ID] = s.ID
		}
		for _, ds := range s.DataStores {
			m[ds.ID] = s.ID
		}
		for _, q := range s.Queues {
			m[q.ID] = s.ID
		}
		for _, c := range s.Containers {
			for _, comp := range c.Components {
				m[comp.ID] = s.ID
			}
		}
	}
	return m
}

func lastSegment(s string) string {
	parts := strings.Split(s, ".")
	return parts[len(parts)-1]
}

func nameByID(m map[string]int64, id int64) string {
	for k, v := range m {
		if v == id {
			return k
		}
	}
	return ""
}

func kindOfTop(arch *language.Architecture, id string) string {
	for _, p := range arch.Persons {
		if p.ID == id {
			return KindPerson
		}
	}
	for _, s := range arch.Systems {
		if s.ID == id {
			return KindSystem
		}
	}
	return KindExternal
}

func lightenColor(hex string) string {
	if len(hex) == 7 && hex[0] == '#' {
		var r, g, b int
		_, err := fmt.Sscanf(hex, "#%02x%02x%02x", &r, &g, &b)
		if err != nil {
			return hex
		}
		r = min(255, r+38)
		g = min(255, g+38)
		b = min(255, b+38)
		return fmt.Sprintf("#%02x%02x%02x", r, g, b)
	}
	return hex
}

//nolint:gocritic,unused,revive // Min is a standard helper, shadowing is intended for simplicity
func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func shouldShowIcon(e *Exporter) bool {
	return e.NodeWidth >= 140 && e.NodeHeight >= 50
}

// getMetadataValue returns the value of a metadata key if present
//
//nolint:unparam // Key parameter is used for flexibility
func getMetadataValue(entries []*language.MetaEntry, key string) string {
	for _, e := range entries {
		if e.Key == key && e.Value != nil {
			return *e.Value
		}
	}
	return ""
}

// styleIconFor returns the icon name from element style or metadata, or a default based on type
//
//nolint:gocyclo // Style logic is inherently complex
func styleIconFor(arch *language.Architecture, id string) string {
	// Check persons
	for _, p := range arch.Persons {
		if p.ID == id {
			if p.Style != nil {
				if v, ok := p.Style["icon"]; ok && v != "" {
					return v
				}
			}
			if v := getMetadataValue(p.Metadata, "icon"); v != "" {
				return v
			}
			return "user"
		}
	}

	for _, s := range arch.Systems {
		if s.ID == id {
			if s.Style != nil {
				if v, ok := s.Style["icon"]; ok && v != "" {
					return v
				}
			}
			if v := getMetadataValue(s.Metadata, "icon"); v != "" {
				return v
			}
			return "server"
		}

		for _, c := range s.Containers {
			if c.ID == id {
				if c.Style != nil {
					if v, ok := c.Style["icon"]; ok && v != "" {
						return v
					}
				}
				if v := getMetadataValue(c.Metadata, "icon"); v != "" {
					return v
				}
				// Default based on technology
				var tech string
				for i := range c.Items {
					item := c.Items[i]
					if item.Technology != nil {
						tech = strings.ToLower(*item.Technology)
						break
					}
				}

				if tech != "" {
					if strings.Contains(tech, "react") || strings.Contains(tech, "angular") || strings.Contains(tech, "vue") || strings.Contains(tech, "web") || strings.Contains(tech, "browser") || strings.Contains(tech, "html") || strings.Contains(tech, "js") {
						return "browser"
					}
					if strings.Contains(tech, "mobile") || strings.Contains(tech, "ios") || strings.Contains(tech, "android") || strings.Contains(tech, "flutter") || strings.Contains(tech, "react native") {
						return "mobile"
					}
				}
				return "server"
			}
			for _, comp := range c.Components {
				if comp.ID == id {
					if comp.Style != nil {
						if v, ok := comp.Style["icon"]; ok && v != "" {
							return v
						}
					}
					if v := getMetadataValue(comp.Metadata, "icon"); v != "" {
						return v
					}
					return KindComponent
				}
			}
		}
		for _, ds := range s.DataStores {
			if ds.ID == id {
				if ds.Style != nil {
					if v, ok := ds.Style["icon"]; ok && v != "" {
						return v
					}
				}
				if v := getMetadataValue(ds.Metadata, "icon"); v != "" {
					return v
				}
				return "database"
			}
		}
		for _, q := range s.Queues {
			if q.ID == id {
				if q.Style != nil {
					if v, ok := q.Style["icon"]; ok && v != "" {
						return v
					}
				}
				if v := getMetadataValue(q.Metadata, "icon"); v != "" {
					return v
				}
				return "queue"
			}
		}
	}
	return ""
}

// wrapLabel splits text into lines that fit within maxChars using word boundaries
func wrapLabel(text string, maxChars int) []string {
	if maxChars <= 3 {
		maxChars = 3
	}
	t := strings.TrimSpace(text)
	if t == "" {
		return []string{}
	}
	words := strings.Fields(t)
	lines := []string{}
	var cur strings.Builder
	for _, w := range words {
		if cur.Len() == 0 {
			if len(w) <= maxChars {
				cur.WriteString(w)
			} else {
				// hard split long word
				for i := 0; i < len(w); i += maxChars {
					end := i + maxChars
					if end > len(w) {
						end = len(w)
					}
					lines = append(lines, w[i:end])
				}
			}
		} else {
			if cur.Len()+1+len(w) <= maxChars {
				cur.WriteString(" ")
				cur.WriteString(w)
			} else {
				lines = append(lines, cur.String())
				cur.Reset()
				if len(w) <= maxChars {
					cur.WriteString(w)
				} else {
					for i := 0; i < len(w); i += maxChars {
						end := i + maxChars
						if end > len(w) {
							end = len(w)
						}
						lines = append(lines, w[i:end])
					}
				}
			}
		}
	}
	if cur.Len() > 0 {
		lines = append(lines, cur.String())
	}
	// limit lines to prevent excessive height; append ellipsis if truncated
	if len(lines) > 3 {
		lines = append(lines[:2], "…")
	}
	return lines
}

// pkg/export/json/metadata.go
// Metadata extraction and layout handling for JSON export
package json

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

//nolint:funlen,gocyclo // Metadata extraction is detailed and complex
func populateMetadataFromDSL(meta *MetadataJSON, arch *language.Architecture) {
	layout := make(map[string]LayoutData)

	if val, ok := arch.MetaString("brand_logo"); ok {
		meta.BrandLogo = val
	}
	if val, ok := arch.MetaString("layout_engine"); ok {
		meta.LayoutEngine = val
	}
	if val, ok := arch.MetaString("layout"); ok {
		if meta.LayoutEngine == "" {
			meta.LayoutEngine = val
		}
	}

	parseInt := func(s string) (int, bool) {
		if s == "" {
			return 0, false
		}
		sign := 1
		if s[0] == '-' {
			sign = -1
			s = s[1:]
		}
		n := 0
		for i := 0; i < len(s); i++ {
			c := s[i]
			if c < '0' || c > '9' {
				return 0, false
			}
			n = n*10 + int(c-'0')
		}
		return n * sign, true
	}

	addPos := func(id string, md []*language.MetaEntry) {
		var xStr, yStr, wStr, hStr string
		for _, m := range md {
			switch m.Key {
			case "layout_x", "pos_x":
				if m.Value != nil {
					xStr = *m.Value
				}
			case "layout_y", "pos_y":
				if m.Value != nil {
					yStr = *m.Value
				}
			case "layout_w", "pos_w":
				if m.Value != nil {
					wStr = *m.Value
				}
			case "layout_h", "pos_h":
				if m.Value != nil {
					hStr = *m.Value
				}
			}
		}
		x, okx := parseInt(xStr)
		y, oky := parseInt(yStr)
		if okx && oky {
			var wPtr, hPtr *int
			if w, okw := parseInt(wStr); okw {
				wPtr = &w
			}
			if h, okh := parseInt(hStr); okh {
				hPtr = &h
			}
			layout[id] = LayoutData{X: x, Y: y, Width: wPtr, Height: hPtr}
		}
	}

	for _, p := range arch.Persons {
		if len(p.Metadata) > 0 {
			addPos(p.ID, p.Metadata)
		}
	}
	for _, s := range arch.Systems {
		if len(s.Metadata) > 0 {
			addPos(s.ID, s.Metadata)
		}
		buildQualifiedID := func(prefix, id string) string {
			if prefix == "" {
				return id
			}
			buf := make([]byte, 0, len(prefix)+len(id)+1)
			buf = append(buf, prefix...)
			buf = append(buf, '.')
			buf = append(buf, id...)
			return string(buf)
		}

		for _, c := range s.Containers {
			if len(c.Metadata) > 0 {
				addPos(buildQualifiedID(s.ID, c.ID), c.Metadata)
			}
		}
		for _, d := range s.DataStores {
			if len(d.Metadata) > 0 {
				addPos(buildQualifiedID(s.ID, d.ID), d.Metadata)
			}
		}
		for _, q := range s.Queues {
			if len(q.Metadata) > 0 {
				addPos(buildQualifiedID(s.ID, q.ID), q.Metadata)
			}
		}
	}
	for _, c := range arch.Containers {
		if len(c.Metadata) > 0 {
			addPos(c.ID, c.Metadata)
		}
	}
	for _, d := range arch.DataStores {
		if len(d.Metadata) > 0 {
			addPos(d.ID, d.Metadata)
		}
	}
	for _, q := range arch.Queues {
		if len(q.Metadata) > 0 {
			addPos(q.ID, q.Metadata)
		}
	}

	if len(layout) > 0 {
		meta.Layout = layout
	}
}

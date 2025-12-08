package json

import (
	"encoding/json"
	"strconv"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

type Exporter struct {
	Extended bool // If true, include pre-computed views in output
}

func NewExporter() *Exporter { return &Exporter{} }

func (e *Exporter) Export(arch *language.Architecture) (string, error) {
	doc := ArchitectureJSON{
		Metadata:     NewMetadata(arch.Name),
		Architecture: convertArchitectureToJSON(arch),
		Navigation:   buildNavigation(arch),
	}

	// Populate metadata layout and branding from DSL metadata
	populateMetadataFromDSL(&doc.Metadata, arch)

	// Generate pre-computed views if extended mode enabled
	if e.Extended {
		doc.Views = GenerateViews(arch)
	}

	b, err := json.MarshalIndent(doc, "", "  ")
	if err != nil {
		return "", err
	}
	return string(b), nil
}

//nolint:funlen,gocyclo // Metadata extraction is detailed and complex
func populateMetadataFromDSL(meta *MetadataJSON, arch *language.Architecture) {
	// Layout positions map
	layout := make(map[string]LayoutData)

	// Architecture-level branding and layout engine
	if val, ok := arch.MetaString("brand_logo"); ok {
		meta.BrandLogo = val
	}
	if val, ok := arch.MetaString("layout_engine"); ok {
		meta.LayoutEngine = val
	}
	if val, ok := arch.MetaString("layout"); ok { // allow simple key
		// if user sets layout: elk in metadata, capture as engine
		if meta.LayoutEngine == "" {
			meta.LayoutEngine = val
		}
	}

	// Helper to parse integer safely
	parseInt := func(s string) (int, bool) {
		if s == "" {
			return 0, false
		}
		// trim quotes if any; values are already strings without quotes in AST
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

	// Helper to pick XY from metadata entries with keys like layout_x/layout_y or pos_x/pos_y
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

	// Persons
	for _, p := range arch.Persons {
		if len(p.Metadata) > 0 {
			addPos(p.ID, p.Metadata)
		}
	}
	// Systems and nested
	for _, s := range arch.Systems {
		if len(s.Metadata) > 0 {
			addPos(s.ID, s.Metadata)
		}
		for _, c := range s.Containers {
			if len(c.Metadata) > 0 {
				addPos(s.ID+"."+c.ID, c.Metadata)
			}
		}
		for _, d := range s.DataStores {
			if len(d.Metadata) > 0 {
				addPos(s.ID+"."+d.ID, d.Metadata)
			}
		}
		for _, q := range s.Queues {
			if len(q.Metadata) > 0 {
				addPos(s.ID+"."+q.ID, q.Metadata)
			}
		}
	}
	// Top-level items
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

//nolint:funlen,gocyclo // Conversion logic is verbose and complex
func convertArchitectureToJSON(arch *language.Architecture) ArchitectureBody {
	body := ArchitectureBody{}

	// Imports first
	if len(arch.Imports) > 0 {
		imps := make([]ImportJSON, 0, len(arch.Imports))
		for _, imp := range arch.Imports {
			imps = append(imps, ImportJSON{Path: imp.Path, Alias: imp.Alias})
		}
		body.Imports = imps
	}

	// Systems
	if len(arch.Systems) > 0 {
		systems := make([]SystemJSON, 0, len(arch.Systems))
		for _, s := range arch.Systems {
			systems = append(systems, convertSystem(s))
		}
		body.Systems = systems
	}

	// Persons
	if len(arch.Persons) > 0 {
		persons := make([]PersonJSON, 0, len(arch.Persons))
		for _, p := range arch.Persons {
			persons = append(persons, PersonJSON{ID: p.ID, Label: p.Label, Description: p.Description})
		}
		body.Persons = persons
	}

	// Top-level relations
	if len(arch.Relations) > 0 {
		rels := make([]RelationJSON, 0, len(arch.Relations))
		for _, r := range arch.Relations {
			var verb *string
			if r.Verb != nil {
				verb = r.Verb
			}
			rels = append(rels, RelationJSON{From: sanitize(r.From.String()), To: sanitize(r.To.String()), Verb: verb, Label: r.Label, Tags: r.Tags})
		}
		body.Relations = rels
	}

	// Requirements
	if len(arch.Requirements) > 0 {
		reqs := make([]RequirementJSON, 0, len(arch.Requirements))
		for _, r := range arch.Requirements {
			reqs = append(reqs, RequirementJSON{
				ID:          r.ID,
				Type:        strVal(r.Type),
				Title:       strVal(r.Description),
				Description: strVal(r.Description),
			})
		}
		body.Requirements = reqs
	}

	// ADRs
	if len(arch.ADRs) > 0 {
		adrs := make([]ADRJSON, 0, len(arch.ADRs))
		for _, a := range arch.ADRs {
			title := ""
			if a.Title != nil {
				title = *a.Title
			}
			adr := ADRJSON{
				ID:    a.ID,
				Title: title,
			}
			if a.Body != nil {
				adr.Status = a.Body.Status
				adr.Context = a.Body.Context
				adr.Decision = a.Body.Decision
				adr.Consequences = a.Body.Consequences
			}
			adrs = append(adrs, adr)
		}
		body.ADRs = adrs
	}

	// Scenarios
	if len(arch.Scenarios) > 0 {
		scenarios := make([]ScenarioJSON, 0, len(arch.Scenarios))
		for _, s := range arch.Scenarios {
			scenario := ScenarioJSON{
				ID:          s.ID,
				Title:       s.Title,
				Label:       s.Title,
				Description: s.Description,
			}
			// Convert scenario steps
			if len(s.Steps) > 0 {
				steps := make([]ScenarioStepJSON, 0, len(s.Steps))
				for _, step := range s.Steps {
					steps = append(steps, ScenarioStepJSON{
						From:        step.From.String(),
						To:          step.To.String(),
						Description: step.Description,
						Tags:        step.Tags,
						Order:       stringPtrToIntPtr(step.Order),
					})
				}
				scenario.Steps = steps
			}
			scenarios = append(scenarios, scenario)
		}
		body.Scenarios = scenarios
	}

	// Flows
	if len(arch.Flows) > 0 {
		flows := make([]FlowJSON, 0, len(arch.Flows))
		for _, f := range arch.Flows {
			flow := FlowJSON{
				ID:          f.ID,
				Title:       f.Title,
				Label:       f.Title,
				Description: f.Description,
			}
			// Convert flow steps (Flow is alias to Scenario - uses ScenarioStep)
			if len(f.Steps) > 0 {
				steps := make([]ScenarioStepJSON, 0, len(f.Steps))
				for _, step := range f.Steps {
					steps = append(steps, ScenarioStepJSON{
						From:        step.From.String(),
						To:          step.To.String(),
						Description: step.Description,
						Tags:        step.Tags,
						Order:       stringPtrToIntPtr(step.Order),
					})
				}
				flow.Steps = steps
			}
			flows = append(flows, flow)
		}
		body.Flows = flows
	}

	return body
}

func stringPtrToIntPtr(s *string) *int {
	if s == nil {
		return nil
	}
	val, err := strconv.Atoi(*s)
	if err != nil {
		return nil
	}
	return &val
}

func convertSystem(s *language.System) SystemJSON {
	out := SystemJSON{ID: s.ID, Label: s.Label, Description: s.Description}

	if len(s.Containers) > 0 {
		containers := make([]ContainerJSON, 0, len(s.Containers))
		for _, c := range s.Containers {
			containers = append(containers, convertContainer(c))
		}
		out.Containers = containers
	}
	if len(s.Components) > 0 {
		comps := make([]ComponentJSON, 0, len(s.Components))
		for _, c := range s.Components {
			comps = append(comps, convertComponent(c))
		}
		out.Components = comps
	}
	if len(s.DataStores) > 0 {
		dss := make([]DataStoreJSON, 0, len(s.DataStores))
		for _, d := range s.DataStores {
			dss = append(dss, DataStoreJSON{ID: d.ID, Label: d.Label})
		}
		out.DataStores = dss
	}
	if len(s.Queues) > 0 {
		qs := make([]QueueJSON, 0, len(s.Queues))
		for _, q := range s.Queues {
			qs = append(qs, QueueJSON{ID: q.ID, Label: q.Label})
		}
		out.Queues = qs
	}
	if len(s.Relations) > 0 {
		rels := make([]RelationJSON, 0, len(s.Relations))
		for _, r := range s.Relations {
			var verb *string
			if r.Verb != nil {
				verb = r.Verb
			}
			rels = append(rels, RelationJSON{From: sanitize(r.From.String()), To: sanitize(r.To.String()), Verb: verb, Label: r.Label, Tags: r.Tags})
		}
		out.Relations = rels
	}
	return out
}

//nolint:gocyclo // Conversion logic is complex but straightforward
func convertContainer(c *language.Container) ContainerJSON {
	out := ContainerJSON{
		ID:          c.ID,
		Label:       c.Label,
		Description: c.Description,
		Version:     c.Version,
	}

	// Extract Technology and Tags from Items
	for i := range c.Items {
		item := &c.Items[i]
		if item.Technology != nil {
			out.Technology = item.Technology
		}
		if len(item.Tags) > 0 {
			out.Tags = item.Tags
		}
	}

	if len(c.Requirements) > 0 {
		reqs := make([]RequirementJSON, 0, len(c.Requirements))
		for _, r := range c.Requirements {
			reqs = append(reqs, RequirementJSON{
				ID:          r.ID,
				Type:        strVal(r.Type),
				Title:       strVal(r.Description),
				Description: strVal(r.Description),
			})
		}
		out.Requirements = reqs
	}

	if len(c.ADRs) > 0 {
		adrs := make([]ADRJSON, 0, len(c.ADRs))
		for _, a := range c.ADRs {
			title := ""
			if a.Title != nil {
				title = *a.Title
			}
			adr := ADRJSON{
				ID:    a.ID,
				Title: title,
			}
			if a.Body != nil {
				adr.Status = a.Body.Status
				adr.Context = a.Body.Context
				adr.Decision = a.Body.Decision
				adr.Consequences = a.Body.Consequences
			}
			adrs = append(adrs, adr)
		}
		out.ADRs = adrs
	}

	if len(c.Components) > 0 {
		comps := make([]ComponentJSON, 0, len(c.Components))
		for _, comp := range c.Components {
			comps = append(comps, convertComponent(comp))
		}
		out.Components = comps
	}
	if len(c.DataStores) > 0 {
		dss := make([]DataStoreJSON, 0, len(c.DataStores))
		for _, d := range c.DataStores {
			dss = append(dss, DataStoreJSON{ID: d.ID, Label: d.Label})
		}
		out.DataStores = dss
	}
	if len(c.Queues) > 0 {
		qs := make([]QueueJSON, 0, len(c.Queues))
		for _, q := range c.Queues {
			qs = append(qs, QueueJSON{ID: q.ID, Label: q.Label})
		}
		out.Queues = qs
	}
	if len(c.Relations) > 0 {
		rels := make([]RelationJSON, 0, len(c.Relations))
		for _, r := range c.Relations {
			var verb *string
			if r.Verb != nil {
				verb = r.Verb
			}
			rels = append(rels, RelationJSON{From: sanitize(r.From.String()), To: sanitize(r.To.String()), Verb: verb, Label: r.Label, Tags: r.Tags})
		}
		out.Relations = rels
	}
	if len(c.Metadata) > 0 {
		meta := make([]MetadataEntryJSON, 0, len(c.Metadata))
		for _, m := range c.Metadata {
			meta = append(meta, MetadataEntryJSON{Key: m.Key, Value: m.Value, Array: m.Array})
		}
		out.Metadata = meta
	}
	return out
}

func convertComponent(c *language.Component) ComponentJSON {
	out := ComponentJSON{
		ID:          c.ID,
		Label:       c.Label,
		Description: c.Description,
		Technology:  c.Technology,
	}

	if len(c.Requirements) > 0 {
		reqs := make([]RequirementJSON, 0, len(c.Requirements))
		for _, r := range c.Requirements {
			reqs = append(reqs, RequirementJSON{
				ID:          r.ID,
				Type:        strVal(r.Type),
				Title:       strVal(r.Description),
				Description: strVal(r.Description),
			})
		}
		out.Requirements = reqs
	}

	if len(c.ADRs) > 0 {
		adrs := make([]ADRJSON, 0, len(c.ADRs))
		for _, a := range c.ADRs {
			title := ""
			if a.Title != nil {
				title = *a.Title
			}
			adr := ADRJSON{
				ID:    a.ID,
				Title: title,
			}
			if a.Body != nil {
				adr.Status = a.Body.Status
				adr.Context = a.Body.Context
				adr.Decision = a.Body.Decision
				adr.Consequences = a.Body.Consequences
			}
			adrs = append(adrs, adr)
		}
		out.ADRs = adrs
	}

	if len(c.Relations) > 0 {
		rels := make([]RelationJSON, 0, len(c.Relations))
		for _, r := range c.Relations {
			var verb *string
			if r.Verb != nil {
				verb = r.Verb
			}
			rels = append(rels, RelationJSON{From: sanitize(r.From.String()), To: sanitize(r.To.String()), Verb: verb, Label: r.Label, Tags: r.Tags})
		}
		out.Relations = rels
	}
	if len(c.Metadata) > 0 {
		meta := make([]MetadataEntryJSON, 0, len(c.Metadata))
		for _, m := range c.Metadata {
			meta = append(meta, MetadataEntryJSON{Key: m.Key, Value: m.Value, Array: m.Array})
		}
		out.Metadata = meta
	}
	return out
}

func buildNavigation(arch *language.Architecture) NavigationJSON {
	nav := NavigationJSON{}
	// Simple default levels; can be refined later
	nav.Levels = []string{"level1", "level2", "level3"}
	if len(arch.Scenarios) > 0 {
		ss := make([]ScenarioNav, 0, len(arch.Scenarios))
		for _, s := range arch.Scenarios {
			// Scenario uses Title not Label
			title := s.Title
			ss = append(ss, ScenarioNav{ID: idOrLabel(s.ID, title), Label: title})
		}
		nav.Scenarios = ss
	}
	// Flows removed - Flow type not yet defined (architecture construct, to be implemented)
	// if len(arch.Flows) > 0 {
	//     fs := make([]FlowNav, 0, len(arch.Flows))
	//     for _, f := range arch.Flows {
	//         title := f.Title
	//         fs = append(fs, FlowNav{ ID: idOrLabel(f.ID, title), Label: title })
	//     }
	//     nav.Flows = fs
	// }
	// Domains removed - DDD feature, deferred to Phase 2
	// if len(arch.Domains) > 0 {
	//     ds := make([]DomainNav, 0, len(arch.Domains))
	//     for _, d := range arch.Domains {
	//         ds = append(ds, DomainNav{ ID: idOrLabel(d.ID, d.Label), Label: d.Label })
	//     }
	//     nav.Domains = ds
	// }
	return nav
}

func idOrLabel(id, label string) string {
	if strings.TrimSpace(id) != "" {
		return id
	}
	return label
}

func sanitize(s string) string {
	// minimal sanitation to avoid empty strings; extend as needed
	return strings.TrimSpace(s)
}

func strVal(s *string) string {
	if s == nil {
		return ""
	}
	return *s
}

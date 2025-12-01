package json

import (
    "encoding/json"
    "strings"

    "github.com/sruja-ai/sruja/pkg/language"
)

type Exporter struct{}

func NewExporter() *Exporter { return &Exporter{} }

func (e *Exporter) Export(arch *language.Architecture) (string, error) {
    doc := ArchitectureJSON{
        Metadata:    NewMetadata(arch.Name),
        Architecture: convertArchitectureToJSON(arch),
        Navigation:  buildNavigation(arch),
    }

    b, err := json.MarshalIndent(doc, "", "  ")
    if err != nil {
        return "", err
    }
    return string(b), nil
}

func convertArchitectureToJSON(arch *language.Architecture) ArchitectureBody {
    body := ArchitectureBody{}

    // Imports first
    if len(arch.Imports) > 0 {
        imps := make([]ImportJSON, 0, len(arch.Imports))
        for _, imp := range arch.Imports {
            imps = append(imps, ImportJSON{ Path: imp.Path, Alias: imp.Alias })
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
            persons = append(persons, PersonJSON{ ID: p.ID, Label: p.Label, Description: p.Description })
        }
        body.Persons = persons
    }

    // Top-level relations
    if len(arch.Relations) > 0 {
        rels := make([]RelationJSON, 0, len(arch.Relations))
        for _, r := range arch.Relations {
            rels = append(rels, RelationJSON{ From: sanitize(r.From), To: sanitize(r.To), Verb: r.Verb, Label: r.Label })
        }
        body.Relations = rels
    }

    return body
}

func convertSystem(s *language.System) SystemJSON {
    out := SystemJSON{ ID: s.ID, Label: s.Label, Description: s.Description }

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
            dss = append(dss, DataStoreJSON{ ID: d.ID, Label: d.Label })
        }
        out.DataStores = dss
    }
    if len(s.Queues) > 0 {
        qs := make([]QueueJSON, 0, len(s.Queues))
        for _, q := range s.Queues {
            qs = append(qs, QueueJSON{ ID: q.ID, Label: q.Label })
        }
        out.Queues = qs
    }
    if len(s.Relations) > 0 {
        rels := make([]RelationJSON, 0, len(s.Relations))
        for _, r := range s.Relations {
            rels = append(rels, RelationJSON{ From: sanitize(r.From), To: sanitize(r.To), Verb: r.Verb, Label: r.Label })
        }
        out.Relations = rels
    }
    return out
}

func convertContainer(c *language.Container) ContainerJSON {
    out := ContainerJSON{ ID: c.ID, Label: c.Label, Description: c.Description }
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
            dss = append(dss, DataStoreJSON{ ID: d.ID, Label: d.Label })
        }
        out.DataStores = dss
    }
    if len(c.Queues) > 0 {
        qs := make([]QueueJSON, 0, len(c.Queues))
        for _, q := range c.Queues {
            qs = append(qs, QueueJSON{ ID: q.ID, Label: q.Label })
        }
        out.Queues = qs
    }
    if len(c.Relations) > 0 {
        rels := make([]RelationJSON, 0, len(c.Relations))
        for _, r := range c.Relations {
            rels = append(rels, RelationJSON{ From: sanitize(r.From), To: sanitize(r.To), Verb: r.Verb, Label: r.Label })
        }
        out.Relations = rels
    }
    return out
}

func convertComponent(c *language.Component) ComponentJSON {
    out := ComponentJSON{ ID: c.ID, Label: c.Label, Description: c.Description }
    if len(c.Relations) > 0 {
        rels := make([]RelationJSON, 0, len(c.Relations))
        for _, r := range c.Relations {
            rels = append(rels, RelationJSON{ From: sanitize(r.From), To: sanitize(r.To), Verb: r.Verb, Label: r.Label })
        }
        out.Relations = rels
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
            ss = append(ss, ScenarioNav{ ID: idOrLabel(s.ID, title), Label: title })
        }
        nav.Scenarios = ss
    }
    if len(arch.Flows) > 0 {
        fs := make([]FlowNav, 0, len(arch.Flows))
        for _, f := range arch.Flows {
            title := f.Title
            fs = append(fs, FlowNav{ ID: idOrLabel(f.ID, title), Label: title })
        }
        nav.Flows = fs
    }
    if len(arch.Domains) > 0 {
        ds := make([]DomainNav, 0, len(arch.Domains))
        for _, d := range arch.Domains {
            ds = append(ds, DomainNav{ ID: idOrLabel(d.ID, d.Label), Label: d.Label })
        }
        nav.Domains = ds
    }
    return nav
}

func idOrLabel(id, label string) string {
    if strings.TrimSpace(id) != "" { return id }
    return label
}

func sanitize(s string) string {
    // minimal sanitation to avoid empty strings; extend as needed
    return strings.TrimSpace(s)
}

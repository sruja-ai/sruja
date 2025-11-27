package query

type ElementType string

const (
    TypeSystem    ElementType = "system"
    TypeContainer ElementType = "container"
    TypeComponent ElementType = "component"
    TypePerson    ElementType = "person"
    TypeDataStore ElementType = "datastore"
    TypeQueue     ElementType = "queue"
)

type ResultElement struct {
    ID    string      `json:"id"`
    Label string      `json:"label"`
    Type  ElementType `json:"type"`
}

type ResultRelation struct {
    From  string `json:"from"`
    To    string `json:"to"`
    Verb  string `json:"verb"`
    Label string `json:"label"`
}

type QueryResult struct {
    Elements  []ResultElement  `json:"elements"`
    Relations []ResultRelation `json:"relations"`
}


package views

import (
    "testing"
    "github.com/sruja-ai/sruja/pkg/language"
)

func mkStrV(s string) *string { return &s }

func TestApplyStyles_DefaultSystemTagAndDatastoreTags(t *testing.T) {
    arch := &language.Architecture{Systems: []*language.System{{ID: "S1", Label: "System1"}}}
    ds := &language.DataStore{ID: "DB", Label: "Database", Metadata: []*language.MetaEntry{{Key: "tags", Array: []string{"storage"}}}}
    arch.Systems[0].DataStores = []*language.DataStore{ds}

    vb := &language.ViewBlock{Styles: &language.StylesBlock{Styles: []*language.ElementStyle{
        {Target: "element", Tag: "System", Properties: []*language.StyleProperty{{Key: "color", Value: mkStrV("\"blue\"")}}},
        {Target: "element", Tag: "storage", Properties: []*language.StyleProperty{{Key: "shape", Value: mkStrV("\"cylinder\"")}}},
    }}}

    styles := ApplyStyles(arch, vb)
    // Default tag System should apply to system id
    if styles["S1"]["color"] != "blue" { t.Fatalf("expected system style color blue, got %q", styles["S1"]["color"]) }
    // Metadata tag "storage" should apply to S1.DB
    if styles["S1.DB"]["shape"] != "cylinder" { t.Fatalf("expected datastore shape cylinder, got %q", styles["S1.DB"]["shape"]) }
}

package json

import (
    "testing"
    "github.com/sruja-ai/sruja/pkg/language"
)

func TestPopulateMetadataFromDSL_LayoutExtraction(t *testing.T) {
    arch := &language.Architecture{Systems: []*language.System{
        {
            ID:    "Shop",
            Label: "Shop",
            Containers: []*language.Container{
                {ID: "Web", Label: "Web App", Metadata: []*language.MetaEntry{{Key: "pos_x", Value: mkStr("10")}, {Key: "pos_y", Value: mkStr("20")}, {Key: "pos_w", Value: mkStr("100")}}},
            },
            DataStores: []*language.DataStore{
                {ID: "DB", Label: "Database", Metadata: []*language.MetaEntry{{Key: "layout_x", Value: mkStr("-5")}, {Key: "layout_y", Value: mkStr("15")}}},
            },
        },
    }}

    meta := NewMetadata("Test")
    populateMetadataFromDSL(&meta, arch)

    if meta.Layout == nil {
        t.Fatalf("expected layout map to be populated")
    }
    if _, ok := meta.Layout["Shop.Web"]; !ok {
        t.Fatalf("expected Shop.Web entry in layout")
    }
    if _, ok := meta.Layout["Shop.DB"]; !ok {
        t.Fatalf("expected Shop.DB entry in layout")
    }
    lw := meta.Layout["Shop.Web"].Width
    if lw == nil || *lw != 100 {
        t.Fatalf("expected width=100 for Shop.Web, got %+v", lw)
    }
    if meta.Layout["Shop.DB"].X != -5 || meta.Layout["Shop.DB"].Y != 15 {
        t.Fatalf("expected Shop.DB pos -5,15 got %d,%d", meta.Layout["Shop.DB"].X, meta.Layout["Shop.DB"].Y)
    }
}

func mkStr(s string) *string { return &s }

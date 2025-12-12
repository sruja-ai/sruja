package views

import (
    "testing"

    "github.com/sruja-ai/sruja/pkg/language"
)

func strPtr(s string) *string { return &s }

func TestApplyViewExpressions_IncludeWildcard(t *testing.T) {
    arch := &language.Architecture{
        Systems: []*language.System{
            {
                ID:    "Backend",
                Label: "Backend",
            },
        },
    }

    // Populate nested elements
    sys := arch.Systems[0]
    sys.Containers = []*language.Container{{ID: "API", Label: "API"}}
    sys.DataStores = []*language.DataStore{{ID: "DB", Label: "Database"}}
    sys.Queues = []*language.Queue{{ID: "MQ", Label: "Events"}}
    sys.Containers[0].Components = []*language.Component{{ID: "Auth", Label: "Auth"}}

    view := &language.View{
        Type:  "container",
        Scope: language.QualifiedIdent{Parts: []string{"Backend"}},
        Name:  "\"Containers\"",
        Expressions: []*language.ViewExpression{
            {Type: "include", Wildcard: strPtr("*")},
        },
    }

    included, err := ApplyViewExpressions(arch, view)
    if err != nil {
        t.Fatalf("ApplyViewExpressions error: %v", err)
    }

    // System and elements should be included
    checks := []string{
        "Backend",
        "Backend.API",
        "Backend.DB",
        "Backend.MQ",
        "Backend.API.Auth",
    }
    for _, id := range checks {
        if !included[id] {
            t.Errorf("Expected element '%s' to be included", id)
        }
    }
}

func TestApplyStyles_Tags(t *testing.T) {
    arch := &language.Architecture{
        Systems: []*language.System{
            {ID: "Backend", Label: "Backend"},
        },
    }
    sys := arch.Systems[0]
    sys.Containers = []*language.Container{{ID: "API", Label: "API"}}
    sys.Containers[0].Metadata = []*language.MetaEntry{{Key: "tags", Array: []string{"api"}}}

    // Style block that targets tag "api"
    vb := &language.ViewBlock{
        Styles: &language.StylesBlock{
            Styles: []*language.ElementStyle{
                {
                    Target: "element",
                    Tag:    "api",
                    Properties: []*language.StyleProperty{
                        {Key: "color", Value: strPtr("\"red\"")},
                    },
                },
            },
        },
    }

    styles := ApplyStyles(arch, vb)
    contID := "Backend.API"
    if styles == nil || styles[contID] == nil {
        t.Fatalf("Expected styles for %s", contID)
    }
    if styles[contID]["color"] != "red" {
        t.Errorf("Expected color 'red', got '%s'", styles[contID]["color"])
    }
}

func TestFindViewByName(t *testing.T) {
    arch := &language.Architecture{
        Views: &language.ViewBlock{
            Views: []*language.View{
                {Type: "container", Scope: language.QualifiedIdent{Parts: []string{"Backend"}}, Name: "\"My View\""},
                {Type: "component", Scope: language.QualifiedIdent{Parts: []string{"Backend.API"}}, Name: "\"Other\""},
            },
        },
    }

    v := FindViewByName(arch, "My View")
    if v == nil {
        t.Fatal("Expected to find view 'My View'")
    }
    if v.Type != "container" {
        t.Errorf("Expected type 'container', got '%s'", v.Type)
    }
}

func TestGetAutolayoutDirection(t *testing.T) {
    // Explicit LR
    v1 := &language.View{Autolayout: strPtr("LR")}
    if dir := GetAutolayoutDirection(v1, "TB"); dir != "LR" {
        t.Errorf("Expected 'LR', got '%s'", dir)
    }

    // Explicit TB
    v2 := &language.View{Autolayout: strPtr("TB")}
    if dir := GetAutolayoutDirection(v2, "LR"); dir != "TB" {
        t.Errorf("Expected 'TB', got '%s'", dir)
    }

    // AUTO falls back to default
    v3 := &language.View{Autolayout: strPtr("AUTO")}
    if dir := GetAutolayoutDirection(v3, "LR"); dir != "LR" {
        t.Errorf("Expected default 'LR', got '%s'", dir)
    }

    // Nil falls back to default
    if dir := GetAutolayoutDirection(nil, "TB"); dir != "TB" {
        t.Errorf("Expected default 'TB', got '%s'", dir)
    }
}


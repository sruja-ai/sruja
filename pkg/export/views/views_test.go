package views

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestApplyViewExpressions_IncludeWildcard(t *testing.T) {
	dsl := `model {
        Backend = system "Backend" {
            API = container "API" {
                Auth = component "Auth"
            }
            DB = database "Database"
            MQ = queue "Events"
        }
    }`
	prog := parseDSL(t, dsl)

	view := &language.View{
		Type:  "container",
		Scope: language.QualifiedIdent{Parts: []string{"Backend"}},
		Name:  "\"Containers\"",
		Expressions: []*language.ViewExpression{
			{Type: "include", Wildcard: strPtr("*")},
		},
	}

	included, err := ApplyViewExpressions(prog, view)
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
	dsl := `model {
        Backend = system "Backend" {
            API = container "API" #api
        }
    }`
	prog := parseDSL(t, dsl)

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

	styles := ApplyStyles(prog, vb)
	contID := "Backend.API"
	if styles == nil || styles[contID] == nil {
		t.Fatalf("Expected styles for %s", contID)
	}
	if styles[contID]["color"] != "red" {
		t.Errorf("Expected color 'red', got '%s'", styles[contID]["color"])
	}
}

func TestFindViewByName(t *testing.T) {
	// Note: FindViewByName currently returns nil as ViewBlock is not part of Program
	// This test is skipped until ViewBlock support is added to Program
	prog := &language.Program{}
	v := FindViewByName(prog, "My View")
	if v != nil {
		t.Error("FindViewByName not yet implemented for Program")
	}
}

func TestApplyViewExpressions_NilView(t *testing.T) {
	prog := parseDSL(t, `model { sys = system "System" }`)
	_, err := ApplyViewExpressions(prog, nil)
	if err == nil {
		t.Error("ApplyViewExpressions should error on nil view")
	}
}

func TestApplyViewExpressions_Exclude(t *testing.T) {
	dsl := `model {
		Backend = system "Backend" {
			API = container "API"
			DB = database "Database"
		}
	}`
	prog := parseDSL(t, dsl)

	view := &language.View{
		Type:  "container",
		Scope: language.QualifiedIdent{Parts: []string{"Backend"}},
		Expressions: []*language.ViewExpression{
			{Type: "include", Wildcard: strPtr("*")},
			{Type: "exclude", Elements: []language.QualifiedIdent{
				{Parts: []string{"Backend", "DB"}},
			}},
		},
	}

	included, err := ApplyViewExpressions(prog, view)
	if err != nil {
		t.Fatalf("ApplyViewExpressions error: %v", err)
	}

	if included["Backend.DB"] {
		t.Error("Backend.DB should be excluded")
	}
	if !included["Backend.API"] {
		t.Error("Backend.API should be included")
	}
}

func TestApplyViewExpressions_IncludeElements(t *testing.T) {
	dsl := `model {
		Backend = system "Backend" {
			API = container "API"
			DB = database "Database"
		}
	}`
	prog := parseDSL(t, dsl)

	view := &language.View{
		Type:  "container",
		Scope: language.QualifiedIdent{Parts: []string{"Backend"}},
		Expressions: []*language.ViewExpression{
			{Type: "include", Elements: []language.QualifiedIdent{
				{Parts: []string{"Backend", "API"}},
			}},
		},
	}

	included, err := ApplyViewExpressions(prog, view)
	if err != nil {
		t.Fatalf("ApplyViewExpressions error: %v", err)
	}

	if !included["Backend.API"] {
		t.Error("Backend.API should be included")
	}
	if included["Backend.DB"] {
		t.Error("Backend.DB should not be included (not in include list)")
	}
}

func TestApplyStyles_Nil(t *testing.T) {
	prog := parseDSL(t, `model { sys = system "System" }`)
	styles := ApplyStyles(prog, nil)
	if styles != nil {
		t.Error("ApplyStyles should return nil for nil viewBlock")
	}
}

func TestApplyStyles_EmptyStyles(t *testing.T) {
	prog := parseDSL(t, `model { sys = system "System" }`)
	viewBlock := &language.ViewBlock{
		Styles: &language.StylesBlock{},
	}
	styles := ApplyStyles(prog, viewBlock)
	if styles == nil {
		t.Error("ApplyStyles should return empty map, not nil")
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

	// Auto returns default
	v3 := &language.View{Autolayout: strPtr("AUTO")}
	if dir := GetAutolayoutDirection(v3, "LR"); dir != "LR" {
		t.Errorf("Expected 'LR' (default), got '%s'", dir)
	}

	// Invalid value returns default
	v4 := &language.View{Autolayout: strPtr("INVALID")}
	if dir := GetAutolayoutDirection(v4, "TB"); dir != "TB" {
		t.Errorf("Expected 'TB' (default), got '%s'", dir)
	}

	// Nil view returns default
	if dir := GetAutolayoutDirection(nil, "LR"); dir != "LR" {
		t.Errorf("Expected 'LR' (default), got '%s'", dir)
	}

	// View with nil autolayout returns default
	v5 := &language.View{Autolayout: nil}
	if dir := GetAutolayoutDirection(v5, "TB"); dir != "TB" {
		t.Errorf("Expected 'TB' (default), got '%s'", dir)
	}

	// Nil falls back to default
	if dir := GetAutolayoutDirection(nil, "TB"); dir != "TB" {
		t.Errorf("Expected default 'TB', got '%s'", dir)
	}
}

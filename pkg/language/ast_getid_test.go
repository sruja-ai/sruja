// pkg/language/ast_getid_test.go
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestGetID_Methods(t *testing.T) {
	tests := []struct {
		name string
		elem interface {
			GetID() string
		}
		expected string
	}{
		{"System", &language.System{ID: "Sys1"}, "Sys1"},
		{"Container", &language.Container{ID: "Cont1"}, "Cont1"},
		{"Component", &language.Component{ID: "Comp1"}, "Comp1"},
		{"Person", &language.Person{ID: "User1"}, "User1"},
		{"DataStore", &language.DataStore{ID: "DB1"}, "DB1"},
		{"Queue", &language.Queue{ID: "Q1"}, "Q1"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.elem.GetID()
			if result != tt.expected {
				t.Errorf("Expected GetID() to return %q, got %q", tt.expected, result)
			}
		})
	}
}

func TestGetID_Empty(t *testing.T) {
	sys := &language.System{ID: ""}
	if sys.GetID() != "" {
		t.Error("GetID should return empty string for empty ID")
	}
}

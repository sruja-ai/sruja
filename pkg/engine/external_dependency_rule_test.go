// pkg/engine/external_dependency_rule_test.go
package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestExternalDependencyRule_ContainerToParentSystem(t *testing.T) {
	rule := &ExternalDependencyRule{}

	program := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{
				{
					ID:    "App",
					Label: "Application",
					Containers: []*language.Container{
						{
							ID:    "API",
							Label: "API",
						},
					},
				},
			},
			Relations: []*language.Relation{
				{
					From: "API",
					To:   "App",
					Verb: stringPtr("depends"),
				},
			},
		},
	}

	errors := rule.Validate(program)
	if len(errors) == 0 {
		t.Fatal("Expected validation error for container depending on parent system")
	}

	if errors[0].Message != "Element 'API' cannot depend on its parent 'App'. Dependencies must be external." {
		t.Errorf("Unexpected error message: %s", errors[0].Message)
	}
}

func TestExternalDependencyRule_ComponentToParentContainer(t *testing.T) {
	rule := &ExternalDependencyRule{}

	program := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{
				{
					ID:    "App",
					Label: "Application",
					Containers: []*language.Container{
						{
							ID:    "API",
							Label: "API",
							Components: []*language.Component{
								{
									ID:    "Controller",
									Label: "Controller",
								},
							},
						},
					},
				},
			},
			Relations: []*language.Relation{
				{
					From: "Controller",
					To:   "API",
					Verb: stringPtr("uses"),
				},
			},
		},
	}

	errors := rule.Validate(program)
	if len(errors) == 0 {
		t.Fatal("Expected validation error for component depending on parent container")
	}

	if errors[0].Message != "Element 'Controller' cannot depend on its parent 'API'. Dependencies must be external." {
		t.Errorf("Unexpected error message: %s", errors[0].Message)
	}
}

func TestExternalDependencyRule_ValidExternalDependency(t *testing.T) {
	rule := &ExternalDependencyRule{}

	program := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{
				{
					ID:    "App1",
					Label: "Application 1",
					Containers: []*language.Container{
						{
							ID:    "API1",
							Label: "API 1",
						},
					},
				},
				{
					ID:    "App2",
					Label: "Application 2",
					Containers: []*language.Container{
						{
							ID:    "API2",
							Label: "API 2",
						},
					},
				},
			},
			Relations: []*language.Relation{
				{
					From: "API1",
					To:   "API2",
					Verb: stringPtr("calls"),
				},
			},
		},
	}

	errors := rule.Validate(program)
	if len(errors) != 0 {
		t.Errorf("Expected no validation errors for valid external dependency, got: %v", errors)
	}
}

func TestExternalDependencyRule_SystemLevelRelation(t *testing.T) {
	rule := &ExternalDependencyRule{}

	program := &language.Program{
		Architecture: &language.Architecture{
			Systems: []*language.System{
				{
					ID:    "App",
					Label: "Application",
					Containers: []*language.Container{
						{
							ID:    "API",
							Label: "API",
						},
					},
					Relations: []*language.Relation{
						{
							From: "API",
							To:   "App",
							Verb: stringPtr("depends"),
						},
					},
				},
			},
		},
	}

	errors := rule.Validate(program)
	if len(errors) == 0 {
		t.Fatal("Expected validation error for container depending on parent system (system-level relation)")
	}
}

func stringPtr(s string) *string {
	return &s
}



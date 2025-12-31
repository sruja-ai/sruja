package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_Scale(t *testing.T) {
	dsl := `
		MySystem = system "System" {
			WebApp = container "Web App" {
				technology "Go"
				scale {
					min 3
					max 10
					metric "cpu > 80%"
				}
			}
			Worker = container "Worker" {
				scale {
					min 1
				}
			}
		}
	`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, _, err := parser.Parse("scale.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse DSL: %v", err)
	}

	// Find system and containers in Model
	var webApp, worker *language.Container
	for _, item := range program.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "system" && item.ElementDef.GetID() == "MySystem" {
			sysBody := item.ElementDef.GetBody()
			if sysBody != nil {
				for _, bodyItem := range sysBody.Items {
					if bodyItem.Element != nil && bodyItem.Element.GetKind() == "container" {
						if bodyItem.Element.GetID() == "WebApp" {
							webApp = &language.Container{ID: bodyItem.Element.GetID()}
							webAppBody := bodyItem.Element.GetBody()
							if webAppBody != nil {
								for _, contBodyItem := range webAppBody.Items {
									if contBodyItem.Scale != nil {
										webApp.Scale = contBodyItem.Scale
									}
								}
							}
						}
						if bodyItem.Element.GetID() == "Worker" {
							worker = &language.Container{ID: bodyItem.Element.GetID()}
							workerBody := bodyItem.Element.GetBody()
							if workerBody != nil {
								for _, contBodyItem := range workerBody.Items {
									if contBodyItem.Scale != nil {
										worker.Scale = contBodyItem.Scale
									}
								}
							}
						}
					}
				}
			}
			break
		}
	}

	if webApp.Scale == nil {
		t.Fatal("Expected WebApp to have a Scale block")
	}

	if *webApp.Scale.Min != 3 {
		t.Errorf("Expected min 3, got %d", *webApp.Scale.Min)
	}
	if *webApp.Scale.Max != 10 {
		t.Errorf("Expected max 10, got %d", *webApp.Scale.Max)
	}
	if *webApp.Scale.Metric != "cpu > 80%" {
		t.Errorf("Expected metric 'cpu > 80%%', got '%s'", *webApp.Scale.Metric)
	}

	if worker == nil {
		t.Fatal("Expected Worker container")
	}
	if worker.Scale == nil {
		t.Fatal("Expected Worker to have a Scale block")
	}
	if *worker.Scale.Min != 1 {
		t.Errorf("Expected min 1, got %d", *worker.Scale.Min)
	}
	if worker.Scale.Max != nil {
		t.Errorf("Expected max to be nil, got %d", *worker.Scale.Max)
	}
}

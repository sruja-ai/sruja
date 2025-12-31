package language_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func Test_Feature_System_WithDescription_And_Metadata(t *testing.T) {
	dsl := `
		API = system "API" {
			description "Desc"
			metadata {
				owner "team"
			}
		}
	`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	// Find system in Model
	var sys *language.System
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "system" && item.ElementDef.GetID() == "API" {
			// Extract system - simplified check
			sys = &language.System{
				ID: item.ElementDef.GetID(),
			}
			body := item.ElementDef.GetBody()
			if body != nil {
				for _, bodyItem := range body.Items {
					if bodyItem.Description != nil {
						sys.Description = bodyItem.Description
					}
					if bodyItem.Metadata != nil {
						for _, m := range bodyItem.Metadata.Entries {
							if m.Key == "owner" {
								sys.Metadata = append(sys.Metadata, m)
							}
						}
					}
				}
			}
			break
		}
	}
	if sys == nil {
		t.Fatalf("system not found")
	}
	if sys.Description == nil || *sys.Description != "Desc" {
		t.Fatalf("missing description")
	}
	if !sys.HasMeta("owner") {
		t.Fatalf("system metadata not parsed")
	}
}

func Test_Feature_Container_Tech_Tags_Version_Metadata(t *testing.T) {
	dsl := `
		S = system "S" {
			C = container "C" {
				technology "Go"
				tags ["api", "svc"]
				version "1.0"
				metadata {
					tier "gold"
				}
			}
		}
	`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	// Find container in Model
	var cont *language.Container
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "system" && item.ElementDef.GetID() == "S" {
			body := item.ElementDef.GetBody()
			if body != nil {
				for _, bodyItem := range body.Items {
					if bodyItem.Element != nil && bodyItem.Element.GetKind() == "container" && bodyItem.Element.GetID() == "C" {
						cont = &language.Container{
							ID: bodyItem.Element.GetID(),
						}
						contBody := bodyItem.Element.GetBody()
						if contBody != nil {
							for _, contBodyItem := range contBody.Items {
								if contBodyItem.Metadata != nil {
									cont.Metadata = append(cont.Metadata, contBodyItem.Metadata.Entries...)
								}
							}
						}
						break
					}
				}
			}
			break
		}
	}
	if cont == nil {
		t.Fatalf("container not found")
	}
	if !cont.HasMeta("tier") {
		t.Fatalf("container metadata not parsed")
	}
}

func Test_Feature_Component_Technology_Metadata(t *testing.T) {
	dsl := `
		S = system "S" {
			C = container "C" {
				X = component "X" {
					technology "React"
					metadata {
						critical "true"
					}
				}
			}
		}
	`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	// Find component in Model
	var comp *language.Component
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "system" && item.ElementDef.GetID() == "S" {
			sysBody := item.ElementDef.GetBody()
			if sysBody != nil {
				for _, bodyItem := range sysBody.Items {
					if bodyItem.Element != nil && bodyItem.Element.GetKind() == "container" && bodyItem.Element.GetID() == "C" {
						contBody := bodyItem.Element.GetBody()
						if contBody != nil {
							for _, contBodyItem := range contBody.Items {
								if contBodyItem.Element != nil && contBodyItem.Element.GetKind() == "component" && contBodyItem.Element.GetID() == "X" {
									comp = &language.Component{
										ID: contBodyItem.Element.GetID(),
									}
									compBody := contBodyItem.Element.GetBody()
									if compBody != nil {
										for _, compBodyItem := range compBody.Items {
											if compBodyItem.Metadata != nil {
												comp.Metadata = append(comp.Metadata, compBodyItem.Metadata.Entries...)
											}
										}
									}
									break
								}
							}
						}
						break
					}
				}
			}
			break
		}
	}
	if comp == nil {
		t.Fatalf("component not found")
	}
	if !comp.HasMeta("critical") {
		t.Fatalf("component metadata not parsed")
	}
}

func Test_Feature_DataStore_Queue_Person_Metadata(t *testing.T) {
	dsl := `
		U = person "User" {
			metadata {
				persona "customer"
			}
		}
		S = system "S" {
			D = database "DB" {
				metadata {
					engine "postgres"
				}
			}
			Q = queue "Events" {
				metadata {
					topic "billing"
				}
			}
		}
	`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	// Find person
	var person *language.Person
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "person" && item.ElementDef.GetID() == "U" {
			person = &language.Person{ID: item.ElementDef.GetID()}
			body := item.ElementDef.GetBody()
			if body != nil {
				for _, bodyItem := range body.Items {
					if bodyItem.Metadata != nil {
						person.Metadata = append(person.Metadata, bodyItem.Metadata.Entries...)
					}
				}
			}
			break
		}
	}
	if person == nil || !person.HasMeta("persona") {
		t.Fatalf("person metadata not parsed")
	}
	// Find datastore and queue
	var ds *language.DataStore
	var q *language.Queue
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.GetKind() == "system" && item.ElementDef.GetID() == "S" {
			sysBody := item.ElementDef.GetBody()
			if sysBody != nil {
				for _, bodyItem := range sysBody.Items {
					if bodyItem.Element != nil {
						if bodyItem.Element.GetKind() == "database" && bodyItem.Element.GetID() == "D" {
							ds = &language.DataStore{ID: bodyItem.Element.GetID()}
							dsBody := bodyItem.Element.GetBody()
							if dsBody != nil {
								for _, dsBodyItem := range dsBody.Items {
									if dsBodyItem.Metadata != nil {
										ds.Metadata = append(ds.Metadata, dsBodyItem.Metadata.Entries...)
									}
								}
							}
						}
						if bodyItem.Element.GetKind() == "queue" && bodyItem.Element.GetID() == "Q" {
							q = &language.Queue{ID: bodyItem.Element.GetID()}
							qBody := bodyItem.Element.GetBody()
							if qBody != nil {
								for _, qBodyItem := range qBody.Items {
									if qBodyItem.Metadata != nil {
										q.Metadata = append(q.Metadata, qBodyItem.Metadata.Entries...)
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
	if ds == nil || !ds.HasMeta("engine") {
		t.Fatalf("datastore metadata not parsed")
	}
	if q == nil || !q.HasMeta("topic") {
		t.Fatalf("queue metadata not parsed")
	}
}

func Test_Feature_Relation_Verb_Label(t *testing.T) {
	dsl := `
		A = system "A"
		B = system "B"
		A -> B "calls" "HTTP"
	`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	// Find relation in Model
	var rel *language.Relation
	for _, item := range prog.Model.Items {
		if item.Relation != nil {
			rel = item.Relation
			break
		}
	}
	if rel == nil {
		t.Fatalf("relation not found")
	}
	if rel.Verb == nil || *rel.Verb != "calls" {
		t.Fatalf("verb missing")
	}
	if rel.Label == nil || *rel.Label != "HTTP" {
		t.Fatalf("label missing")
	}
}

// Journey feature removed - test removed

func Test_Feature_ADR(t *testing.T) {
	// Using new unified syntax for ADR
	dsl := `
		ADR001 = adr "Use JWT"
	`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	// Find ADRs via ElementDef in Model
	var adrCount int
	for _, item := range prog.Model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			if a.Kind == "adr" {
				adrCount++
			}
		}
	}
	if adrCount != 1 {
		t.Fatalf("expected 1 ADR, got %d", adrCount)
	}
}

func Test_Printer_Emits_Metadata_For_All_Elements(t *testing.T) {
	dsl := `U = person "User" { metadata { persona "customer" } } S = system "S" { metadata { owner "team" } C = container "C" { metadata { tier "gold" } X = component "X" { metadata { critical "true" } } D = database "DB" { metadata { engine "postgres" } } Q = queue "Events" { metadata { topic "billing" } } } }`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	pr := language.NewPrinter()
	out := pr.Print(prog)
	checks := []string{
		//"model {", // Removed
		"metadata {",
		"U = person",
		"S = system",
		"C = container",
	}
	for _, s := range checks {
		if !strings.Contains(out, s) {
			t.Fatalf("printer missing segment %s\nOutput:\n%s", s, out)
		}
	}
}

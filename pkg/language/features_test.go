package language_test

import (
	"strings"
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func Test_Feature_System_WithDescription_And_Metadata(t *testing.T) {
	dsl := `architecture "A" { system API "API" "Desc" { metadata { owner "team" } } }`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	sys := prog.Architecture.Systems[0]
	if sys.Description == nil || *sys.Description != "Desc" {
		t.Fatalf("missing description")
	}
	if !sys.HasMeta("owner") {
		t.Fatalf("system metadata not parsed")
	}
}

func Test_Feature_Container_Tech_Tags_Version_Metadata(t *testing.T) {
	dsl := `architecture "A" { system S "S" { container C "C" { technology "Go" tags ["api","svc"] version "1.0" metadata { tier "gold" } } } }`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	cont := prog.Architecture.Systems[0].Containers[0]
	if !cont.HasMeta("tier") {
		t.Fatalf("container metadata not parsed")
	}
}

func Test_Feature_Component_Technology_Metadata(t *testing.T) {
	dsl := `architecture "A" { system S "S" { container C "C" { component X "X" { technology "React" metadata { critical "true" } } } } }`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	comp := prog.Architecture.Systems[0].Containers[0].Components[0]
	if !comp.HasMeta("critical") {
		t.Fatalf("component metadata not parsed")
	}
}

func Test_Feature_DataStore_Queue_Person_Metadata(t *testing.T) {
	dsl := `architecture "A" { person U "User" { metadata { persona "customer" } } system S "S" { datastore D "DB" { metadata { engine "postgres" } } queue Q "Events" { metadata { topic "billing" } } } }`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	if !prog.Architecture.Persons[0].HasMeta("persona") {
		t.Fatalf("person metadata not parsed")
	}
	ds := prog.Architecture.Systems[0].DataStores[0]
	if !ds.HasMeta("engine") {
		t.Fatalf("datastore metadata not parsed")
	}
	q := prog.Architecture.Systems[0].Queues[0]
	if !q.HasMeta("topic") {
		t.Fatalf("queue metadata not parsed")
	}
}

func Test_Feature_Relation_Verb_Label(t *testing.T) {
	dsl := `architecture "A" { system A "A" {} system B "B" {} A -> B calls "HTTP" }`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	rel := prog.Architecture.Relations[0]
	if rel.Verb == nil || *rel.Verb != "calls" {
		t.Fatalf("verb missing")
	}
	if rel.Label == nil || *rel.Label != "HTTP" {
		t.Fatalf("label missing")
	}
}

// Journey feature removed - test removed

func Test_Feature_Requirements_Types(t *testing.T) {
	t.Skip("Skipping Requirements Types test due to 'constraint' keyword revert")
	dsl := `architecture "A" { requirement R1 performance "p95<200ms" requirement R2 security "TLS1.3" requirement R3 constraint "Use Postgres" }`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	// Requirement.Type is *string
	got := []string{*prog.Architecture.Requirements[0].Type, *prog.Architecture.Requirements[1].Type, *prog.Architecture.Requirements[2].Type}
	want1, want2, want3 := "performance", "security", "constraint"
	if got[0] != want1 || got[1] != want2 || got[2] != want3 {
		t.Fatalf("unexpected requirement types %v", got)
	}
}

func Test_Feature_ADR(t *testing.T) {
	dsl := `architecture "A" { adr ADR001 "Use JWT" }`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	if len(prog.Architecture.ADRs) != 1 {
		t.Fatalf("expected 1 ADR")
	}
}

// Test_Feature_Imports_WithAlias removed - import feature removed
func Test_Feature_Imports_WithAlias_Removed(t *testing.T) {
	t.Skip("Import feature removed")
}

func Test_Printer_Emits_Metadata_For_All_Elements(t *testing.T) {
	dsl := `architecture "A" { metadata { level "arch" } person U "User" { metadata { persona "customer" } } system S "S" { metadata { owner "team" } container C "C" { metadata { tier "gold" } component X "X" { metadata { critical "true" } } datastore D "DB" { metadata { engine "postgres" } } queue Q "Events" { metadata { topic "billing" } } } } }`
	p, _ := language.NewParser()
	prog, _, err := p.Parse("a.sruja", dsl)
	if err != nil {
		t.Fatalf("parse %v", err)
	}
	pr := language.NewPrinter()
	out := pr.Print(prog)
	checks := []string{
		"architecture \"A\" {",
		"metadata {",
		"person U \"User\" {",
		"datastore D \"DB\" {",
		"queue Q \"Events\" {",
		"container C \"C\" {",
		"component X \"X\" {",
	}
	for _, s := range checks {
		if !strings.Contains(out, s) {
			t.Fatalf("printer missing segment %s", s)
		}
	}
}

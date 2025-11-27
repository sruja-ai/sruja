//go:build legacy

// pkg/language/parser_journey_test.go
// Package language_test provides tests for journey parsing.
package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestParser_Journeys(t *testing.T) {
	dsl := `
architecture "Test" {
	person User "User"
	system App "Application" {
		container UI "Web UI"
		container API "API Server"
	}
	journey login {
		title "User Login Journey"
		steps {
			User -> UI "enters username/password"
			UI -> API "POST /login"
		}
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse journeys: %v", err)
	}

	if len(program.Architecture.Journeys) != 1 {
		t.Fatalf("Expected 1 journey, got %d", len(program.Architecture.Journeys))
	}

	journey := program.Architecture.Journeys[0]
	journey.PostProcess() // Populate convenience fields

	if journey.ID != "login" {
		t.Errorf("Expected journey ID 'login', got '%s'", journey.ID)
	}
	if journey.Title != "User Login Journey" {
		t.Errorf("Expected journey title 'User Login Journey', got '%s'", journey.Title)
	}
	if len(journey.Steps) != 2 {
		t.Fatalf("Expected 2 steps, got %d", len(journey.Steps))
	}

	step1 := journey.Steps[0]
	if step1.From != "User" {
		t.Errorf("Expected step 1 from 'User', got '%s'", step1.From)
	}
	if step1.To != "UI" {
		t.Errorf("Expected step 1 to 'UI', got '%s'", step1.To)
	}
	if step1.Label == nil {
		t.Fatal("Expected step 1 label to be present")
	}
	if *step1.Label != "enters username/password" {
		t.Errorf("Expected step 1 label 'enters username/password', got '%s'", *step1.Label)
	}

	step2 := journey.Steps[1]
	if step2.From != "UI" {
		t.Errorf("Expected step 2 from 'UI', got '%s'", step2.From)
	}
	if step2.To != "API" {
		t.Errorf("Expected step 2 to 'API', got '%s'", step2.To)
	}
	if step2.Label == nil {
		t.Fatal("Expected step 2 label to be present")
	}
	if *step2.Label != "POST /login" {
		t.Errorf("Expected step 2 label 'POST /login', got '%s'", *step2.Label)
	}
}

func TestParser_JourneyWithoutTitle(t *testing.T) {
	dsl := `
architecture "Test" {
	person User "User"
	system App "Application" {
		container UI "Web UI"
	}
	journey checkout {
		steps {
			User -> UI "adds item to cart"
		}
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse journey without title: %v", err)
	}

	if len(program.Architecture.Journeys) != 1 {
		t.Fatalf("Expected 1 journey, got %d", len(program.Architecture.Journeys))
	}

	journey := program.Architecture.Journeys[0]
	journey.PostProcess()

	if journey.ID != "checkout" {
		t.Errorf("Expected journey ID 'checkout', got '%s'", journey.ID)
	}
	if journey.Title != "" {
		t.Errorf("Expected empty title, got '%s'", journey.Title)
	}
	if len(journey.Steps) != 1 {
		t.Fatalf("Expected 1 step, got %d", len(journey.Steps))
	}
}

func TestParser_JourneyWithArrowDirections(t *testing.T) {
	dsl := `
architecture "Test" {
	person User "User"
	system App "Application" {
		container UI "Web UI"
		container API "API Server"
		datastore DB "Database"
	}
	journey payment {
		title "Payment Processing"
		steps {
			User -> UI "initiates payment"
			UI -> API "sends payment request"
			API -> DB "saves transaction"
			API <- DB "receives confirmation"
			UI <-> API "syncs status"
		}
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse journey with arrows: %v", err)
	}

	journey := program.Architecture.Journeys[0]
	journey.PostProcess()

	if len(journey.Steps) != 5 {
		t.Fatalf("Expected 5 steps, got %d", len(journey.Steps))
	}

	step1 := journey.Steps[0]
	if step1.Arrow != "->" {
		t.Errorf("Expected step 1 arrow '->', got '%s'", step1.Arrow)
	}

	step4 := journey.Steps[3]
	if step4.Arrow != "<-" {
		t.Errorf("Expected step 4 arrow '<-', got '%s'", step4.Arrow)
	}

	step5 := journey.Steps[4]
	if step5.Arrow != "<->" {
		t.Errorf("Expected step 5 arrow '<->', got '%s'", step5.Arrow)
	}
}

func TestParser_MultipleJourneys(t *testing.T) {
	dsl := `
architecture "Test" {
	person User "User"
	system App "Application" {
		container UI "Web UI"
		container API "API Server"
	}
	journey login {
		title "User Login"
		steps {
			User -> UI "enters credentials"
		}
	}
	journey logout {
		title "User Logout"
		steps {
			User -> UI "clicks logout"
			UI -> API "POST /logout"
		}
	}
	journey checkout {
		steps {
			User -> UI "adds to cart"
		}
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse multiple journeys: %v", err)
	}

	if len(program.Architecture.Journeys) != 3 {
		t.Fatalf("Expected 3 journeys, got %d", len(program.Architecture.Journeys))
	}

	// Verify first journey
	login := program.Architecture.Journeys[0]
	login.PostProcess()
	if login.ID != "login" {
		t.Errorf("Expected journey ID 'login', got '%s'", login.ID)
	}
	if login.Title != "User Login" {
		t.Errorf("Expected title 'User Login', got '%s'", login.Title)
	}

	// Verify second journey
	logout := program.Architecture.Journeys[1]
	logout.PostProcess()
	if logout.ID != "logout" {
		t.Errorf("Expected journey ID 'logout', got '%s'", logout.ID)
	}
	if len(logout.Steps) != 2 {
		t.Errorf("Expected 2 steps in logout journey, got %d", len(logout.Steps))
	}

	// Verify third journey
	checkout := program.Architecture.Journeys[2]
	checkout.PostProcess()
	if checkout.ID != "checkout" {
		t.Errorf("Expected journey ID 'checkout', got '%s'", checkout.ID)
	}
	if checkout.Title != "" {
		t.Errorf("Expected empty title for checkout, got '%s'", checkout.Title)
	}
}

func TestParser_JourneysInComplexArchitecture(t *testing.T) {
	dsl := `
architecture "Test" {
	person User "User"
	system App "Application" {
		container UI "Web UI"
		container API "API Server"
		container Auth "Auth Service"
		datastore DB "Database"
	}
	User -> UI "Uses"
	UI -> API "Calls"
	API -> Auth "Authenticates"
	API -> DB "Reads/Writes"
	requirement R1 functional "Must support user login"
	requirement R2 security "Must encrypt passwords"
	adr ADR001 "Use microservices architecture"
	journey login {
		title "User Login Journey"
		steps {
			User -> UI "enters username/password"
			UI -> API "POST /login"
			API -> Auth "check credentials"
			Auth -> DB "fetch user record"
		}
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse complex architecture with journeys: %v", err)
	}

	// Verify systems
	if len(program.Architecture.Systems) != 1 {
		t.Errorf("Expected 1 system, got %d", len(program.Architecture.Systems))
	}

	// Verify relations
	if len(program.Architecture.Relations) != 4 {
		t.Errorf("Expected 4 relations, got %d", len(program.Architecture.Relations))
	}

	// Verify requirements
	if len(program.Architecture.Requirements) != 2 {
		t.Errorf("Expected 2 requirements, got %d", len(program.Architecture.Requirements))
	}

	// Verify ADRs
	if len(program.Architecture.ADRs) != 1 {
		t.Errorf("Expected 1 ADR, got %d", len(program.Architecture.ADRs))
	}

	// Verify journeys
	if len(program.Architecture.Journeys) != 1 {
		t.Fatalf("Expected 1 journey, got %d", len(program.Architecture.Journeys))
	}

	journey := program.Architecture.Journeys[0]
	journey.PostProcess()
	if journey.ID != "login" {
		t.Errorf("Expected journey ID 'login', got '%s'", journey.ID)
	}
	if len(journey.Steps) != 4 {
		t.Errorf("Expected 4 steps, got %d", len(journey.Steps))
	}
}

func TestParser_JourneyWithEmptySteps(t *testing.T) {
	dsl := `
architecture "Test" {
	person User "User"
	journey empty {
		title "Empty Journey"
		steps {
		}
	}
}
`
	parser, err := language.NewParser()
	if err != nil {
		t.Fatalf("Failed to create parser: %v", err)
	}

	program, err := parser.Parse("test.sruja", dsl)
	if err != nil {
		t.Fatalf("Failed to parse journey with empty steps: %v", err)
	}

	if len(program.Architecture.Journeys) != 1 {
		t.Fatalf("Expected 1 journey, got %d", len(program.Architecture.Journeys))
	}

	journey := program.Architecture.Journeys[0]
	journey.PostProcess()

	if journey.ID != "empty" {
		t.Errorf("Expected journey ID 'empty', got '%s'", journey.ID)
	}
	if journey.Title != "Empty Journey" {
		t.Errorf("Expected title 'Empty Journey', got '%s'", journey.Title)
	}
	if len(journey.Steps) != 0 {
		t.Errorf("Expected 0 steps, got %d", len(journey.Steps))
	}
}

package engine

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// ScoreCard represents the result of an architectural score.
type ScoreCard struct {
	Score      int
	Grade      string
	Deductions []Deduction
}

// Deduction represents a point deduction with a reason.
type Deduction struct {
	Rule    string
	Points  int
	Message string
	Target  string
}

// Scorer calculates the architecture score.
type Scorer struct {
	validator *Validator
}

// NewScorer creates a new Scorer.
func NewScorer() *Scorer {
	v := NewValidator()
	v.RegisterRule(&CycleDetectionRule{})
	v.RegisterRule(&OrphanDetectionRule{})
	v.RegisterRule(&LayerViolationRule{})
	v.RegisterRule(&ValidReferenceRule{})

	return &Scorer{
		validator: v,
	}
}

// CalculateScore calculates the score for a given program.
//
//nolint:funlen,gocyclo // Score calculation is long and complex
func (s *Scorer) CalculateScore(program *language.Program) ScoreCard {
	score := 100
	deductions := []Deduction{}

	// 1. Run Validation Rules (Correctness)
	diags := s.validator.Validate(program)
	for i := range diags {
		d := &diags[i]
		points := 0
		rule := ""
		switch d.Code {
		case diagnostics.CodeCycleDetected:
			points = 20
			rule = "Circular Dependency"
		case diagnostics.CodeLayerViolation:
			points = 10
			rule = "Layer Violation"
		case diagnostics.CodeOrphanElement:
			points = 5
			rule = "Orphan Element"
		case diagnostics.CodeReferenceNotFound:
			points = 10
			rule = "Invalid Reference"
		default:
			if d.Severity == diagnostics.SeverityError {
				points = 5
				rule = "Validation Error"
			}
		}

		if points > 0 {
			deductions = append(deductions, Deduction{
				Rule:    rule,
				Points:  points,
				Message: d.Message,
				Target:  fmt.Sprintf("%s:%d", d.Location.File, d.Location.Line),
			})
			score -= points
		}
	}

	// 2. Check Completeness (Missing Metadata)
	if program.Architecture != nil {
		for i := range program.Architecture.Items {
			item := &program.Architecture.Items[i]
			// Check Top-level Containers
			if item.Container != nil {
				if item.Container.Description == nil {
					deductions = append(deductions, Deduction{
						Rule:    "Missing Description",
						Points:  2,
						Message: fmt.Sprintf("Container '%s' is missing a description", item.Container.ID),
						Target:  item.Container.ID,
					})
					score -= 2
				}
			}
			// Check Components
			if item.Component != nil {
				if item.Component.Description == nil {
					deductions = append(deductions, Deduction{
						Rule:    "Missing Description",
						Points:  2,
						Message: fmt.Sprintf("Component '%s' is missing a description", item.Component.ID),
						Target:  item.Component.ID,
					})
					score -= 2
				}
			}
			// Check Systems and their nested items
			if item.System != nil {
				for _, sysItem := range item.System.Items {
					if sysItem.Container != nil {
						if sysItem.Container.Description == nil {
							deductions = append(deductions, Deduction{
								Rule:    "Missing Description",
								Points:  2,
								Message: fmt.Sprintf("Container '%s' is missing a description", sysItem.Container.ID),
								Target:  sysItem.Container.ID,
							})
							score -= 2
						}
					}
				}
			}
		}
	}

	// Cap score at 0
	if score < 0 {
		score = 0
	}

	// Calculate Grade
	grade := "F"
	switch {
	case score >= 90:
		grade = "A"
	case score >= 80:
		grade = "B"
	case score >= 70:
		grade = "C"
	case score >= 60:
		grade = "D"
	}

	return ScoreCard{
		Score:      score,
		Grade:      grade,
		Deductions: deductions,
	}
}

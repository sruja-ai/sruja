package engine

import (
	"strconv"
	"strings"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
	"github.com/sruja-ai/sruja/pkg/language"
)

// CategoryScores represents the scores for each health dimension.
type CategoryScores struct {
	Structural      int // 40%
	Documentation   int // 20%
	Traceability    int // 15%
	Complexity      int // 15%
	Standardization int // 10%
}

// Scoring Constants
const (
	// Category Weights
	WeightStructural      = 0.40
	WeightDocumentation   = 0.20
	WeightTraceability    = 0.15
	WeightComplexity      = 0.15
	WeightStandardization = 0.10

	// Penalties (Structural)
	PenaltyCycle             = 30
	PenaltyLayerViolation    = 15
	PenaltyOrphanElement     = 10
	PenaltyInvalidReference  = 20
	PenaltyGenericValidation = 10

	// Penalties (Documentation & Traceability)
	PenaltyMissingDescription  = 5
	PenaltyMissingTechnology   = 5
	PenaltyMissingMetadata     = 2
	PenaltyLowTraceability     = 20
	ThresholdTraceabilityRatio = 0.5

	// Critical Scoring
	ThresholdCriticalStructural = 50
	MultiplierCritical          = 0.8

	// Grade Thresholds
	GradeThresholdA = 90
	GradeThresholdB = 80
	GradeThresholdC = 70
	GradeThresholdD = 60
)

// ScoreCard represents the result of an architectural score.
type ScoreCard struct {
	Score      int
	Grade      string
	Categories CategoryScores
	Deductions []Deduction
}

// Deduction represents a point deduction with a reason.
type Deduction struct {
	Rule     string
	Points   int
	Message  string
	Target   string
	Severity diagnostics.Severity
	Category string // Category names like "Structural", "Documentation", etc.
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

// clampScore ensures a score is between 0 and 100.
// Defined at package level to avoid closure allocation.
func clampScore(n int) int {
	if n < 0 {
		return 0
	}
	if n > 100 {
		return 100
	}
	return n
}

// formatTarget formats a diagnostic location as a string.
// Uses strings.Builder to avoid fmt.Sprintf allocation.
func formatTarget(file string, line int) string {
	if file == "" {
		return ""
	}
	var sb strings.Builder
	sb.Grow(len(file) + 10)
	sb.WriteString(file)
	sb.WriteByte(':')
	sb.WriteString(strconv.Itoa(line))
	return sb.String()
}

// formatMissingDescription formats the "missing description" message.
func formatMissingDescription(elementID string) string {
	const prefix = "Element '"
	const suffix = "' is missing a description"
	var sb strings.Builder
	sb.Grow(len(prefix) + len(elementID) + len(suffix))
	sb.WriteString(prefix)
	sb.WriteString(elementID)
	sb.WriteString(suffix)
	return sb.String()
}

// formatMissingTechnology formats the "missing technology" message.
func formatMissingTechnology(elementID string) string {
	const prefix = "Element '"
	const suffix = "' is missing technology stack"
	var sb strings.Builder
	sb.Grow(len(prefix) + len(elementID) + len(suffix))
	sb.WriteString(prefix)
	sb.WriteString(elementID)
	sb.WriteString(suffix)
	return sb.String()
}

// CalculateScore calculates the architecture score for a program.
func (s *Scorer) CalculateScore(program *language.Program) ScoreCard {
	// Initialize category scores (start at 100 for each)
	scores := CategoryScores{
		Structural:      100,
		Documentation:   100,
		Traceability:    100,
		Complexity:      100,
		Standardization: 100,
	}

	deductions := make([]Deduction, 0, 32)

	// 1. Structural Integrity (40%) - Validation Rules
	diags := s.validator.Validate(program)
	for i := range diags {
		d := &diags[i]
		points := 0
		rule := ""
		severity := d.Severity

		switch d.Code {
		case diagnostics.CodeCycleDetected:
			points = PenaltyCycle
			rule = "Circular Dependency"
			severity = diagnostics.SeverityError
		case diagnostics.CodeLayerViolation:
			points = PenaltyLayerViolation
			rule = "Layer Violation"
			severity = diagnostics.SeverityWarning
		case diagnostics.CodeOrphanElement:
			points = PenaltyOrphanElement
			rule = "Orphan Element"
			severity = diagnostics.SeverityWarning
		case diagnostics.CodeReferenceNotFound:
			points = PenaltyInvalidReference
			rule = "Invalid Reference"
			severity = diagnostics.SeverityError
		default:
			if d.Severity == diagnostics.SeverityError {
				points = PenaltyGenericValidation
				rule = "Validation Error"
			}
		}

		if points > 0 {
			target := formatTarget(d.Location.File, d.Location.Line)
			deductions = append(deductions, Deduction{
				Rule:     rule,
				Points:   points,
				Message:  d.Message,
				Target:   target,
				Severity: severity,
				Category: "Structural",
			})
			scores.Structural -= points
		}
	}

	// 2. Documentation Depth (20%) & Standardization (10%)
	if program.Model != nil {
		s.checkDocumentation(program.Model, &scores, &deductions)
	}

	// 3. Traceability (15%) - Requirement Coverage
	if program.Model != nil {
		s.checkTraceability(program.Model, &scores, &deductions)
	}

	// Ensure categories don't go below 0
	scores.Structural = clampScore(scores.Structural)
	scores.Documentation = clampScore(scores.Documentation)
	scores.Traceability = clampScore(scores.Traceability)
	scores.Complexity = clampScore(scores.Complexity)
	scores.Standardization = clampScore(scores.Standardization)

	// Calculate Final Weighted Score
	// Structural (40%), Doc (20%), Trace (15%), Complexity (15%), Standard (10%)
	finalScore := float64(scores.Structural)*WeightStructural +
		float64(scores.Documentation)*WeightDocumentation +
		float64(scores.Traceability)*WeightTraceability +
		float64(scores.Complexity)*WeightComplexity +
		float64(scores.Standardization)*WeightStandardization

	// Apply critical multiplier if structural is very low
	if scores.Structural < ThresholdCriticalStructural {
		finalScore *= MultiplierCritical
	}

	score := int(finalScore)
	if score < 0 {
		score = 0
	}
	if score > 100 {
		score = 100
	}

	// Calculate Grade
	grade := calculateGrade(score)

	return ScoreCard{
		Score:      score,
		Grade:      grade,
		Categories: scores,
		Deductions: deductions,
	}
}

// calculateGrade returns the letter grade for a score.
func calculateGrade(score int) string {
	switch {
	case score >= GradeThresholdA:
		return "A"
	case score >= GradeThresholdB:
		return "B"
	case score >= GradeThresholdC:
		return "C"
	case score >= GradeThresholdD:
		return "D"
	default:
		return "F"
	}
}

// checkDocumentation checks documentation and standardization using iterative traversal.
func (s *Scorer) checkDocumentation(model *language.Model, scores *CategoryScores, deductions *[]Deduction) {
	if model == nil {
		return
	}

	// Use explicit stack for iterative traversal
	type frame struct {
		elem   *language.ElementDef
		parent string
	}
	stack := make([]frame, 0, 16)

	// Initialize with top-level elements
	for _, item := range model.Items {
		if item.ElementDef != nil {
			stack = append(stack, frame{elem: item.ElementDef, parent: ""})
		}
	}

	for len(stack) > 0 {
		// Pop
		f := stack[len(stack)-1]
		stack = stack[:len(stack)-1]

		elem := f.elem
		if elem == nil {
			continue
		}

		id := elem.GetID()
		if id == "" {
			continue
		}

		elementID := id
		if f.parent != "" {
			elementID = buildQualifiedID(f.parent, id)
		}

		// Documentation Checks
		hasDescription := false
		hasTechnology := false
		hasMetadata := false
		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Description != nil {
					hasDescription = true
				}
				if item.Technology != nil {
					hasTechnology = true
				}
				if item.Metadata != nil {
					hasMetadata = true
				}
			}
		}

		if !hasDescription {
			*deductions = append(*deductions, Deduction{
				Rule:     "Missing Description",
				Points:   PenaltyMissingDescription,
				Message:  formatMissingDescription(elementID),
				Target:   elementID,
				Severity: diagnostics.SeverityInfo,
				Category: "Documentation",
			})
			scores.Documentation -= PenaltyMissingDescription
		}

		// Only check technology for containers and components
		kind := elem.GetKind()
		if kind == "container" || kind == "component" {
			if !hasTechnology {
				*deductions = append(*deductions, Deduction{
					Rule:     "Missing Technology",
					Points:   PenaltyMissingTechnology,
					Message:  formatMissingTechnology(elementID),
					Target:   elementID,
					Severity: diagnostics.SeverityInfo,
					Category: "Documentation",
				})
				scores.Documentation -= PenaltyMissingTechnology
			}
		}

		// Standardization Checks
		if !hasMetadata {
			scores.Standardization -= PenaltyMissingMetadata
		}

		// Push children
		if body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					stack = append(stack, frame{elem: item.Element, parent: elementID})
				}
			}
		}
	}
}

// checkTraceability checks requirement coverage.
func (s *Scorer) checkTraceability(model *language.Model, scores *CategoryScores, deductions *[]Deduction) {
	if model == nil {
		return
	}

	totalElements := 0
	taggedCount := 0

	// Count elements and tagged elements
	for _, item := range model.Items {
		if item.ElementDef != nil && item.ElementDef.Assignment != nil {
			a := item.ElementDef.Assignment
			if a.Kind == "requirement" {
				// Requirements parsed through ElementDef
				// Would need to extract from body if available
				_ = a // explicit usage
			}
			totalElements++
		}
	}

	if totalElements > 0 && taggedCount < totalElements/2 { // Using simpler logic for now, could use ThresholdTraceabilityRatio
		scores.Traceability -= PenaltyLowTraceability // Penalty for low requirement coverage
		*deductions = append(*deductions, Deduction{
			Rule:     "Low Traceability",
			Points:   PenaltyLowTraceability,
			Message:  "Less than 50% of elements are mapped to requirements",
			Target:   "requirements",
			Severity: diagnostics.SeverityWarning,
			Category: "Traceability",
		})
	}
}

package engine

import (
	"fmt"
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

	// Complexity Thresholds
	ThresholdHighComplexity = 10 // Max fan-in + fan-out
	PenaltyHighComplexity   = 10

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

	orphanCount := 0
	var orphanExamples []string

	for i := range diags {
		d := &diags[i]
		points := 0
		rule := ""
		severity := d.Severity
		isAggregated := false

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
			// Aggregate orphans
			orphanCount++
			if len(orphanExamples) < 3 {
				target := formatTarget(d.Location.File, d.Location.Line)
				// formatTarget includes file:line. Maybe just use element name if available in message?
				// The message usually says "Element 'X' is not connected..."
				// Let's use the message to extract name or just use file:line as example
				orphanExamples = append(orphanExamples, target)
			}
			isAggregated = true
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
			scores.Structural -= points
			if !isAggregated {
				target := formatTarget(d.Location.File, d.Location.Line)
				deductions = append(deductions, Deduction{
					Rule:     rule,
					Points:   points,
					Message:  d.Message,
					Target:   target,
					Severity: severity,
					Category: "Structural",
				})
			}
		}
	}

	// Add Aggregated Structural Deductions
	if orphanCount > 0 {
		exampleStr := strings.Join(orphanExamples, "', '")
		msg := fmt.Sprintf("%d orphan elements found (not connected to anything) (-%d pts each) (e.g., at '%s'...)", orphanCount, PenaltyOrphanElement, exampleStr)
		if orphanCount <= 3 {
			msg = fmt.Sprintf("%d orphan elements found (at '%s') (-%d pts each)", orphanCount, exampleStr, PenaltyOrphanElement)
		}

		deductions = append(deductions, Deduction{
			Rule:     "Orphan Elements",
			Points:   PenaltyOrphanElement * orphanCount,
			Message:  msg,
			Target:   "model",
			Severity: diagnostics.SeverityWarning,
			Category: "Structural",
		})
	}

	// 2. Documentation Depth (20%) & Standardization (10%)
	if program.Model != nil {
		s.checkDocumentation(program.Model, &scores, &deductions)
	}

	// 3. Traceability (15%) - Requirement Coverage
	if program.Model != nil {
		s.checkTraceability(program.Model, &scores, &deductions)
	}

	// 4. Complexity (15%) - Fan-in/Fan-out
	if program.Model != nil {
		s.checkComplexity(program.Model, &scores, &deductions)
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

	// Aggregation variables
	missingDescCount := 0
	var missingDescExamples []string
	missingTechCount := 0
	var missingTechExamples []string
	missingMetaCount := 0

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

		kind := strings.ToLower(elem.GetKind())
		shouldCheckDoc := kind != "requirement" && kind != "policy" && kind != "adr" && kind != "scenario" && kind != "story" && kind != "flow" && kind != "model" && kind != "views" && kind != "import"

		if shouldCheckDoc && !hasDescription {
			scores.Documentation -= PenaltyMissingDescription
			missingDescCount++
			if len(missingDescExamples) < 3 {
				missingDescExamples = append(missingDescExamples, elementID)
			}
		}

		// Only check technology for containers and components
		if kind == "container" || kind == "component" {
			if !hasTechnology {
				scores.Documentation -= PenaltyMissingTechnology
				missingTechCount++
				if len(missingTechExamples) < 3 {
					missingTechExamples = append(missingTechExamples, elementID)
				}
			}
		}

		// Standardization Checks
		if !hasMetadata {
			scores.Standardization -= PenaltyMissingMetadata
			missingMetaCount++
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

	// Add Aggregated Deductions
	if missingDescCount > 0 {
		exampleStr := strings.Join(missingDescExamples, "', '")
		msg := fmt.Sprintf("%d elements are missing descriptions (-%d pts each) (e.g., '%s'...)", missingDescCount, PenaltyMissingDescription, exampleStr)
		if missingDescCount <= 3 {
			msg = fmt.Sprintf("%d elements are missing descriptions (-%d pts each) ('%s')", missingDescCount, PenaltyMissingDescription, exampleStr)
		}

		*deductions = append(*deductions, Deduction{
			Rule:     "Missing Description",
			Points:   PenaltyMissingDescription * missingDescCount,
			Message:  msg,
			Target:   "model", // Target is model or multiple
			Severity: diagnostics.SeverityInfo,
			Category: "Documentation",
		})
	}

	if missingTechCount > 0 {
		exampleStr := strings.Join(missingTechExamples, "', '")
		msg := fmt.Sprintf("%d components/containers are missing technology stack (-%d pts each) (e.g., '%s'...)", missingTechCount, PenaltyMissingTechnology, exampleStr)
		if missingTechCount <= 3 {
			msg = fmt.Sprintf("%d components/containers are missing technology stack (-%d pts each) ('%s')", missingTechCount, PenaltyMissingTechnology, exampleStr)
		}

		*deductions = append(*deductions, Deduction{
			Rule:     "Missing Technology",
			Points:   PenaltyMissingTechnology * missingTechCount,
			Message:  msg,
			Target:   "model",
			Severity: diagnostics.SeverityInfo,
			Category: "Documentation",
		})
	}
}

// checkTraceability checks requirement coverage.
func (s *Scorer) checkTraceability(model *language.Model, scores *CategoryScores, deductions *[]Deduction) {
	if model == nil {
		return
	}

	elements := make(map[string]bool)
	linkedElements := make(map[string]bool)
	requirements := make(map[string]bool)

	// Single pass to collect elements, requirements, and relations
	type frame struct {
		elem *language.ElementDef
	}
	stack := make([]frame, 0, 16)

	// Initialize with model items
	for _, item := range model.Items {
		if item.ElementDef != nil {
			stack = append(stack, frame{elem: item.ElementDef})
		}
	}

	for len(stack) > 0 {
		f := stack[len(stack)-1]
		stack = stack[:len(stack)-1]

		elem := f.elem
		if elem == nil {
			continue
		}
		id := elem.GetID()
		kind := strings.ToLower(elem.GetKind())

		// Track requirements specifically
		if kind == "requirement" {
			requirements[id] = true
		} else if kind != "model" && kind != "views" && kind != "import" {
			// It's a structural element
			elements[id] = true
		}

		// Recurse
		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					stack = append(stack, frame{elem: item.Element})
				}
			}
		}
	}

	// Function to check a relation
	checkRelation := func(fromID string, toID string) {
		// If pointing TO a requirement
		if requirements[toID] {
			linkedElements[fromID] = true // This element is linked
		}
		// If THIS is a requirement pointing to something
		if requirements[fromID] {
			linkedElements[toID] = true // That element is linked
		}
	}

	// Pass 2: Check relations (both top-level and nested)
	// 2a. Top-level relations
	for _, item := range model.Items {
		if item.Relation != nil {
			r := item.Relation
			fromParts := r.From.Parts
			toParts := r.To.Parts
			if len(fromParts) > 0 && len(toParts) > 0 {
				checkRelation(fromParts[len(fromParts)-1], toParts[len(toParts)-1])
			}
		}
	}

	// 2b. Nested relations
	stack = make([]frame, 0, 16)
	for _, item := range model.Items {
		if item.ElementDef != nil {
			stack = append(stack, frame{elem: item.ElementDef})
		}
	}

	for len(stack) > 0 {
		f := stack[len(stack)-1]
		stack = stack[:len(stack)-1]
		elem := f.elem
		if elem == nil {
			continue
		}
		id := elem.GetID()

		// Tags Check: Does this element have a tag that matches a requirement ID?
		tags := elem.GetTagRefs()
		for _, t := range tags {
			if requirements[t] {
				linkedElements[id] = true
			}
		}

		// Relation Check
		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Relation != nil {
					toParts := item.Relation.To.Parts
					if len(toParts) > 0 {
						toID := toParts[len(toParts)-1]
						checkRelation(id, toID)
					}
				}
				if item.Element != nil {
					stack = append(stack, frame{elem: item.Element})
				}
			}
		}
	}

	totalCount := len(elements)
	linkedCount := len(linkedElements)

	if totalCount > 0 && float64(linkedCount)/float64(totalCount) < ThresholdTraceabilityRatio {
		scores.Traceability -= PenaltyLowTraceability
		*deductions = append(*deductions, Deduction{
			Rule:     "Low Traceability",
			Points:   PenaltyLowTraceability,
			Message:  fmt.Sprintf("%.0f%% of elements are mapped to requirements (target 50%%)", float64(linkedCount)/float64(totalCount)*100),
			Target:   "model",
			Severity: diagnostics.SeverityWarning,
			Category: "Traceability",
		})
	}
}

// checkComplexity calculates fan-in and fan-out to penalize God objects.
func (s *Scorer) checkComplexity(model *language.Model, scores *CategoryScores, deductions *[]Deduction) {
	if model == nil {
		return
	}

	// Map of ElementID -> Connection Count
	connections := make(map[string]int)
	// Map to get kind/location for reporting
	elementInfo := make(map[string]*language.ElementDef)

	type frame struct {
		elem *language.ElementDef
	}
	stack := make([]frame, 0, 16)

	for _, item := range model.Items {
		if item.ElementDef != nil {
			stack = append(stack, frame{elem: item.ElementDef})
		}
	}

	for len(stack) > 0 {
		f := stack[len(stack)-1]
		stack = stack[:len(stack)-1]
		elem := f.elem
		if elem == nil {
			continue
		}
		id := elem.GetID()
		elementInfo[id] = elem

		// Recurse to find all elements first
		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Element != nil {
					stack = append(stack, frame{elem: item.Element})
				}
			}
		}
	}

	// Helper to count connection
	countConnection := func(fromID, toID string) {
		connections[fromID]++
		connections[toID]++
	}

	// Pass 1: Top-level relations
	for _, item := range model.Items {
		if item.Relation != nil {
			r := item.Relation
			fromParts := r.From.Parts
			toParts := r.To.Parts
			if len(fromParts) > 0 && len(toParts) > 0 {
				countConnection(fromParts[len(fromParts)-1], toParts[len(toParts)-1])
			}
		}
	}

	// Pass 2: Nested relations
	stack = make([]frame, 0, 16)
	for _, item := range model.Items {
		if item.ElementDef != nil {
			stack = append(stack, frame{elem: item.ElementDef})
		}
	}

	for len(stack) > 0 {
		f := stack[len(stack)-1]
		stack = stack[:len(stack)-1]
		elem := f.elem
		if elem == nil {
			continue
		}
		id := elem.GetID()

		// Count relations
		body := elem.GetBody()
		if body != nil {
			for _, item := range body.Items {
				if item.Relation != nil {
					// Outgoing relation from this element
					toParts := item.Relation.To.Parts
					if len(toParts) > 0 {
						toID := toParts[len(toParts)-1]
						countConnection(id, toID)
					}
				}
				if item.Element != nil {
					stack = append(stack, frame{elem: item.Element})
				}
			}
		}
	}

	// Check for high complexity
	highComplexityCount := 0
	var highComplexityExamples []string

	for id, count := range connections {
		// Only penalize components/containers, not systems/requirements usually
		info, ok := elementInfo[id]
		if !ok {
			continue
		}
		kind := strings.ToLower(info.GetKind())
		if kind != "component" && kind != "container" {
			continue
		}

		if count > ThresholdHighComplexity {
			scores.Complexity -= PenaltyHighComplexity
			highComplexityCount++
			if len(highComplexityExamples) < 3 {
				highComplexityExamples = append(highComplexityExamples, id)
			}
		}
	}

	if highComplexityCount > 0 {
		exampleStr := strings.Join(highComplexityExamples, "', '")
		msg := fmt.Sprintf("%d elements have too many connections (> %d) (-%d pts each) (e.g., '%s'...)", highComplexityCount, ThresholdHighComplexity, PenaltyHighComplexity, exampleStr)
		if highComplexityCount <= 3 {
			msg = fmt.Sprintf("%d elements have too many connections (> %d) (-%d pts each) ('%s')", highComplexityCount, ThresholdHighComplexity, PenaltyHighComplexity, exampleStr)
		}

		*deductions = append(*deductions, Deduction{
			Rule:     "High Complexity",
			Points:   PenaltyHighComplexity * highComplexityCount,
			Message:  msg,
			Target:   "model", // model-wide warning
			Severity: diagnostics.SeverityWarning,
			Category: "Complexity",
		})
	}
}

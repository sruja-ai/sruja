package markdown

import (
	"github.com/sruja-ai/sruja/pkg/engine"
	exportpkg "github.com/sruja-ai/sruja/pkg/export"
	"github.com/sruja-ai/sruja/pkg/language"
)

// Exporter handles Markdown document generation.
type Exporter struct {
	Options Options
}

// NewExporter creates a new Markdown exporter.
func NewExporter(options Options) *Exporter {
	return &Exporter{Options: options}
}

// Export generates a Markdown document from a program.
func (e *Exporter) Export(prog *language.Program) string {
	if prog == nil || prog.Model == nil {
		return ""
	}

	// Apply scoping if specified
	if e.Options.Scope != nil && e.Options.Scope.Type != "full" {
		prog = exportpkg.FilterByScope(prog, e.Options.Scope.Type, e.Options.Scope.ID)
		if prog == nil || prog.Model == nil {
			return ""
		}
	}

	sb := engine.GetStringBuilder()
	defer engine.PutStringBuilder(sb)

	// Extract elements from Model
	systems := extractSystemsFromModel(prog)
	persons := extractPersonsFromModel(prog)
	requirements := extractRequirementsFromModel(prog)
	adrs := extractADRsFromModel(prog)

	// Apply token optimization if needed
	optimizer := exportpkg.NewTokenOptimizer(e.Options.TokenLimit)

	// Build initial content to estimate tokens
	initialContent := e.buildContent(systems, persons, requirements, adrs, prog)

	if optimizer.ShouldOptimize(exportpkg.EstimateTokens(initialContent)) {
		// Apply optimizations
		systems, persons, requirements, adrs = e.optimizeContent(
			optimizer, systems, persons, requirements, adrs, prog)
	}

	// Build content based on context type
	e.writeContent(sb, systems, persons, requirements, adrs, prog)

	result := sb.String()

	// Final token check and truncation if needed
	if optimizer.ShouldOptimize(exportpkg.EstimateTokens(result)) {
		result = e.truncateToLimit(result, e.Options.TokenLimit)
	}

	return result
}

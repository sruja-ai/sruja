// pkg/export/svg/render_ddd.go
// DDD domain model rendering for SVG export
package svg

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/export/d2"
	"github.com/sruja-ai/sruja/pkg/language"
)

// renderDomain renders a DDD domain as an independent D2 view
func (e *Exporter) renderDomain(arch *language.Architecture, domain *language.DomainBlock) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Create a temporary architecture with only this domain
	// Ensure it's properly initialized
	tempArch := &language.Architecture{
		Name:    arch.Name,
		Persons: arch.Persons,
		Systems: arch.Systems, // Keep systems for context
		Domains: []*language.DomainBlock{domain},
	}

	// Post-process to ensure all fields are populated
	tempArch.PostProcess()

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("D2 export failed for domain %s: %w", domain.ID, err)
	}

	// Check if D2 script is empty or invalid
	if len(d2Script) == 0 {
		// Return empty SVG instead of error for empty domains
		return `<svg xmlns="http://www.w3.org/2000/svg"><text x="10" y="20">No content to display</text></svg>`, nil
	}

	return e.compileAndRenderD2(d2Script)
}

// renderContext renders a DDD context as an independent D2 view
func (e *Exporter) renderContext(arch *language.Architecture, domain *language.DomainBlock, ctx *language.ContextBlock) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Create a temporary domain with only this context
	tempDomain := &language.DomainBlock{
		ID:          domain.ID,
		Label:       domain.Label,
		Description: domain.Description,
		Contexts:    []*language.ContextBlock{ctx},
	}

	// Create a temporary architecture with only this domain and context
	tempArch := &language.Architecture{
		Name:    arch.Name,
		Persons: arch.Persons,
		Systems: arch.Systems, // Keep systems for context
		Domains: []*language.DomainBlock{tempDomain},
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export context %s: %w", ctx.ID, err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderAggregate renders a DDD aggregate as an independent D2 view
func (e *Exporter) renderAggregate(arch *language.Architecture, domain *language.DomainBlock, ctx *language.ContextBlock, agg *language.Aggregate) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	// Create a temporary context with only this aggregate
	tempCtx := &language.ContextBlock{
		ID:          ctx.ID,
		Label:       ctx.Label,
		Description: ctx.Description,
		Aggregates:  []*language.Aggregate{agg},
	}

	tempDomain := &language.DomainBlock{
		ID:          domain.ID,
		Label:       domain.Label,
		Description: domain.Description,
		Contexts:    []*language.ContextBlock{tempCtx},
	}

	tempArch := &language.Architecture{
		Name:    arch.Name,
		Persons: arch.Persons,
		Systems: arch.Systems,
		Domains: []*language.DomainBlock{tempDomain},
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export aggregate %s: %w", agg.ID, err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderEntity renders a DDD entity as an independent D2 view
func (e *Exporter) renderEntity(arch *language.Architecture, domain *language.DomainBlock, ctx *language.ContextBlock, entity *language.Entity) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	tempCtx := &language.ContextBlock{
		ID:          ctx.ID,
		Label:       ctx.Label,
		Description: ctx.Description,
		Entities:    []*language.Entity{entity},
	}

	tempDomain := &language.DomainBlock{
		ID:          domain.ID,
		Label:       domain.Label,
		Description: domain.Description,
		Contexts:    []*language.ContextBlock{tempCtx},
	}

	tempArch := &language.Architecture{
		Name:    arch.Name,
		Persons: arch.Persons,
		Systems: arch.Systems,
		Domains: []*language.DomainBlock{tempDomain},
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export entity %s: %w", entity.ID, err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderValueObject renders a DDD value object as an independent D2 view
func (e *Exporter) renderValueObject(arch *language.Architecture, domain *language.DomainBlock, ctx *language.ContextBlock, vo *language.ValueObject) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	tempCtx := &language.ContextBlock{
		ID:           ctx.ID,
		Label:        ctx.Label,
		Description:  ctx.Description,
		ValueObjects: []*language.ValueObject{vo},
	}

	tempDomain := &language.DomainBlock{
		ID:          domain.ID,
		Label:       domain.Label,
		Description: domain.Description,
		Contexts:    []*language.ContextBlock{tempCtx},
	}

	tempArch := &language.Architecture{
		Name:    arch.Name,
		Persons: arch.Persons,
		Systems: arch.Systems,
		Domains: []*language.DomainBlock{tempDomain},
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export value object %s: %w", vo.ID, err)
	}

	return e.compileAndRenderD2(d2Script)
}

// renderDomainEvent renders a DDD domain event as an independent D2 view
func (e *Exporter) renderDomainEvent(arch *language.Architecture, domain *language.DomainBlock, ctx *language.ContextBlock, event *language.DomainEvent) (string, error) {
	d2Exporter := d2.NewExporter()
	d2Exporter.InjectIDTooltips = true
	d2Exporter.ScopeLevel = 0

	tempCtx := &language.ContextBlock{
		ID:          ctx.ID,
		Label:       ctx.Label,
		Description: ctx.Description,
		Events:      []*language.DomainEvent{event},
	}

	tempDomain := &language.DomainBlock{
		ID:          domain.ID,
		Label:       domain.Label,
		Description: domain.Description,
		Contexts:    []*language.ContextBlock{tempCtx},
	}

	tempArch := &language.Architecture{
		Name:    arch.Name,
		Persons: arch.Persons,
		Systems: arch.Systems,
		Domains: []*language.DomainBlock{tempDomain},
	}

	d2Script, err := d2Exporter.Export(tempArch)
	if err != nil {
		return "", fmt.Errorf("failed to export domain event %s: %w", event.ID, err)
	}

	return e.compileAndRenderD2(d2Script)
}

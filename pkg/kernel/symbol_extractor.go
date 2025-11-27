// pkg/kernel/symbol_extractor.go
// Package kernel provides symbol extraction from AST for symbol table population.
package kernel

import (
	"github.com/sruja-ai/sruja/pkg/language"
	"github.com/sruja-ai/sruja/pkg/model"
)

// extractSymbolsFromProgram extracts all symbols from a parsed program and adds them to the symbol table.
func (k *Kernel) extractSymbolsFromProgram(cellID CellID, program *language.Program) {
	if program == nil || program.Architecture == nil {
		return
	}

	arch := program.Architecture

	// Extract systems
	for _, sys := range arch.Systems {
		if sys != nil {
			loc := convertSourceLocation(cellID, sys.Location())
			k.symbols.AddSymbol(sys.ID, SymbolKindSystem, sys.Label, loc)
		}
	}

	// Extract containers from systems
	for _, sys := range arch.Systems {
		if sys != nil {
			for _, container := range sys.Containers {
				if container != nil {
					loc := convertSourceLocation(cellID, container.Location())
					k.symbols.AddSymbol(container.ID, SymbolKindContainer, container.Label, loc)
				}
			}
		}
	}

	// Extract components from containers
	for _, sys := range arch.Systems {
		if sys != nil {
			for _, container := range sys.Containers {
				if container != nil {
					for _, component := range container.Components {
						if component != nil {
							loc := convertSourceLocation(cellID, component.Location())
							k.symbols.AddSymbol(component.ID, SymbolKindComponent, component.Label, loc)
						}
					}
				}
			}
		}
	}

	// Extract entities (from architecture level)
	for _, entity := range arch.Entities {
		if entity != nil {
			loc := convertSourceLocation(cellID, entity.Location())
			k.symbols.AddSymbol(entity.Name, SymbolKindEntity, entity.Name, loc)
		}
	}

	// Extract entities and events from domains (via ArchitectureItem)
	for _, item := range arch.Items {
		if item.Domain != nil {
			domain := item.Domain
			// Extract entities from domain
			if domain.EntitiesBlock != nil {
				for _, entity := range domain.EntitiesBlock.Entities {
					if entity != nil {
						loc := convertSourceLocation(cellID, entity.Location())
						k.symbols.AddSymbol(entity.Name, SymbolKindEntity, entity.Name, loc)
					}
				}
			}
			// Extract events from domain
			if domain.EventsBlock != nil {
				for _, event := range domain.EventsBlock.Events {
					if event != nil {
						loc := convertSourceLocation(cellID, event.Location())
						k.symbols.AddSymbol(event.Name, SymbolKindEvent, event.Name, loc)
					}
				}
			}
		}
	}

	// Extract events (from architecture level)
	for _, event := range arch.Events {
		if event != nil {
			loc := convertSourceLocation(cellID, event.Location())
			k.symbols.AddSymbol(event.Name, SymbolKindEvent, event.Name, loc)
		}
	}

	// Extract contracts
	for _, contract := range arch.Contracts {
		if contract != nil {
			loc := convertSourceLocation(cellID, contract.Location())
			k.symbols.AddSymbol(contract.ID, SymbolKindContract, contract.ID, loc)
		}
	}

	// Extract requirements
	for _, req := range arch.Requirements {
		if req != nil {
			loc := convertSourceLocation(cellID, req.Location())
			k.symbols.AddSymbol(req.ID, SymbolKindRequirement, req.ID, loc)
		}
	}

	// Extract ADRs
	for _, adr := range arch.ADRs {
		if adr != nil {
			loc := convertSourceLocation(cellID, adr.Location())
			title := adr.ID
			if adr.Title != "" {
				title = adr.Title
			}
			k.symbols.AddSymbol(adr.ID, SymbolKindADR, title, loc)
		}
	}

	// Extract relations (for reference tracking)
	for _, rel := range arch.Relations {
		if rel != nil {
			loc := convertSourceLocation(cellID, rel.Location())
			// Add reference from source to target
			k.symbols.AddReference(rel.From, SymbolReference{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			})
			k.symbols.AddReference(rel.To, SymbolReference{
				File:   loc.File,
				Line:   loc.Line,
				Column: loc.Column,
			})
		}
	}

	// Extract persons
	for _, person := range arch.Persons {
		if person != nil {
			loc := convertSourceLocation(cellID, person.Location())
			k.symbols.AddSymbol(person.ID, SymbolKindSystem, person.Label, loc) // Using System kind for now
		}
	}
}

// convertSourceLocation converts language.SourceLocation to model.Location.
func convertSourceLocation(cellID CellID, loc language.SourceLocation) model.Location {
	return model.Location{
		File:   string(cellID),
		Line:   loc.Line,
		Column: loc.Column,
	}
}

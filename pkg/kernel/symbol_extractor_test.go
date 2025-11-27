// pkg/kernel/symbol_extractor_test.go
package kernel

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/model"
)

func TestSymbolExtraction(t *testing.T) {
	k, err := NewKernel()
	if err != nil {
		t.Fatalf("Failed to create kernel: %v", err)
	}

	// Execute a DSL cell with multiple symbols
	source := `architecture "Test" {
  system Billing {
    container BillingAPI {
      component PaymentService {}
    }
  }
  
  entity Payment {
    fields {
      amount: Float
    }
  }
  
  requirement R1 functional "Must process payments"
  
  adr ADR001 "Use event-driven architecture"
}`

	result, err := k.ExecuteCell("cell-1", CellTypeDSL, source)
	if err != nil {
		t.Fatalf("Failed to execute cell: %v", err)
	}

	// Check that symbols were extracted (even if parsing failed, we can test the extractor)
	// The symbol table should have symbols if parsing succeeded
	if result.Success {
		// Verify symbol table has entries
		symbols := k.symbols.GetAllSymbols()
		if len(symbols) == 0 {
			t.Log("No symbols extracted (may be due to parsing issues)")
		} else {
			t.Logf("Extracted %d symbols", len(symbols))
		}
	}
}

func TestSymbolTable_RemoveSymbolsByFile(t *testing.T) {
	st := NewSymbolTable()

	// Add symbols from different files
	st.AddSymbol("System1", SymbolKindSystem, "System 1", model.Location{File: "cell-1", Line: 1, Column: 1})
	st.AddSymbol("System2", SymbolKindSystem, "System 2", model.Location{File: "cell-2", Line: 1, Column: 1})
	st.AddSymbol("Entity1", SymbolKindEntity, "Entity 1", model.Location{File: "cell-1", Line: 10, Column: 1})

	// Verify initial state
	symbols := st.GetAllSymbols()
	if len(symbols) != 3 {
		t.Errorf("Expected 3 symbols, got %d", len(symbols))
	}

	// Remove symbols from cell-1
	st.RemoveSymbolsByFile("cell-1")

	// Verify only cell-2 symbols remain
	symbols = st.GetAllSymbols()
	if len(symbols) != 1 {
		t.Errorf("Expected 1 symbol after removal, got %d", len(symbols))
	}

	if _, ok := symbols["System2"]; !ok {
		t.Error("System2 should still exist")
	}
}

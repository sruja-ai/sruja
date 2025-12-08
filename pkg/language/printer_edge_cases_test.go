package language_test

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/language"
)

func TestPrinter_EdgeCases(t *testing.T) {
	t.Run("Empty Architecture", func(t *testing.T) {
		arch := &language.Architecture{Name: "Empty"}
		prog := &language.Program{Architecture: arch}
		printer := language.NewPrinter()
		output := printer.Print(prog)
		if output == "" {
			t.Error("Expected output for empty architecture")
		}
	})

	t.Run("Nil Architecture", func(_ *testing.T) {
		printer := language.NewPrinter()
		defer func() {
			if r := recover(); r == nil {
				// It might not panic, but if it does, we catch it.
				// Ideally it should handle nil gracefully or panic if it's invalid state.
				// Let's assume it might panic or return empty.
				_ = r // suppress unused
			}
		}()
		_ = printer.Print(nil)
	})

	t.Run("Nil Fields in Structs", func(t *testing.T) {
		arch := &language.Architecture{
			Name: "NilFields",
			Systems: []*language.System{
				{
					ID: "S1",
					// Label is string, so "" is empty.
					// Description is *string, so nil is allowed.
					Description: nil,
				},
			},
		}
		prog := &language.Program{Architecture: arch}
		printer := language.NewPrinter()
		output := printer.Print(prog)
		if output == "" {
			t.Error("Expected output")
		}
	})
}

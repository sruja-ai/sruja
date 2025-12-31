package engine

import (
	"testing"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

func TestGetStringBoolMap(t *testing.T) {
	m := GetStringBoolMap()
	if m == nil {
		t.Fatal("GetStringBoolMap returned nil")
	}

	// Test that map is usable
	(*m)["test"] = true
	if !(*m)["test"] {
		t.Error("Expected map to contain 'test' = true")
	}

	// Return to pool
	PutStringBoolMap(m)

	// Get another and ensure it's cleared
	m2 := GetStringBoolMap()
	if len(*m2) != 0 {
		t.Error("Pooled map was not cleared")
	}
	PutStringBoolMap(m2)
}

func TestGetStringSliceMap(t *testing.T) {
	m := GetStringSliceMap()
	if m == nil {
		t.Fatal("GetStringSliceMap returned nil")
	}

	// Test that map is usable
	(*m)["key"] = []string{"a", "b", "c"}
	if len((*m)["key"]) != 3 {
		t.Error("Expected map to contain slice with 3 elements")
	}

	// Return to pool
	PutStringSliceMap(m)

	// Get another and ensure it's cleared
	m2 := GetStringSliceMap()
	if len(*m2) != 0 {
		t.Error("Pooled map was not cleared")
	}
	PutStringSliceMap(m2)
}

func TestGetSourceLocationMap(t *testing.T) {
	m := GetSourceLocationMap()
	if m == nil {
		t.Fatal("GetSourceLocationMap returned nil")
	}

	// Test that map is usable
	(*m)["element"] = diagnostics.SourceLocation{File: "test.sruja", Line: 1}
	if (*m)["element"].File != "test.sruja" {
		t.Error("Expected map to contain SourceLocation")
	}

	// Return to pool
	PutSourceLocationMap(m)

	// Get another and ensure it's cleared
	m2 := GetSourceLocationMap()
	if len(*m2) != 0 {
		t.Error("Pooled map was not cleared")
	}
	PutSourceLocationMap(m2)
}

func TestGetStringSlice(t *testing.T) {
	s := GetStringSlice()
	if s == nil {
		t.Fatal("GetStringSlice returned nil")
	}

	// Test that slice is usable
	*s = append(*s, "a", "b", "c")
	if len(*s) != 3 {
		t.Error("Expected slice with 3 elements")
	}

	// Return to pool
	PutStringSlice(s)

	// Get another and ensure it's reset
	s2 := GetStringSlice()
	if len(*s2) != 0 {
		t.Error("Pooled slice was not reset")
	}
	PutStringSlice(s2)
}

func TestGetSuggestionsSlice(t *testing.T) {
	s := GetSuggestionsSlice()
	if s == nil {
		t.Fatal("GetSuggestionsSlice returned nil")
	}

	// Test that slice is usable
	*s = append(*s, "suggestion1", "suggestion2")
	if len(*s) != 2 {
		t.Error("Expected slice with 2 elements")
	}

	// Return to pool
	PutSuggestionsSlice(s)

	// Get another and ensure it's reset
	s2 := GetSuggestionsSlice()
	if len(*s2) != 0 {
		t.Error("Pooled slice was not reset")
	}
	PutSuggestionsSlice(s2)
}

func TestPoolsNilSafety(t *testing.T) {
	// These should not panic
	PutStringBoolMap(nil)
	PutStringSliceMap(nil)
	PutSourceLocationMap(nil)
	PutStringSlice(nil)
	PutSuggestionsSlice(nil)
}

// BenchmarkStringBoolMapPool benchmarks pooled vs non-pooled map creation.
func BenchmarkStringBoolMapPool(b *testing.B) {
	b.Run("Pooled", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			m := GetStringBoolMap()
			(*m)["key1"] = true
			(*m)["key2"] = true
			(*m)["key3"] = true
			PutStringBoolMap(m)
		}
	})

	b.Run("NonPooled", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			m := make(map[string]bool, 32)
			m["key1"] = true
			m["key2"] = true
			m["key3"] = true
			_ = m
		}
	})
}

// BenchmarkStringSliceMapPool benchmarks pooled vs non-pooled adjacency list creation.
func BenchmarkStringSliceMapPool(b *testing.B) {
	b.Run("Pooled", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			m := GetStringSliceMap()
			(*m)["a"] = append((*m)["a"], "b")
			(*m)["b"] = append((*m)["b"], "c")
			PutStringSliceMap(m)
		}
	})

	b.Run("NonPooled", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			m := make(map[string][]string, 32)
			m["a"] = append(m["a"], "b")
			m["b"] = append(m["b"], "c")
			_ = m
		}
	})
}

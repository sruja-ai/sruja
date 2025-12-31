// Package engine provides sync.Pool instances for map types.
// These pools reduce GC pressure in hot paths like validation and scoring
// where maps are frequently created and discarded.
package engine

import (
	"sync"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

// Map pool capacity constants based on profiling data.
// These represent typical sizes to avoid reallocations.
const (
	// DefaultMapCapacity is the default initial capacity for pooled maps.
	DefaultMapCapacity = 32

	// SmallMapCapacity is for maps that typically hold fewer items.
	SmallMapCapacity = 16

	// LargeMapCapacity is for maps in large architectures.
	LargeMapCapacity = 64
)

// stringBoolMapPool provides reusable map[string]bool instances.
// Used for visited/defined sets in graph algorithms.
var stringBoolMapPool = sync.Pool{
	New: func() interface{} {
		m := make(map[string]bool, DefaultMapCapacity)
		return &m
	},
}

// stringSliceMapPool provides reusable map[string][]string instances.
// Used for adjacency lists and suffix maps.
var stringSliceMapPool = sync.Pool{
	New: func() interface{} {
		m := make(map[string][]string, DefaultMapCapacity)
		return &m
	},
}

// sourceLocationMapPool provides reusable map[string]SourceLocation instances.
// Used for tracking element locations during validation.
var sourceLocationMapPool = sync.Pool{
	New: func() interface{} {
		m := make(map[string]diagnostics.SourceLocation, DefaultMapCapacity)
		return &m
	},
}

// stringSlicePool provides reusable []string instances.
// Used for path tracking in DFS and other algorithms.
var stringSlicePool = sync.Pool{
	New: func() interface{} {
		s := make([]string, 0, SmallMapCapacity)
		return &s
	},
}

// GetStringBoolMap retrieves a map[string]bool from the pool.
// The map is cleared before returning to ensure clean state.
// Always pair with PutStringBoolMap when done.
//
// Example:
//
//	m := GetStringBoolMap()
//	defer PutStringBoolMap(m)
//	(*m)["key"] = true
func GetStringBoolMap() *map[string]bool {
	m := stringBoolMapPool.Get().(*map[string]bool)
	// Clear the map efficiently
	for k := range *m {
		delete(*m, k)
	}
	return m
}

// PutStringBoolMap returns a map[string]bool to the pool.
// Call this when done with a map obtained from GetStringBoolMap.
func PutStringBoolMap(m *map[string]bool) {
	if m == nil {
		return
	}
	// Only pool maps with reasonable capacity to avoid memory bloat
	if len(*m) <= LargeMapCapacity*2 {
		stringBoolMapPool.Put(m)
	}
}

// GetStringSliceMap retrieves a map[string][]string from the pool.
// The map is cleared before returning to ensure clean state.
// Always pair with PutStringSliceMap when done.
func GetStringSliceMap() *map[string][]string {
	m := stringSliceMapPool.Get().(*map[string][]string)
	// Clear the map - note: this doesn't free the slice memory
	for k := range *m {
		delete(*m, k)
	}
	return m
}

// PutStringSliceMap returns a map[string][]string to the pool.
// Call this when done with a map obtained from GetStringSliceMap.
func PutStringSliceMap(m *map[string][]string) {
	if m == nil {
		return
	}
	// Only pool maps with reasonable capacity
	if len(*m) <= LargeMapCapacity*2 {
		stringSliceMapPool.Put(m)
	}
}

// GetSourceLocationMap retrieves a map[string]SourceLocation from the pool.
// The map is cleared before returning to ensure clean state.
// Always pair with PutSourceLocationMap when done.
func GetSourceLocationMap() *map[string]diagnostics.SourceLocation {
	m := sourceLocationMapPool.Get().(*map[string]diagnostics.SourceLocation)
	// Clear the map efficiently
	for k := range *m {
		delete(*m, k)
	}
	return m
}

// PutSourceLocationMap returns a map[string]SourceLocation to the pool.
// Call this when done with a map obtained from GetSourceLocationMap.
func PutSourceLocationMap(m *map[string]diagnostics.SourceLocation) {
	if m == nil {
		return
	}
	// Only pool maps with reasonable capacity
	if len(*m) <= LargeMapCapacity*2 {
		sourceLocationMapPool.Put(m)
	}
}

// GetStringSlice retrieves a []string from the pool.
// The slice is reset to zero length before returning.
// Always pair with PutStringSlice when done.
func GetStringSlice() *[]string {
	s := stringSlicePool.Get().(*[]string)
	*s = (*s)[:0]
	return s
}

// PutStringSlice returns a []string to the pool.
// Call this when done with a slice obtained from GetStringSlice.
func PutStringSlice(s *[]string) {
	if s == nil {
		return
	}
	// Only pool slices with reasonable capacity
	if cap(*s) <= LargeMapCapacity*2 {
		stringSlicePool.Put(s)
	}
}

// suggestionsPool provides pre-allocated suggestion slices.
// Commonly used in diagnostic creation with 3-5 suggestions.
var suggestionsPool = sync.Pool{
	New: func() interface{} {
		s := make([]string, 0, 5)
		return &s
	},
}

// GetSuggestionsSlice retrieves a []string for suggestions from the pool.
func GetSuggestionsSlice() *[]string {
	s := suggestionsPool.Get().(*[]string)
	*s = (*s)[:0]
	return s
}

// PutSuggestionsSlice returns a suggestions slice to the pool.
func PutSuggestionsSlice(s *[]string) {
	if s == nil {
		return
	}
	if cap(*s) <= 16 {
		suggestionsPool.Put(s)
	}
}

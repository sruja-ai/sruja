// Package engine provides sync.Pool instances for frequently allocated objects.
// Using pools reduces GC pressure in hot paths like validation and scoring.
package engine

import (
	"strings"
	"sync"

	"github.com/sruja-ai/sruja/pkg/diagnostics"
)

// stringBuilderPool provides reusable strings.Builder instances.
// String builders are frequently used in message construction during validation.
var stringBuilderPool = sync.Pool{
	New: func() interface{} {
		return new(strings.Builder)
	},
}

// diagnosticsSlicePool provides reusable diagnostic slices.
// Pre-allocates with capacity 16 to handle typical validation output.
var diagnosticsSlicePool = sync.Pool{
	New: func() interface{} {
		s := make([]diagnostics.Diagnostic, 0, 16)
		return &s
	},
}

// byteSlicePool provides reusable byte slices for string building.
// Pre-allocates with capacity 256 for typical qualified ID construction.
var byteSlicePool = sync.Pool{
	New: func() interface{} {
		b := make([]byte, 0, 256)
		return &b
	},
}

// GetStringBuilder retrieves a strings.Builder from the pool.
// The builder is reset before returning to ensure clean state.
// Always pair with PutStringBuilder when done.
//
// Example:
//
//	sb := GetStringBuilder()
//	defer PutStringBuilder(sb)
//	sb.WriteString("hello")
//	result := sb.String()
func GetStringBuilder() *strings.Builder {
	sb := stringBuilderPool.Get().(*strings.Builder)
	sb.Reset()
	return sb
}

// PutStringBuilder returns a strings.Builder to the pool.
// Call this when done with a builder obtained from GetStringBuilder.
func PutStringBuilder(sb *strings.Builder) {
	// Only pool builders with reasonable capacity to avoid memory bloat
	if sb.Cap() <= 4096 {
		stringBuilderPool.Put(sb)
	}
}

// GetDiagnosticsSlice retrieves a diagnostics slice from the pool.
// The slice is reset to zero length before returning.
// Always pair with PutDiagnosticsSlice when done.
func GetDiagnosticsSlice() *[]diagnostics.Diagnostic {
	s := diagnosticsSlicePool.Get().(*[]diagnostics.Diagnostic)
	*s = (*s)[:0]
	return s
}

// PutDiagnosticsSlice returns a diagnostics slice to the pool.
// Call this when done with a slice obtained from GetDiagnosticsSlice.
func PutDiagnosticsSlice(s *[]diagnostics.Diagnostic) {
	// Only pool slices with reasonable capacity to avoid memory bloat
	if cap(*s) <= 256 {
		diagnosticsSlicePool.Put(s)
	}
}

// GetByteSlice retrieves a byte slice from the pool.
// The slice is reset to zero length before returning.
// Always pair with PutByteSlice when done.
func GetByteSlice() *[]byte {
	b := byteSlicePool.Get().(*[]byte)
	*b = (*b)[:0]
	return b
}

// PutByteSlice returns a byte slice to the pool.
// Call this when done with a slice obtained from GetByteSlice.
func PutByteSlice(b *[]byte) {
	// Only pool slices with reasonable capacity to avoid memory bloat
	if cap(*b) <= 1024 {
		byteSlicePool.Put(b)
	}
}

// BuildQualifiedID efficiently builds a qualified ID string from parts.
// Uses pooled byte slice to minimize allocations.
// Example: BuildQualifiedID("system", "container", "component") returns "system.container.component"
func BuildQualifiedID(parts ...string) string {
	if len(parts) == 0 {
		return ""
	}
	if len(parts) == 1 {
		return parts[0]
	}

	// Calculate total length needed
	totalLen := len(parts) - 1 // for dots
	for _, p := range parts {
		totalLen += len(p)
	}

	// Use pooled buffer
	bufPtr := GetByteSlice()
	buf := *bufPtr

	// Ensure capacity
	if cap(buf) < totalLen {
		buf = make([]byte, 0, totalLen)
	}

	buf = append(buf, parts[0]...)
	for i := 1; i < len(parts); i++ {
		buf = append(buf, '.')
		buf = append(buf, parts[i]...)
	}

	result := string(buf)

	// Return buffer to pool
	*bufPtr = buf
	PutByteSlice(bufPtr)

	return result
}

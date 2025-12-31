//go:build js && wasm

// Package main provides monitoring, logging, and resource management for WASM exports.
package main

import (
	"fmt"
	"runtime"
	"time"
)

// ExportMetrics tracks metrics for export operations.
type ExportMetrics struct {
	Operation     string
	StartTime     time.Time
	Duration      time.Duration
	ElementCount  int
	RelationCount int
	ViewLevel     int
	InputSize     int
	OutputSize    int
	Success       bool
	ErrorCode     ErrorCode
	MemoryBefore  uint64
	MemoryAfter   uint64
	CacheHit      bool
}

// ResourceLimits defines resource limits for export operations.
type ResourceLimits struct {
	MaxDuration   time.Duration
	MaxMemoryMB   uint64
	MaxInputSize  int
	MaxOutputSize int
}

// DefaultResourceLimits returns default resource limits.
func DefaultResourceLimits() ResourceLimits {
	return ResourceLimits{
		MaxDuration:   30 * time.Second,
		MaxMemoryMB:   100, // 100MB
		MaxInputSize:  MaxInputSize,
		MaxOutputSize: 50 * 1024 * 1024, // 50MB
	}
}

// CheckMemoryUsage checks current memory usage and returns bytes used.
func CheckMemoryUsage() uint64 {
	var m runtime.MemStats
	runtime.ReadMemStats(&m)
	return m.Alloc
}

// CheckMemoryLimit checks if memory usage exceeds the limit.
func CheckMemoryLimit(limitMB uint64) *ExportError {
	memUsed := CheckMemoryUsage()
	memUsedMB := memUsed / (1024 * 1024)

	if memUsedMB > limitMB {
		return NewExportError(ErrCodeExportFailed,
			fmt.Sprintf("memory limit exceeded: %dMB used, %dMB limit", memUsedMB, limitMB)).
			WithContext("memoryUsedMB", memUsedMB).
			WithContext("memoryLimitMB", limitMB)
	}

	return nil
}

// StartMetrics starts collecting metrics for an operation.
func StartMetrics(operation string, inputSize int) *ExportMetrics {
	metrics := &ExportMetrics{
		Operation:    operation,
		StartTime:    time.Now(),
		InputSize:    inputSize,
		MemoryBefore: CheckMemoryUsage(),
	}
	return metrics
}

// FinishMetrics completes metrics collection.
func (m *ExportMetrics) FinishMetrics(success bool, err *ExportError, elementCount, relationCount, outputSize int) {
	m.Duration = time.Since(m.StartTime)
	m.Success = success
	m.ElementCount = elementCount
	m.RelationCount = relationCount
	m.OutputSize = outputSize
	m.MemoryAfter = CheckMemoryUsage()

	if err != nil {
		m.ErrorCode = err.Code
	}

	// Log metrics (in WASM, we can't use standard logging, so we'll collect for JS)
	m.Log()
}

// Log outputs metrics. In WASM context, this could send to JS console or collect for reporting.
func (m *ExportMetrics) Log() {
	// In WASM, we can't use standard Go logging
	// Instead, we format metrics as a string that could be sent to JS
	// For now, we'll create a structured format that can be logged via JS

	// Format: operation|duration|success|errorCode|elements|relations|memoryDelta
	// This can be captured by JS and sent to logging service
	_ = fmt.Sprintf("METRICS: %s|%v|%v|%s|%d|%d|%dMB",
		m.Operation,
		m.Duration,
		m.Success,
		m.ErrorCode,
		m.ElementCount,
		m.RelationCount,
		(m.MemoryAfter-m.MemoryBefore)/(1024*1024),
	)

	// In a real implementation, this would send to JS via syscall/js
	// For now, metrics are collected but not actively logged
	// The caller can access metrics and log them if needed
}

// String returns a formatted string representation of metrics.
func (m *ExportMetrics) String() string {
	memDelta := int64(m.MemoryAfter) - int64(m.MemoryBefore)
	memDeltaMB := float64(memDelta) / (1024 * 1024)

	return fmt.Sprintf(
		"ExportMetrics{Operation: %s, Duration: %v, Success: %v, ErrorCode: %s, Elements: %d, Relations: %d, MemoryDelta: %.2fMB}",
		m.Operation,
		m.Duration,
		m.Success,
		m.ErrorCode,
		m.ElementCount,
		m.RelationCount,
		memDeltaMB,
	)
}

// ValidateResourceLimits validates that operation stays within resource limits.
func ValidateResourceLimits(limits ResourceLimits, inputSize int, startTime time.Time) *ExportError {
	// Check duration
	elapsed := time.Since(startTime)
	if elapsed > limits.MaxDuration {
		return NewExportError(ErrCodeExportTimeout,
			fmt.Sprintf("operation exceeded maximum duration of %v", limits.MaxDuration)).
			WithContext("duration", elapsed).
			WithContext("maxDuration", limits.MaxDuration)
	}

	// Check input size
	if inputSize > limits.MaxInputSize {
		return NewExportError(ErrCodeInputTooLarge,
			fmt.Sprintf("input exceeds maximum size of %d bytes", limits.MaxInputSize)).
			WithContext("size", inputSize).
			WithContext("maxSize", limits.MaxInputSize)
	}

	// Check memory
	if err := CheckMemoryLimit(limits.MaxMemoryMB); err != nil {
		return err
	}

	return nil
}

// WithTimeout executes a function with a timeout.
// Note: In WASM, we can't use context.Context easily, so this is a simplified version.
func WithTimeout(limits ResourceLimits, fn func() (*ExportMetrics, *ExportError)) (*ExportMetrics, *ExportError) {
	startTime := time.Now()

	// Start metrics
	metrics := StartMetrics("export", 0)

	// Execute function
	resultMetrics, err := fn()

	// Update metrics
	if resultMetrics != nil {
		metrics = resultMetrics
	}

	// Check timeout
	if err := ValidateResourceLimits(limits, metrics.InputSize, startTime); err != nil {
		metrics.FinishMetrics(false, err, 0, 0, 0)
		return metrics, err
	}

	return metrics, err
}

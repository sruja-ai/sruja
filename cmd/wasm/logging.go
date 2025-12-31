//go:build js && wasm

// Package main provides structured logging for WASM exports.
// In WASM context, logging is sent to JavaScript console or collected for reporting.
package main

import (
	"fmt"
	"syscall/js"
	"time"
)

// LogLevel represents the severity of a log message.
type LogLevel string

const (
	LogLevelDebug LogLevel = "DEBUG"
	LogLevelInfo  LogLevel = "INFO"
	LogLevelWarn  LogLevel = "WARN"
	LogLevelError LogLevel = "ERROR"
)

// LogEntry represents a structured log entry.
type LogEntry struct {
	Level     LogLevel
	Message   string
	Timestamp time.Time
	Component string
	Action    string
	ErrorCode ErrorCode
	Context   map[string]interface{}
}

// Logger provides structured logging for WASM operations.
type Logger struct {
	component string
	enabled   bool
}

// NewLogger creates a new logger for a component.
func NewLogger(component string) *Logger {
	return &Logger{
		component: component,
		enabled:   true, // Can be controlled via JS
	}
}

// log sends a log entry to JavaScript console.
func (l *Logger) log(level LogLevel, action, message string, errCode ErrorCode, context map[string]interface{}) {
	if !l.enabled {
		return
	}

	entry := LogEntry{
		Level:     level,
		Message:   message,
		Timestamp: time.Now(),
		Component: l.component,
		Action:    action,
		ErrorCode: errCode,
		Context:   context,
	}

	// Format log message
	logMsg := l.formatLog(entry)

	// Send to JavaScript console
	// In WASM, we use js.Global().Get("console")
	console := js.Global().Get("console")
	if !console.Truthy() {
		return // Console not available
	}

	// Map log levels to console methods
	var consoleMethod js.Value
	switch level {
	case LogLevelDebug:
		consoleMethod = console.Get("debug")
	case LogLevelInfo:
		consoleMethod = console.Get("info")
	case LogLevelWarn:
		consoleMethod = console.Get("warn")
	case LogLevelError:
		consoleMethod = console.Get("error")
	default:
		consoleMethod = console.Get("log")
	}

	if consoleMethod.Truthy() {
		// Create structured log object for JS
		logObj := js.Global().Get("Object").New()
		logObj.Set("level", string(level))
		logObj.Set("component", l.component)
		logObj.Set("action", action)
		logObj.Set("message", message)
		logObj.Set("timestamp", entry.Timestamp.Format(time.RFC3339))

		if errCode != "" {
			logObj.Set("errorCode", string(errCode))
		}

		if len(context) > 0 {
			contextObj := js.Global().Get("Object").New()
			for k, v := range context {
				contextObj.Set(k, fmt.Sprintf("%v", v))
			}
			logObj.Set("context", contextObj)
		}

		// Also include formatted string for simple console output
		consoleMethod.Invoke(logMsg, logObj)
	}
}

// formatLog formats a log entry as a string.
func (l *Logger) formatLog(entry LogEntry) string {
	msg := fmt.Sprintf("[%s] %s: %s", entry.Level, l.component, entry.Message)

	if entry.Action != "" {
		msg += fmt.Sprintf(" (action: %s)", entry.Action)
	}

	if entry.ErrorCode != "" {
		msg += fmt.Sprintf(" [%s]", entry.ErrorCode)
	}

	if len(entry.Context) > 0 {
		msg += fmt.Sprintf(" %v", entry.Context)
	}

	return msg
}

// Debug logs a debug message.
func (l *Logger) Debug(action, message string, context map[string]interface{}) {
	l.log(LogLevelDebug, action, message, "", context)
}

// Info logs an info message.
func (l *Logger) Info(action, message string, context map[string]interface{}) {
	l.log(LogLevelInfo, action, message, "", context)
}

// Warn logs a warning message.
func (l *Logger) Warn(action, message string, context map[string]interface{}) {
	l.log(LogLevelWarn, action, message, "", context)
}

// Error logs an error message.
func (l *Logger) Error(action, message string, errCode ErrorCode, context map[string]interface{}) {
	l.log(LogLevelError, action, message, errCode, context)
}

// LogExport logs export operation start/end.
func (l *Logger) LogExport(action string, metrics *ExportMetrics, err *ExportError) {
	context := map[string]interface{}{
		"duration":      metrics.Duration.String(),
		"elementCount":  metrics.ElementCount,
		"relationCount": metrics.RelationCount,
		"inputSize":     metrics.InputSize,
		"outputSize":    metrics.OutputSize,
		"viewLevel":     metrics.ViewLevel,
	}

	if metrics.MemoryAfter > metrics.MemoryBefore {
		memDelta := metrics.MemoryAfter - metrics.MemoryBefore
		context["memoryDeltaMB"] = float64(memDelta) / (1024 * 1024)
	}

	if err != nil {
		l.Error(action, fmt.Sprintf("Export failed: %s", err.Message), err.Code, context)
	} else {
		l.Info(action, "Export completed successfully", context)
	}
}

// Global logger instance
var defaultLogger = NewLogger("wasm")

// LogDebug is a convenience function for debug logging.
func LogDebug(action, message string, context map[string]interface{}) {
	defaultLogger.Debug(action, message, context)
}

// LogInfo is a convenience function for info logging.
func LogInfo(action, message string, context map[string]interface{}) {
	defaultLogger.Info(action, message, context)
}

// LogWarn is a convenience function for warning logging.
func LogWarn(action, message string, context map[string]interface{}) {
	defaultLogger.Warn(action, message, context)
}

// LogError is a convenience function for error logging.
func LogError(action, message string, errCode ErrorCode, context map[string]interface{}) {
	defaultLogger.Error(action, message, errCode, context)
}

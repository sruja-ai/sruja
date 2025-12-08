package diagnostics

// Standard Error Codes
const (
	// Syntax Errors (E1xx)
	CodeSyntaxError     = "E101" // Generic syntax error
	CodeUnexpectedToken = "E102" // Unexpected token
	CodeMissingBrace    = "E103" // Missing closing brace
	CodeInvalidString   = "E104" // Invalid string literal

	// Semantic Errors (E2xx)
	CodeDuplicateID     = "E201" // Duplicate identifier
	CodeUndefinedRef    = "E202" // Undefined reference
	CodeInvalidRelation = "E203" // Invalid relation
	CodeCycleDetected   = "E204" // Cycle detected
	CodeOrphanElement   = "E205" // Orphan element (warning)
	CodeLayerViolation  = "E206" // Layer violation

	// Validation Errors (E3xx)
	CodeInvalidProperty     = "E301" // Invalid property key/value
	CodeMissingField        = "E302" // Missing required field
	CodeValidationRuleError = "E303" // Generic validation rule error
	CodeValidationTimeout   = "E304" // Validation rule timed out
	CodeValidationPanic     = "E305" // Validation rule panicked
	CodeDuplicateIdentifier = "E201" // Alias for CodeDuplicateID
	CodeReferenceNotFound   = "E202" // Alias for CodeUndefinedRef
	CodeBestPractice        = "W001" // Best practice warning
)

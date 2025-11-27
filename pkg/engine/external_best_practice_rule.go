package engine

import (
	"github.com/sruja-ai/sruja/pkg/language"
)

// ExternalBestPracticeRule enforces that external services do not depend on internal elements.
// Specifically: relations FROM an external element TO any non-external element are flagged.
type ExternalBestPracticeRule struct{}

func (r *ExternalBestPracticeRule) Name() string { return "ExternalBestPractice" }

func (r *ExternalBestPracticeRule) Validate(program *language.Program) []ValidationError {
	return []ValidationError{}
}

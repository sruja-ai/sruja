package engine

import (
	"fmt"

	"github.com/sruja-ai/sruja/pkg/language"
)

type ValidationError struct {
	Message string
	Line    int
	Column  int
}

func (e ValidationError) String() string {
	return fmt.Sprintf("Error at line %d, col %d: %s", e.Line, e.Column, e.Message)
}

func (e ValidationError) Error() string {
	return e.String()
}

type Rule interface {
	Name() string
	Validate(program *language.Program) []ValidationError
}

type Validator struct {
	Rules []Rule
}

func NewValidator() *Validator {
	return &Validator{
		Rules: []Rule{},
	}
}

func (v *Validator) RegisterRule(rule Rule) {
	v.Rules = append(v.Rules, rule)
}

func (v *Validator) Validate(program *language.Program) []ValidationError {
	var errors []ValidationError
	for _, rule := range v.Rules {
		errs := rule.Validate(program)
		errors = append(errors, errs...)
	}
	return errors
}

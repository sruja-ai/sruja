package notebook

import (
	"fmt"
	"io/ioutil"
	"regexp"
	"strings"

	"github.com/sruja-ai/sruja/pkg/compiler"
	"github.com/sruja-ai/sruja/pkg/engine"
	"github.com/sruja-ai/sruja/pkg/language"
)

type Runner struct{}

func NewRunner() *Runner {
	return &Runner{}
}

// ProcessMarkdown reads a markdown file, finds sruja blocks, compiles them,
// and appends the generated mermaid diagram immediately after the block.
func (r *Runner) ProcessMarkdown(filePath string) error {
	content, err := ioutil.ReadFile(filePath)
	if err != nil {
		return err
	}

	markdown := string(content)

	// Regex to find ```sruja ... ``` blocks
	// This is a naive implementation. A proper markdown parser would be better for production.
	re := regexp.MustCompile("(?s)```sruja\\n(.*?)\\n```")

	newMarkdown := re.ReplaceAllStringFunc(markdown, func(match string) string {
		// Extract code
		code := strings.TrimPrefix(match, "```sruja\n")
		code = strings.TrimSuffix(code, "\n```")

		// Parse
		parser, err := language.NewParser()
		if err != nil {
			return fmt.Sprintf("%s\n\n> **Sruja Parser Error:**\n> %v\n", match, err)
		}
		program, err := parser.Parse("notebook.sruja", code)
		if err != nil {
			return fmt.Sprintf("%s\n\n> **Sruja Parse Errors:**\n> %v\n", match, err)
		}

		// Validation
		validator := engine.NewValidator()
		validator.RegisterRule(&engine.UniqueIDRule{})
		validator.RegisterRule(&engine.ValidReferenceRule{})
		validationErrors := validator.Validate(program)

		if len(validationErrors) > 0 {
			var errMsgs []string
			for _, err := range validationErrors {
				errMsgs = append(errMsgs, err.String())
			}
			return fmt.Sprintf("%s\n\n> **Sruja Validation Errors:**\n> %s\n", match, strings.Join(errMsgs, "\n> "))
		}

		c := compiler.NewMermaidCompiler()
		mermaidCode, err := c.Compile(program)
		if err != nil {
			return fmt.Sprintf("%s\n\n> **Sruja Compilation Error:**\n> %v\n", match, err)
		}

		// Return original block + generated mermaid block
		return fmt.Sprintf("%s\n\n```mermaid\n%s\n```", match, mermaidCode)
	})

	// Write back to file (or stdout, but for "notebook" feel, updating file is better)
	// For safety in this demo, I'll write to a new file or stdout.
	// Let's overwrite for the true "notebook" experience as requested.
	return ioutil.WriteFile(filePath, []byte(newMarkdown), 0644)
}

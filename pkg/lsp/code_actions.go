// pkg/lsp/code_actions.go
package lsp

import (
	"fmt"
	"strings"

	"github.com/sruja-ai/sruja/pkg/language"
)

// CodeActionProvider provides quick actions and code fixes for LSP.
type CodeActionProvider struct {
	semanticIndex    *SemanticIndex
	metadataRegistry *MetadataRegistry
}

// NewCodeActionProvider creates a new code action provider.
func NewCodeActionProvider(index *SemanticIndex, registry *MetadataRegistry) *CodeActionProvider {
	return &CodeActionProvider{
		semanticIndex:    index,
		metadataRegistry: registry,
	}
}

// CodeAction represents a code action that can be applied.
type CodeAction struct {
	Title       string         `json:"title"`
	Kind        string         `json:"kind,omitempty"`
	Diagnostics []interface{}  `json:"diagnostics,omitempty"`
	Edit        *WorkspaceEdit `json:"edit,omitempty"`
	Command     *Command       `json:"command,omitempty"`
}

// WorkspaceEdit represents edits to be made to a workspace.
type WorkspaceEdit struct {
	Changes map[string][]TextEdit `json:"changes"`
}

// TextEdit represents a text edit.
type TextEdit struct {
	Range   Range  `json:"range"`
	NewText string `json:"newText"`
}

// Command represents a command that can be executed.
type Command struct {
	Title     string        `json:"title"`
	Command   string        `json:"command"`
	Arguments []interface{} `json:"arguments,omitempty"`
}

// ProvideCodeActions provides code actions for a given context.
func (cap *CodeActionProvider) ProvideCodeActions(
	filePath string,
	text string,
	range_ Range,
	context CodeActionContext,
) ([]CodeAction, error) {
	var actions []CodeAction

	// Parse the file to understand context
	parser, err := language.NewParser()
	if err != nil {
		return nil, err
	}

	program, err := parser.Parse(filePath, text)
	if err != nil {
		// Even if parsing fails, we can provide some actions
		return cap.provideBasicActions(filePath, text, range_, context), nil
	}

	// Analyze context and provide relevant actions
	actions = append(actions, cap.provideElementActions(program, filePath, text, range_)...)
	actions = append(actions, cap.provideMetadataActions(program, filePath, text, range_)...)
	actions = append(actions, cap.provideImportActions(program, filePath, text, range_)...)

	return actions, nil
}

// CodeActionContext provides context for code actions.
type CodeActionContext struct {
	Diagnostics []interface{} `json:"diagnostics"`
	Only        []string      `json:"only,omitempty"`
}

// provideBasicActions provides basic code actions when parsing fails.
func (cap *CodeActionProvider) provideBasicActions(
	filePath string,
	text string,
	range_ Range,
	context CodeActionContext,
) []CodeAction {
	var actions []CodeAction

	// Add "Create architecture block" action if file seems empty or invalid
	lines := strings.Split(text, "\n")
	if len(lines) < 3 || !strings.Contains(text, "architecture") {
		actions = append(actions, CodeAction{
			Title: "Create architecture block",
			Kind:  "quickfix",
			Edit: &WorkspaceEdit{
				Changes: map[string][]TextEdit{
					filePath: {
						{
							Range: Range{
								Start: Position{Line: 0, Character: 0},
								End:   Position{Line: 0, Character: 0},
							},
							NewText: `architecture "My Architecture" {
  
}
`,
						},
					},
				},
			},
		})
	}

	return actions
}

// provideElementActions provides actions for creating elements.
func (cap *CodeActionProvider) provideElementActions(
	program *language.Program,
	filePath string,
	text string,
	range_ Range,
) []CodeAction {
	var actions []CodeAction

	if program == nil || program.Architecture == nil {
		return actions
	}

	// Get the line where cursor is
	lines := strings.Split(text, "\n")
	if range_.Start.Line < 0 || range_.Start.Line >= len(lines) {
		return actions
	}

	currentLine := lines[range_.Start.Line]
	indent := cap.detectIndent(currentLine)

	// Check if we're in a system block
	if cap.isInSystemBlock(lines, range_.Start.Line) {
		// Suggest creating a container
		actions = append(actions, CodeAction{
			Title: "Create container",
			Kind:  "refactor",
			Edit: &WorkspaceEdit{
				Changes: map[string][]TextEdit{
					filePath: {
						{
							Range: Range{
								Start: Position{Line: range_.Start.Line, Character: 0},
								End:   Position{Line: range_.Start.Line, Character: 0},
							},
							NewText: fmt.Sprintf("%scontainer ${1:ID} \"${2:Label}\" {\n%s  \n%s}\n", indent, indent, indent),
						},
					},
				},
			},
		})

		// Suggest creating a component
		actions = append(actions, CodeAction{
			Title: "Create component",
			Kind:  "refactor",
			Edit: &WorkspaceEdit{
				Changes: map[string][]TextEdit{
					filePath: {
						{
							Range: Range{
								Start: Position{Line: range_.Start.Line, Character: 0},
								End:   Position{Line: range_.Start.Line, Character: 0},
							},
							NewText: fmt.Sprintf("%scomponent ${1:ID} \"${2:Label}\" {\n%s  \n%s}\n", indent, indent, indent),
						},
					},
				},
			},
		})

		// Suggest creating a datastore
		actions = append(actions, CodeAction{
			Title: "Create datastore",
			Kind:  "refactor",
			Edit: &WorkspaceEdit{
				Changes: map[string][]TextEdit{
					filePath: {
						{
							Range: Range{
								Start: Position{Line: range_.Start.Line, Character: 0},
								End:   Position{Line: range_.Start.Line, Character: 0},
							},
							NewText: fmt.Sprintf("%sdatastore ${1:ID} \"${2:Label}\"\n", indent),
						},
					},
				},
			},
		})
	}

	// Check if we're in a container block
	if cap.isInContainerBlock(lines, range_.Start.Line) {
		// Suggest creating a component
		actions = append(actions, CodeAction{
			Title: "Create component",
			Kind:  "refactor",
			Edit: &WorkspaceEdit{
				Changes: map[string][]TextEdit{
					filePath: {
						{
							Range: Range{
								Start: Position{Line: range_.Start.Line, Character: 0},
								End:   Position{Line: range_.Start.Line, Character: 0},
							},
							NewText: fmt.Sprintf("%scomponent ${1:ID} \"${2:Label}\" {\n%s  \n%s}\n", indent, indent, indent),
						},
					},
				},
			},
		})
	}

	// Always suggest creating a system at top level
	if cap.isAtTopLevel(lines, range_.Start.Line) {
		actions = append(actions, CodeAction{
			Title: "Create system",
			Kind:  "refactor",
			Edit: &WorkspaceEdit{
				Changes: map[string][]TextEdit{
					filePath: {
						{
							Range: Range{
								Start: Position{Line: range_.Start.Line, Character: 0},
								End:   Position{Line: range_.Start.Line, Character: 0},
							},
							NewText: fmt.Sprintf("%ssystem ${1:ID} \"${2:Label}\" {\n%s  \n%s}\n", indent, indent, indent),
						},
					},
				},
			},
		})
	}

	return actions
}

// provideMetadataActions provides actions for adding metadata.
func (cap *CodeActionProvider) provideMetadataActions(
	program *language.Program,
	filePath string,
	text string,
	range_ Range,
) []CodeAction {
	var actions []CodeAction

	lines := strings.Split(text, "\n")
	if range_.Start.Line < 0 || range_.Start.Line >= len(lines) {
		return actions
	}

	currentLine := lines[range_.Start.Line]
	indent := cap.detectIndent(currentLine)

	// Check if we're in an element block (system, container, component)
	if cap.isInElementBlock(lines, range_.Start.Line) {
		// Check if metadata block already exists
		if !strings.Contains(strings.Join(lines, "\n"), "metadata {") {
			actions = append(actions, CodeAction{
				Title: "Add metadata block",
				Kind:  "refactor",
				Edit: &WorkspaceEdit{
					Changes: map[string][]TextEdit{
						filePath: {
							{
								Range: Range{
									Start: Position{Line: range_.Start.Line, Character: 0},
									End:   Position{Line: range_.Start.Line, Character: 0},
								},
								NewText: fmt.Sprintf("%smetadata {\n%s  ${1:key}: \"${2:value}\"\n%s}\n", indent, indent, indent),
							},
						},
					},
				},
			})
		}
	}

	return actions
}

// provideImportActions provides actions for adding imports.
func (cap *CodeActionProvider) provideImportActions(
	program *language.Program,
	filePath string,
	text string,
	range_ Range,
) []CodeAction {
	var actions []CodeAction

	// Suggest adding an import at the top of the architecture block
	if cap.isAtTopLevel(strings.Split(text, "\n"), range_.Start.Line) {
		actions = append(actions, CodeAction{
			Title: "Add import",
			Kind:  "refactor",
			Edit: &WorkspaceEdit{
				Changes: map[string][]TextEdit{
					filePath: {
						{
							Range: Range{
								Start: Position{Line: range_.Start.Line, Character: 0},
								End:   Position{Line: range_.Start.Line, Character: 0},
							},
							NewText: "import \"${1:path}\" as ${2:Alias}\n",
						},
					},
				},
			},
		})
	}

	return actions
}

// Helper functions

func (cap *CodeActionProvider) detectIndent(line string) string {
	for i, r := range line {
		if r != ' ' && r != '\t' {
			return line[:i]
		}
	}
	return ""
}

func (cap *CodeActionProvider) isInSystemBlock(lines []string, lineNum int) bool {
	depth := 0
	for i := 0; i <= lineNum && i < len(lines); i++ {
		line := strings.TrimSpace(lines[i])
		if strings.HasPrefix(line, "system ") {
			depth++
		}
		for _, r := range lines[i] {
			if r == '{' {
				depth++
			} else if r == '}' {
				depth--
			}
		}
	}
	return depth > 1
}

func (cap *CodeActionProvider) isInContainerBlock(lines []string, lineNum int) bool {
	depth := 0
	inContainer := false
	for i := 0; i <= lineNum && i < len(lines); i++ {
		line := strings.TrimSpace(lines[i])
		if strings.HasPrefix(line, "container ") {
			inContainer = true
			depth++
		} else if strings.HasPrefix(line, "system ") {
			depth++
		}
		for _, r := range lines[i] {
			if r == '{' {
				depth++
			} else if r == '}' {
				depth--
				if depth == 1 {
					inContainer = false
				}
			}
		}
	}
	return inContainer && depth > 1
}

func (cap *CodeActionProvider) isInElementBlock(lines []string, lineNum int) bool {
	return cap.isInSystemBlock(lines, lineNum) || cap.isInContainerBlock(lines, lineNum)
}

func (cap *CodeActionProvider) isAtTopLevel(lines []string, lineNum int) bool {
	depth := 0
	for i := 0; i <= lineNum && i < len(lines); i++ {
		for _, r := range lines[i] {
			if r == '{' {
				depth++
			} else if r == '}' {
				depth--
			}
		}
	}
	return depth <= 1
}

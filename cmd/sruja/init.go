package main

import (
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
	"github.com/sruja-ai/sruja/pkg/dx"
)

var initCmd = &cobra.Command{
	Use:   "init [project-name]",
	Short: "Initialize a new Sruja project",
	Long:  `Initialize a new Sruja project with a default directory structure and example files.`,
	Args:  cobra.MaximumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		if runInit(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("init failed")
		}
		return nil
	},
}

func runInit(args []string, stdout, stderr io.Writer) int {
	projectName := "my-sruja-project"
	if len(args) > 0 {
		projectName = args[0]
	}

	// Create project directory
	if err := os.MkdirAll(projectName, 0o750); err != nil {
		_, _ = fmt.Fprintf(stderr, "Error: failed to create project directory: %v\n", err)
		return 1
	}

	// Create main.sruja
	mainContent := `// Define your system here
	system MySystem "My System" {
		description "A new Sruja system"
	}`
	if err := os.WriteFile(filepath.Join(projectName, "main.sruja"), []byte(mainContent), 0o644); err != nil { //nolint:gosec // template file safe to be world-readable
		_, _ = fmt.Fprintf(stderr, "Error: failed to create main.sruja: %v\n", err)
		return 1
	}

	// Create README.md
	readmeContent := `# ` + projectName + `

This is a Sruja project.

## Getting Started

1. Install Sruja CLI
2. Run ` + "`sruja compile main.sruja`" + ` to compile
3. Run ` + "`sruja export json main.sruja`" + ` to export architecture
`
	if err := os.WriteFile(filepath.Join(projectName, "README.md"), []byte(readmeContent), 0o644); err != nil { //nolint:gosec // template file safe to be world-readable
		_, _ = fmt.Fprintf(stderr, "Error: failed to create README.md: %v\n", err)
		return 1
	}

	// Create .gitignore
	gitignoreContent := `
# Sruja generated files
output/
dist/
*.html
*.png
*.svg
`
	if err := os.WriteFile(filepath.Join(projectName, ".gitignore"), []byte(gitignoreContent), 0o644); err != nil { //nolint:gosec // template file safe to be world-readable
		_, _ = fmt.Fprintf(stderr, "Error: failed to create .gitignore: %v\n", err)
		return 1
	}

	_, _ = fmt.Fprintf(stdout, "%s\n", dx.Success(fmt.Sprintf("Initialized new Sruja project in %s", projectName)))
	return 0
}

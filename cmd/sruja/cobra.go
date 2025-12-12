package main

import (
	"fmt"
	"os"
	"strings"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:           "sruja",
	Short:         "Sruja DSL CLI",
	SilenceErrors: true,
	SilenceUsage:  true,
}

func init() {
	// Add flags to change create command (must be before adding to parent)
	cmdChangeCreate.Flags().String("requirement", "", "Requirement ID")
	cmdChangeCreate.Flags().String("owner", "", "Change owner")
	cmdChangeCreate.Flags().String("stakeholders", "", "Comma-separated list of stakeholders")

	// Add flags to import command
	cmdImport.Flags().String("output", "", "Output directory (default: current directory)")
	cmdImport.Flags().String("format", "single", "Output format: single (one file) or multiple (separate files)")

	// Add subcommands to change command
	cmdChange.AddCommand(cmdChangeCreate)
	cmdChange.AddCommand(cmdChangeValidate)

	rootCmd.AddCommand(cmdVersion)
	rootCmd.AddCommand(cmdCompile)
	rootCmd.AddCommand(cmdLint)
	rootCmd.AddCommand(cmdFmt)
	rootCmd.AddCommand(cmdExport)
	rootCmd.AddCommand(cmdImport)
	// cmdExportFolder removed - SVG export removed (Studio will provide)
	rootCmd.AddCommand(cmdExplain)
	rootCmd.AddCommand(cmdList)
	rootCmd.AddCommand(cmdTree)
	rootCmd.AddCommand(cmdDiff)
	rootCmd.AddCommand(cmdChange)
	rootCmd.AddCommand(cmdCompletion)
	rootCmd.AddCommand(cmdLSP)
	rootCmd.AddCommand(initCmd)
	rootCmd.AddCommand(cmdScore)
}

var cmdVersion = &cobra.Command{
	Use:                "version",
	Short:              "Print version",
	Long:               "Print the version number of sruja",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runVersion(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("version failed")
		}
		return nil
	},
}

var cmdCompile = &cobra.Command{
	Use:                "compile",
	Short:              "Compile a file",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runCompile(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("compile failed")
		}
		return nil
	},
}

var cmdLint = &cobra.Command{
	Use:                "lint",
	Short:              "Lint a file",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runLint(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("lint failed")
		}
		return nil
	},
}

var cmdFmt = &cobra.Command{
	Use:                "fmt",
	Short:              "Format a file",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runFmt(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("fmt failed")
		}
		return nil
	},
}

var cmdExport = &cobra.Command{
	Use:                "export",
	Short:              "Export to a format",
	Long:               "Export a .sruja file to various formats (json)",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runExport(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("export failed")
		}
		return nil
	},
}

func init() {
	// Note: HTML export removed from CLI - HTML export code exists in pkg/export/html
	// but is not accessible via CLI export command
	// Legacy flags kept for backward compatibility but unused
	_ = cmdExport.Flags().Bool("local", false, "(deprecated) Use local multi-file mode")
	_ = cmdExport.Flags().Bool("single-file", false, "(deprecated) Use single-file mode")
	_ = cmdExport.Flags().StringP("out", "o", "", "(deprecated) Output directory or file")
}

var cmdImport = &cobra.Command{
	Use:                "import",
	Short:              "Import from a format",
	Long:               "Import a .sruja file from various formats (json)",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runImport(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("import failed")
		}
		return nil
	},
}

// cmdExportFolder removed - SVG export removed (Studio will provide SVG/PNG export)

var cmdExplain = &cobra.Command{
	Use:                "explain",
	Short:              "Explain architecture elements",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runExplain(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("explain failed")
		}
		return nil
	},
}

var cmdList = &cobra.Command{
	Use:                "list",
	Short:              "List elements from a file",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runList(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("list failed")
		}
		return nil
	},
}

var cmdTree = &cobra.Command{
	Use:                "tree",
	Short:              "Print architecture tree",
	Long:               "Print the architecture hierarchy as a tree",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runTree(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("tree failed")
		}
		return nil
	},
}

var cmdDiff = &cobra.Command{
	Use:                "diff",
	Short:              "Diff",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runDiff(args, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("diff failed")
		}
		return nil
	},
}

var cmdChange = &cobra.Command{
	Use:   "change",
	Short: "Manage architectural changes",
}

var cmdChangeCreate = &cobra.Command{
	Use:   "create <change-name>",
	Short: "Create a new change file",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		requirement, _ := cmd.Flags().GetString("requirement")
		owner, _ := cmd.Flags().GetString("owner")
		stakeholdersStr, _ := cmd.Flags().GetString("stakeholders")
		var stakeholders []string
		if stakeholdersStr != "" {
			stakeholders = strings.Split(stakeholdersStr, ",")
			for i := range stakeholders {
				stakeholders[i] = strings.TrimSpace(stakeholders[i])
			}
		}
		if runChangeCreate(args[0], requirement, owner, stakeholders, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("change create failed")
		}
		return nil
	},
}

var cmdChangeValidate = &cobra.Command{
	Use:   "validate <change-file>",
	Short: "Validate a change file",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		if runChangeValidate(args[0], cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("change validate failed")
		}
		return nil
	},
}

var cmdCompletion = &cobra.Command{
	Use:   "completion [bash|zsh|fish]",
	Short: "Generate shell completions",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		switch args[0] {
		case "bash":
			return cmd.Root().GenBashCompletion(cmd.OutOrStdout())
		case "zsh":
			return cmd.Root().GenZshCompletion(cmd.OutOrStdout())
		case "fish":
			return cmd.Root().GenFishCompletion(cmd.OutOrStdout(), true)
		default:
			return fmt.Errorf("supported shells: bash zsh fish")
		}
	},
}

func Execute() int {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		return 1
	}
	return 0
}

package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:           "sruja",
	Short:         "Sruja DSL CLI",
	SilenceErrors: true,
	SilenceUsage:  true,
}

func init() {
	rootCmd.AddCommand(cmdVersion)
	rootCmd.AddCommand(cmdCompile)
	rootCmd.AddCommand(cmdLint)
	rootCmd.AddCommand(cmdFmt)
	rootCmd.AddCommand(cmdExport)
	rootCmd.AddCommand(cmdExportFolder)
	rootCmd.AddCommand(cmdExplain)
	rootCmd.AddCommand(cmdList)
	rootCmd.AddCommand(cmdTree)
	rootCmd.AddCommand(cmdDiff)
	rootCmd.AddCommand(cmdCompletion)
}

var cmdVersion = &cobra.Command{
	Use:   "version",
	Short: "Show version",
	RunE: func(cmd *cobra.Command, args []string) error {
		if runVersion(os.Stdout) != 0 {
			return fmt.Errorf("version failed")
		}
		return nil
	},
}

var cmdCompile = &cobra.Command{
	Use:                "compile [file]",
	Short:              "Compile a .sruja file",
	DisableFlagParsing: true,
	Args:               cobra.MinimumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		if runCompile(args[0], os.Stdout, os.Stderr) != 0 {
			return fmt.Errorf("compile failed")
		}
		return nil
	},
}

var cmdLint = &cobra.Command{
	Use:                "lint [file]",
	Short:              "Lint a .sruja file",
	DisableFlagParsing: true,
	Args:               cobra.MinimumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		if runLint(args[0], os.Stdout, os.Stderr) != 0 {
			return fmt.Errorf("lint failed")
		}
		return nil
	},
}

var cmdFmt = &cobra.Command{
	Use:                "fmt [file]",
	Short:              "Format a .sruja file",
	DisableFlagParsing: true,
	Args:               cobra.MinimumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		if runFmt(args[0], os.Stdout, os.Stderr) != 0 {
			return fmt.Errorf("fmt failed")
		}
		return nil
	},
}

var cmdExport = &cobra.Command{
	Use:                "export [format] [file]",
	Short:              "Export to a format",
	DisableFlagParsing: true,
	Args:               cobra.MinimumNArgs(2),
	RunE: func(cmd *cobra.Command, args []string) error {
		if runExport(args[0], args[1], os.Stdout, os.Stderr) != 0 {
			return fmt.Errorf("export failed")
		}
		return nil
	},
}

var cmdExportFolder = &cobra.Command{
	Use:                "export-folder [folder] [output-dir]",
	Short:              "Export all .sruja files in a folder to SVG (handles imports)",
	Long:               "Recursively finds all .sruja files in the specified folder, resolves imports, and generates SVG files. Output directory defaults to 'svg-output' in the source folder.",
	DisableFlagParsing: true,
	Args:               cobra.MinimumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		outputDir := ""
		if len(args) >= 2 {
			outputDir = args[1]
		}
		if runExportFolder(args[0], outputDir, os.Stdout, os.Stderr) != 0 {
			return fmt.Errorf("export-folder failed")
		}
		return nil
	},
}

var cmdExplain = &cobra.Command{
	Use:                "explain",
	Short:              "Explain architecture elements",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runExplain(os.Stdout, os.Stderr) != 0 {
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
		if runList(os.Stdout, os.Stderr) != 0 {
			return fmt.Errorf("list failed")
		}
		return nil
	},
}

var cmdTree = &cobra.Command{
	Use:                "tree",
	Short:              "Show tree",
	DisableFlagParsing: true,
	RunE: func(cmd *cobra.Command, args []string) error {
		if runTree(os.Stdout, os.Stderr) != 0 {
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
		if runDiff(os.Stdout, os.Stderr) != 0 {
			return fmt.Errorf("diff failed")
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
			return cmd.Root().GenBashCompletion(os.Stdout)
		case "zsh":
			return cmd.Root().GenZshCompletion(os.Stdout)
		case "fish":
			return cmd.Root().GenFishCompletion(os.Stdout, true)
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

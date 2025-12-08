package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
	slsp "github.com/sruja-ai/sruja/pkg/lsp"
)

var cmdLSP = &cobra.Command{
	Use:   "lsp",
	Short: "Start Sruja LSP server (stdio)",
	RunE: func(_ *cobra.Command, _ []string) error {
		if err := slsp.StartStdioServer(); err != nil {
			fmt.Fprintln(os.Stderr, err)
			return fmt.Errorf("lsp server failed")
		}
		return nil
	},
}

func init() {
	// Accept and ignore the --stdio flag commonly passed by language clients
	cmdLSP.Flags().Bool("stdio", true, "Use stdio transport")
}

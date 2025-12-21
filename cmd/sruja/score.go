package main

import (
	"fmt"
	"io"

	"github.com/spf13/cobra"
	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/engine"
)

var cmdScore = &cobra.Command{
	Use:   "score [file]",
	Short: "Calculate architecture health score",
	Long:  `Analyze the architecture and calculate a health score (0-100) based on best practices and validation rules.`,
	Args:  cobra.MaximumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		filePath := ""
		if len(args) > 0 {
			filePath = args[0]
		}
		if runScore(filePath, cmd.OutOrStdout(), cmd.ErrOrStderr()) != 0 {
			return fmt.Errorf("score failed")
		}
		return nil
	},
}

func runScore(filePath string, stdout, stderr io.Writer) int {
	// 1. Find file
	targetFile := findSrujaFile(filePath)
	if targetFile == "" {
		fmt.Fprintln(stderr, dx.Error("No .sruja file found. Please specify a file or run in a directory with a .sruja file."))
		return 1
	}

	// 2. Parse
	program, err := parseArchitectureFile(targetFile, stderr)
	if err != nil {
		return 1
	}

	// 3. Score
	scorer := engine.NewScorer()
	card := scorer.CalculateScore(program)

	// 4. Report
	useColor := dx.SupportsColor()
	scoreStr := fmt.Sprintf("%d", card.Score)
	//nolint:gocritic // if-else chain preferred for readability here
	if card.Score < 60 {
		scoreStr = dx.Colorize(dx.ColorRed, scoreStr, useColor)
	} else if card.Score < 80 {
		scoreStr = dx.Colorize(dx.ColorYellow, scoreStr, useColor)
	} else {
		scoreStr = dx.Colorize(dx.ColorGreen, scoreStr, useColor)
	}

	fmt.Fprintln(stdout, dx.Bold(fmt.Sprintf("Architecture Score: %s (%s)", scoreStr, card.Grade)))
	fmt.Fprintln(stdout, "")

	if len(card.Deductions) > 0 {
		fmt.Fprintln(stdout, dx.Bold("Deductions:"))
		for _, d := range card.Deductions {
			pointsStr := dx.Colorize(dx.ColorRed, fmt.Sprintf("-%d pts", d.Points), useColor)
			ruleStr := dx.Dim(d.Rule)
			fmt.Fprintf(stdout, "- [%s] %s (%s)\n", pointsStr, d.Message, ruleStr)
		}
	} else {
		fmt.Fprintln(stdout, dx.Success("Perfect Score! No deductions."))
	}

	return 0
}

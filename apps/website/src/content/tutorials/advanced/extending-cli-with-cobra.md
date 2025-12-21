---
title: "Extending the CLI with Cobra"
weight: 80
summary: "Add new subcommands using Cobra and reuse existing run functions."
tags: ["cli", "go", "cobra"]
---

# Extending the CLI with Cobra

Sruja’s CLI uses Cobra. This tutorial shows how to add a new subcommand.

## Define a Command

```go
var cmdHello = &cobra.Command{
  Use:   "hello",
  Short: "Say hello",
  RunE: func(cmd *cobra.Command, args []string) error {
    fmt.Fprintln(os.Stdout, "hello")
    return nil
  },
}
```

## Wire It In

```go
func init() {
  rootCmd.AddCommand(cmdHello)
}
```

## Generate Completions

```bash
sruja completion bash
sruja completion zsh
sruja completion fish
```

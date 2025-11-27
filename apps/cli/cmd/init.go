// apps/cli/cmd/init.go
package main

import (
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/sruja-ai/sruja/pkg/dx"
	"github.com/sruja-ai/sruja/pkg/templates"
)

func runInit() {
	initCmd := flag.NewFlagSet("init", flag.ExitOnError)
	template := initCmd.String("template", "basic", "template to use: basic, microservices, event-driven, monolith, api-gateway, service-mesh")
	dir := initCmd.String("dir", ".", "directory to initialize")
	initCmd.Parse(os.Args[2:])

	// Resolve directory
	targetDir := *dir
	if targetDir == "." {
		var err error
		targetDir, err = os.Getwd()
		if err != nil {
			fmt.Println(dx.Error(fmt.Sprintf("Error getting current directory: %v", err)))
			os.Exit(1)
		}
	}

	// Check if directory is empty (allow .sruja files)
	entries, err := os.ReadDir(targetDir)
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error reading directory: %v", err)))
		os.Exit(1)
	}

	hasOtherFiles := false
	for _, entry := range entries {
		if !entry.IsDir() && !strings.HasSuffix(entry.Name(), ".sruja") && entry.Name() != ".git" {
			hasOtherFiles = true
			break
		}
	}

	if hasOtherFiles {
		fmt.Println(dx.Warning("Directory is not empty. Files will be created alongside existing files."))
	}

	// Get template
	tmpl, err := templates.GetTemplate(*template)
	if err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error loading template '%s': %v", *template, err)))
		fmt.Println(dx.Info("Available templates: basic, microservices, event-driven, monolith, api-gateway, service-mesh"))
		os.Exit(1)
	}

	// Generate files
	fmt.Println(dx.Info(fmt.Sprintf("Initializing Sruja project in %s...", targetDir)))
	fmt.Println(dx.Info(fmt.Sprintf("Using template: %s", *template)))

	if err := tmpl.Generate(targetDir); err != nil {
		fmt.Println(dx.Error(fmt.Sprintf("Error generating project: %v", err)))
		os.Exit(1)
	}

	fmt.Println(dx.Success("Project initialized successfully!"))
	fmt.Println()
	fmt.Println("Next steps:")
	fmt.Println("  1. Review the generated architecture.sruja file")
	fmt.Println("  2. Run: sruja compile architecture.sruja")
	fmt.Println("  3. Run: sruja lint architecture.sruja")
	fmt.Println("  4. Open in your editor with Sruja LSP support")
}

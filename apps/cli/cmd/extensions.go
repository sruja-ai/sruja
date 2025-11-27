package main

import (
	"fmt"
	"os"

	"github.com/sruja-ai/sruja/pkg/extensions"
)

func runInstall() {
	// Load manifest
	manifest, err := extensions.LoadManifest("sruja.json")
	if err != nil {
		fmt.Printf("Error loading sruja.json: %v\n", err)
		fmt.Println("Tip: Run this command in a directory with a sruja.json file")
		os.Exit(1)
	}

	// Create package manager
	pm, err := extensions.NewPackageManager()
	if err != nil {
		fmt.Printf("Error initializing package manager: %v\n", err)
		os.Exit(1)
	}

	// Install all dependencies
	fmt.Println("Installing dependencies...")
	if err := pm.InstallAll(manifest); err != nil {
		fmt.Printf("Error installing dependencies: %v\n", err)
		os.Exit(1)
	}

	fmt.Println("\nAll dependencies installed successfully!")
}

func runUpdate() {
	// Load manifest
	manifest, err := extensions.LoadManifest("sruja.json")
	if err != nil {
		fmt.Printf("Error loading sruja.json: %v\n", err)
		os.Exit(1)
	}

	// Create package manager
	pm, err := extensions.NewPackageManager()
	if err != nil {
		fmt.Printf("Error initializing package manager: %v\n", err)
		os.Exit(1)
	}

	// Update all dependencies
	fmt.Println("Updating dependencies...")
	for name, refStr := range manifest.Imports {
		fmt.Printf("Updating %s...\n", name)

		ref, err := extensions.ParsePackageRef(refStr)
		if err != nil {
			fmt.Printf("Error parsing reference for %s: %v\n", name, err)
			continue
		}

		// Remove cached version
		pkgDir := fmt.Sprintf("%s/%s/%s/%s@%s", pm.CacheDir, ref.Host, ref.Owner, ref.Repo, ref.Version)
		os.RemoveAll(pkgDir)

		// Re-download
		if _, err := pm.Install(ref); err != nil {
			fmt.Printf("Error updating %s: %v\n", name, err)
			continue
		}
	}

	fmt.Println("\nAll dependencies updated successfully!")
}

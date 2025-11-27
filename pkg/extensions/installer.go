package extensions

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
)

// PackageManager handles downloading and caching packages
type PackageManager struct {
	CacheDir string // e.g., ~/.sruja/extensions
}

// NewPackageManager creates a new package manager
func NewPackageManager() (*PackageManager, error) {
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return nil, fmt.Errorf("failed to get home directory: %w", err)
	}

	cacheDir := filepath.Join(homeDir, ".sruja", "extensions")
	
	// Ensure cache directory exists
	if err := os.MkdirAll(cacheDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create cache directory: %w", err)
	}

	return &PackageManager{
		CacheDir: cacheDir,
	}, nil
}

// Install downloads and caches a package
// Supports both public and private repositories (uses git credentials/SSH)
func (pm *PackageManager) Install(ref *PackageRef) (string, error) {
	// Package cache path: ~/.sruja/extensions/host/owner/repo@version
	pkgDir := filepath.Join(pm.CacheDir, ref.Host, ref.Owner, fmt.Sprintf("%s@%s", ref.Repo, ref.Version))

	// Check if already cached
	if _, err := os.Stat(pkgDir); err == nil {
		fmt.Printf("Package already cached: %s\n", ref.String())
		return pkgDir, nil
	}

	fmt.Printf("Downloading package: %s\n", ref.String())

	// Create parent directory
	parentDir := filepath.Dir(pkgDir)
	if err := os.MkdirAll(parentDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create parent directory: %w", err)
	}

	// Clone repository
	// This will use the user's git credentials (SSH keys, credential helper, etc.)
	// Works for both public and private repositories
	cmd := exec.Command("git", "clone", "--depth", "1", "--branch", ref.Version, ref.GitURL(), pkgDir)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	
	if err := cmd.Run(); err != nil {
		return "", fmt.Errorf("failed to clone repository: %w (ensure you have access to the repository)", err)
	}

	fmt.Printf("Successfully downloaded: %s\n", ref.String())
	return pkgDir, nil
}

// InstallAll installs all dependencies from a manifest
func (pm *PackageManager) InstallAll(manifest *Manifest) error {
	for name, refStr := range manifest.Imports {
		fmt.Printf("Installing %s...\n", name)
		
		ref, err := ParsePackageRef(refStr)
		if err != nil {
			return fmt.Errorf("failed to parse reference for %s: %w", name, err)
		}

		if _, err := pm.Install(ref); err != nil {
			return fmt.Errorf("failed to install %s: %w", name, err)
		}
	}

	return nil
}

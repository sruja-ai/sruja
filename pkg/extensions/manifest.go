package extensions

import (
	"encoding/json"
	"fmt"
	"os"
)

// PackageType defines whether this is a project or extension
type PackageType string

const (
	PackageTypeProject   PackageType = "project"
	PackageTypeExtension PackageType = "extension"
)

// Manifest represents a sruja.json file
type Manifest struct {
	Name    string            `json:"name"`
	Version string            `json:"version"`
	Type    PackageType       `json:"type"`
	Imports map[string]string `json:"imports,omitempty"` // name -> git URL with version
	Exports *ExportsConfig    `json:"exports,omitempty"`
}

// ExportsConfig defines what an extension exports
type ExportsConfig struct {
	Rules     []string `json:"rules,omitempty"`     // Glob patterns for rule files
	Compilers []string `json:"compilers,omitempty"` // Glob patterns for compiler files
}

// LoadManifest reads and parses a sruja.json file
func LoadManifest(path string) (*Manifest, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read manifest: %w", err)
	}

	var manifest Manifest
	if err := json.Unmarshal(data, &manifest); err != nil {
		return nil, fmt.Errorf("failed to parse manifest: %w", err)
	}

	return &manifest, nil
}

// SaveManifest writes a manifest to a sruja.json file
func SaveManifest(path string, manifest *Manifest) error {
	data, err := json.MarshalIndent(manifest, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal manifest: %w", err)
	}

	if err := os.WriteFile(path, data, 0644); err != nil {
		return fmt.Errorf("failed to write manifest: %w", err)
	}

	return nil
}

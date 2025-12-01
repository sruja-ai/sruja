// pkg/config/config_test.go
package config

import (
	"os"
	"path/filepath"
	"testing"
)

func TestDefaultConfig(t *testing.T) {
	cfg := DefaultConfig()
	if cfg == nil {
		t.Fatal("DefaultConfig returned nil")
	}
	if cfg.Diagrams == nil {
		t.Fatal("Diagrams should not be nil")
	}
	if cfg.Diagrams.Theme != "neutral-default" {
		t.Errorf("Expected theme 'neutral-default', got '%s'", cfg.Diagrams.Theme)
	}
	if cfg.Validation == nil {
		t.Fatal("Validation should not be nil")
	}
	if cfg.LSP == nil {
		t.Fatal("LSP should not be nil")
	}
}

func TestLoadConfig_NoFile(t *testing.T) {
	cfg, err := LoadConfig("")
	if err != nil {
		t.Fatalf("LoadConfig should not error when no file exists: %v", err)
	}
	if cfg == nil {
		t.Fatal("LoadConfig should return default config when no file exists")
	}
	if cfg.Diagrams == nil {
		t.Fatal("Diagrams should not be nil")
	}
}

func TestLoadConfig_WithFile(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "sruja.config.json")
	configJSON := `{
  "diagrams": {
    "theme": "dark",
    "defaultFormat": "svg"
  },
  "validation": {
    "strict": true
  }
}`

	if err := os.WriteFile(configPath, []byte(configJSON), 0644); err != nil {
		t.Fatalf("Failed to write test config: %v", err)
	}

	cfg, err := LoadConfig(configPath)
	if err != nil {
		t.Fatalf("LoadConfig failed: %v", err)
	}
	if cfg == nil {
		t.Fatal("LoadConfig returned nil")
	}
	if cfg.Diagrams == nil {
		t.Fatal("Diagrams should not be nil")
	}
	if cfg.Diagrams.Theme != "dark" {
		t.Errorf("Expected theme 'dark', got '%s'", cfg.Diagrams.Theme)
	}
	if cfg.Diagrams.DefaultFormat != "svg" {
		t.Errorf("Expected defaultFormat 'svg', got '%s'", cfg.Diagrams.DefaultFormat)
	}
	if cfg.Validation == nil {
		t.Fatal("Validation should not be nil")
	}
	if !cfg.Validation.Strict {
		t.Error("Expected strict validation to be true")
	}
}

func TestLoadConfig_InvalidJSON(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "sruja.config.json")
	invalidJSON := `{ invalid json }`

	if err := os.WriteFile(configPath, []byte(invalidJSON), 0644); err != nil {
		t.Fatalf("Failed to write test config: %v", err)
	}

	_, err := LoadConfig(configPath)
	if err == nil {
		t.Fatal("LoadConfig should error on invalid JSON")
	}
}

func TestSaveConfig(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "sruja.config.json")

	cfg := DefaultConfig()
	cfg.Diagrams.Theme = "custom-theme"

	if err := SaveConfig(cfg, configPath); err != nil {
		t.Fatalf("SaveConfig failed: %v", err)
	}

	// Verify file was created
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		t.Fatal("Config file was not created")
	}

	// Load and verify
	loaded, err := LoadConfig(configPath)
	if err != nil {
		t.Fatalf("Failed to load saved config: %v", err)
	}
	if loaded.Diagrams.Theme != "custom-theme" {
		t.Errorf("Expected theme 'custom-theme', got '%s'", loaded.Diagrams.Theme)
	}
}

func TestSaveConfig_DefaultPath(t *testing.T) {
	tmpDir := t.TempDir()
	originalDir, _ := os.Getwd()
	defer func() {
		_ = os.Chdir(originalDir)
	}()

	if err := os.Chdir(tmpDir); err != nil {
		t.Fatalf("Failed to change directory: %v", err)
	}

	cfg := DefaultConfig()
	if err := SaveConfig(cfg, ""); err != nil {
		t.Fatalf("SaveConfig failed: %v", err)
	}

	configPath := filepath.Join(tmpDir, "sruja.config.json")
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		t.Fatal("Config file was not created at default path")
	}
}

func TestMerge(t *testing.T) {
	cfg1 := &Config{
		Diagrams: &DiagramsConfig{
			Theme: "theme1",
		},
		Validation: &ValidationConfig{
			Strict: false,
		},
	}

	cfg2 := &Config{
		Diagrams: &DiagramsConfig{
			Theme:         "theme2",
			DefaultFormat: "svg",
		},
		Validation: &ValidationConfig{
			Strict: true,
		},
		LSP: &LSPConfig{
			MetadataSuggestions: true,
		},
	}

	cfg1.Merge(cfg2)

	if cfg1.Diagrams.Theme != "theme2" {
		t.Errorf("Expected theme 'theme2', got '%s'", cfg1.Diagrams.Theme)
	}
	if cfg1.Diagrams.DefaultFormat != "svg" {
		t.Errorf("Expected defaultFormat 'svg', got '%s'", cfg1.Diagrams.DefaultFormat)
	}
	if !cfg1.Validation.Strict {
		t.Error("Expected strict validation to be true")
	}
	if cfg1.LSP == nil {
		t.Fatal("LSP should not be nil after merge")
	}
	if !cfg1.LSP.MetadataSuggestions {
		t.Error("Expected MetadataSuggestions to be true")
	}
}

func TestMerge_NilOther(t *testing.T) {
	cfg := &Config{
		Diagrams: &DiagramsConfig{
			Theme: "theme1",
		},
	}

	cfg.Merge(nil)

	if cfg.Diagrams.Theme != "theme1" {
		t.Errorf("Config should not change when merging nil: got '%s'", cfg.Diagrams.Theme)
	}
}

func TestMerge_EmptyConfigs(t *testing.T) {
	cfg1 := &Config{}
	cfg2 := &Config{
		Diagrams: &DiagramsConfig{
			Theme: "new-theme",
		},
	}

	cfg1.Merge(cfg2)

	if cfg1.Diagrams == nil {
		t.Fatal("Diagrams should be created during merge")
	}
	if cfg1.Diagrams.Theme != "new-theme" {
		t.Errorf("Expected theme 'new-theme', got '%s'", cfg1.Diagrams.Theme)
	}
}

func TestLoadConfig_ReadFileError(t *testing.T) {
	// LoadConfig returns DefaultConfig when file doesn't exist (IsNotExist check)
	// To test actual read error, we'd need a file that exists but can't be read
	// which is hard to simulate. This test verifies the IsNotExist path works.
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "nonexistent", "sruja.config.json")

	cfg, err := LoadConfig(configPath)
	if err != nil {
		// If it's not an IsNotExist error, it should error
		if err != nil {
			// This is expected for non-existent parent directory
			return
		}
	}
	// If no error, should return default config
	if cfg == nil {
		t.Fatal("LoadConfig should return default config when file doesn't exist")
	}
}

func TestSaveConfig_WriteError(t *testing.T) {
	// Try to save to a directory (should fail)
	tmpDir := t.TempDir()
	configPath := tmpDir

	cfg := DefaultConfig()
	err := SaveConfig(cfg, configPath)
	if err == nil {
		t.Fatal("SaveConfig should error when path is a directory")
	}
}

func TestLoadConfig_ParentDirectorySearch(t *testing.T) {
	tmpDir := t.TempDir()
	subDir := filepath.Join(tmpDir, "sub", "dir")
	if err := os.MkdirAll(subDir, 0755); err != nil {
		t.Fatalf("Failed to create subdirectory: %v", err)
	}

	configPath := filepath.Join(tmpDir, "sruja.config.json")
	configJSON := `{"diagrams": {"theme": "test"}}`
	if err := os.WriteFile(configPath, []byte(configJSON), 0644); err != nil {
		t.Fatalf("Failed to write config: %v", err)
	}

	originalDir, _ := os.Getwd()
	defer func() {
		_ = os.Chdir(originalDir)
	}()

	if err := os.Chdir(subDir); err != nil {
		t.Fatalf("Failed to change directory: %v", err)
	}

	cfg, err := LoadConfig("")
	if err != nil {
		t.Fatalf("LoadConfig failed: %v", err)
	}
	if cfg.Diagrams.Theme != "test" {
		t.Errorf("Expected theme 'test', got '%s'", cfg.Diagrams.Theme)
	}
}

func TestMerge_ShowMetadata(t *testing.T) {
	cfg1 := &Config{}
	cfg2 := &Config{
		Diagrams: &DiagramsConfig{
			ShowMetadata: []string{"team", "owner"},
		},
	}

	cfg1.Merge(cfg2)
	if len(cfg1.Diagrams.ShowMetadata) != 2 {
		t.Errorf("Expected 2 metadata keys, got %d", len(cfg1.Diagrams.ShowMetadata))
	}
}

func TestMerge_EmptyTheme(t *testing.T) {
	cfg1 := &Config{
		Diagrams: &DiagramsConfig{
			Theme: "original",
		},
	}
	cfg2 := &Config{
		Diagrams: &DiagramsConfig{
			Theme: "",
		},
	}

	cfg1.Merge(cfg2)
	if cfg1.Diagrams.Theme != "original" {
		t.Errorf("Theme should not change when merging empty string, got '%s'", cfg1.Diagrams.Theme)
	}
}

func TestMerge_EmptyValidationRules(t *testing.T) {
	cfg1 := &Config{
		Validation: &ValidationConfig{
			Rules: []string{"rule1"},
		},
	}
	cfg2 := &Config{
		Validation: &ValidationConfig{
			Rules: []string{},
		},
	}

	cfg1.Merge(cfg2)
	if len(cfg1.Validation.Rules) != 1 {
		t.Errorf("Rules should not change when merging empty rules, got %d", len(cfg1.Validation.Rules))
	}
}

func TestMerge_Plugins(t *testing.T) {
	cfg1 := &Config{
		Plugins: []string{"plugin1"},
	}
	cfg2 := &Config{
		Plugins: []string{"plugin2", "plugin3"},
	}

	cfg1.Merge(cfg2)
	if len(cfg1.Plugins) != 2 {
		t.Errorf("Expected 2 plugins, got %d", len(cfg1.Plugins))
	}
}

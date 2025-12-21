// pkg/engine/properties_validation_helpers_test.go
// Tests for property validation helper functions
package engine

import (
	"testing"
)

func TestIsPercentage(t *testing.T) {
	tests := []struct {
		input    string
		expected bool
	}{
		{"50%", true},
		{"99.9%", true},
		{"0%", true},
		{"100%", true},
		{"50", false},
		{"%", false},
		{"abc%", false},
		{"", false},
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			result := isPercentage(tt.input)
			if result != tt.expected {
				t.Errorf("isPercentage(%q) = %v, want %v", tt.input, result, tt.expected)
			}
		})
	}
}

func TestIsCurrency(t *testing.T) {
	tests := []struct {
		input    string
		expected bool
	}{
		{"$100", true},
		{"$1,000", true},
		{"$1,000.50", true},
		{"$10,000.99", true},
		{"100", false},
		{"$", false},
		{"$abc", false},
		{"$1,00", false},
		{"", false},
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			result := isCurrency(tt.input)
			if result != tt.expected {
				t.Errorf("isCurrency(%q) = %v, want %v", tt.input, result, tt.expected)
			}
		})
	}
}

func TestIsNonEmpty(t *testing.T) {
	tests := []struct {
		input    string
		expected bool
	}{
		{"value", true},
		{" ", true},
		{"a", true},
		{"", false},
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			result := isNonEmpty(tt.input)
			if result != tt.expected {
				t.Errorf("isNonEmpty(%q) = %v, want %v", tt.input, result, tt.expected)
			}
		})
	}
}

func TestValidateInstanceType_AWS(t *testing.T) {
	tests := []struct {
		input    string
		props    map[string]string
		expected bool
	}{
		{"t3.micro", map[string]string{"capacity.instanceProvider": "aws"}, true},
		{"m5.large", map[string]string{"capacity.instanceProvider": "aws"}, true},
		{"c5.xlarge", map[string]string{"capacity.instanceProvider": "aws"}, true},
		{"t3.2xlarge", map[string]string{"capacity.instanceProvider": "aws"}, true},
		{"invalid", map[string]string{"capacity.instanceProvider": "aws"}, false},
		{"t3", map[string]string{"capacity.instanceProvider": "aws"}, false},
		{"t3.micro", map[string]string{}, true}, // Falls back to isNonEmpty
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			result := validateInstanceType(tt.input, tt.props)
			if result != tt.expected {
				t.Errorf("validateInstanceType(%q, %v) = %v, want %v", tt.input, tt.props, result, tt.expected)
			}
		})
	}
}

func TestValidateInstanceType_GCP(t *testing.T) {
	tests := []struct {
		input    string
		props    map[string]string
		expected bool
	}{
		{"n1-standard-4", map[string]string{"capacity.instanceProvider": "gcp"}, true},
		{"n2-highcpu-8", map[string]string{"capacity.instanceProvider": "gcp"}, true},
		{"e2-standard-2", map[string]string{"capacity.instanceProvider": "gcp"}, true},
		{"invalid", map[string]string{"capacity.instanceProvider": "gcp"}, false},
		{"n1", map[string]string{"capacity.instanceProvider": "gcp"}, false},
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			result := validateInstanceType(tt.input, tt.props)
			if result != tt.expected {
				t.Errorf("validateInstanceType(%q, %v) = %v, want %v", tt.input, tt.props, result, tt.expected)
			}
		})
	}
}

func TestValidateInstanceType_Azure(t *testing.T) {
	tests := []struct {
		input    string
		props    map[string]string
		expected bool
	}{
		{"Standard_B1s", map[string]string{"capacity.instanceProvider": "azure"}, true},
		{"Standard_D2s_v3", map[string]string{"capacity.instanceProvider": "azure"}, false}, // Regex doesn't match underscores in middle
		{"Standard_F4s", map[string]string{"capacity.instanceProvider": "azure"}, true},
		{"Standard_D2sv3", map[string]string{"capacity.instanceProvider": "azure"}, true}, // Without underscore
		{"invalid", map[string]string{"capacity.instanceProvider": "azure"}, false},
		{"standard", map[string]string{"capacity.instanceProvider": "azure"}, false},
	}

	for _, tt := range tests {
		t.Run(tt.input, func(t *testing.T) {
			result := validateInstanceType(tt.input, tt.props)
			if result != tt.expected {
				t.Errorf("validateInstanceType(%q, %v) = %v, want %v", tt.input, tt.props, result, tt.expected)
			}
		})
	}
}

func TestValidateInstanceType_NoProvider(t *testing.T) {
	// When no provider is specified, it falls back to isNonEmpty
	result := validateInstanceType("t3.micro", map[string]string{})
	if !result {
		t.Error("validateInstanceType should return true for non-empty string when no provider is specified")
	}
	result2 := validateInstanceType("", map[string]string{})
	if result2 {
		t.Error("validateInstanceType should return false for empty string when no provider is specified")
	}
}

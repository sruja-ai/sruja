package engine

import (
	"testing"
)

func TestGetStringBuilder(t *testing.T) {
	sb := GetStringBuilder()
	if sb == nil {
		t.Fatal("GetStringBuilder returned nil")
	}

	sb.WriteString("test")
	result := sb.String()
	if result != "test" {
		t.Errorf("Expected 'test', got '%s'", result)
	}

	PutStringBuilder(sb)

	// Get another and ensure it's reset
	sb2 := GetStringBuilder()
	if sb2.Len() != 0 {
		t.Error("Pooled builder was not reset")
	}
	PutStringBuilder(sb2)
}

func TestGetDiagnosticsSlice(t *testing.T) {
	s := GetDiagnosticsSlice()
	if s == nil {
		t.Fatal("GetDiagnosticsSlice returned nil")
	}

	if len(*s) != 0 {
		t.Error("Slice should be empty")
	}

	PutDiagnosticsSlice(s)

	// Get another and ensure it's reset
	s2 := GetDiagnosticsSlice()
	if len(*s2) != 0 {
		t.Error("Pooled slice was not reset")
	}
	PutDiagnosticsSlice(s2)
}

func TestGetByteSlice(t *testing.T) {
	b := GetByteSlice()
	if b == nil {
		t.Fatal("GetByteSlice returned nil")
	}

	if len(*b) != 0 {
		t.Error("Slice should be empty")
	}

	*b = append(*b, []byte("test")...)
	if string(*b) != "test" {
		t.Errorf("Expected 'test', got '%s'", string(*b))
	}

	PutByteSlice(b)
}

func TestBuildQualifiedID(t *testing.T) {
	tests := []struct {
		name     string
		parts    []string
		expected string
	}{
		{"empty", []string{}, ""},
		{"single", []string{"system"}, "system"},
		{"two", []string{"system", "container"}, "system.container"},
		{"three", []string{"system", "container", "component"}, "system.container.component"},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			result := BuildQualifiedID(tc.parts...)
			if result != tc.expected {
				t.Errorf("Expected '%s', got '%s'", tc.expected, result)
			}
		})
	}
}

func TestValidatorWithOptions(t *testing.T) {
	// Test with default rules
	v1 := NewValidatorWithOptions(WithDefaultRules())
	if len(v1.Rules) == 0 {
		t.Error("Expected default rules to be registered")
	}

	// Test with specific rules
	v2 := NewValidatorWithOptions(WithRules(&CycleDetectionRule{}))
	if len(v2.Rules) != 1 {
		t.Errorf("Expected 1 rule, got %d", len(v2.Rules))
	}

	// Test empty validator
	v3 := NewValidatorWithOptions()
	if len(v3.Rules) != 0 {
		t.Error("Expected empty validator")
	}
}

func TestScorerWithOptions(t *testing.T) {
	scorer := NewScorerWithOptions()
	if scorer == nil {
		t.Fatal("NewScorerWithOptions returned nil")
	}
}

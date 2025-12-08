package svg

import (
	"testing"
)

func TestWrapLabel(t *testing.T) {
	tests := []struct {
		name     string
		text     string
		maxChars int
		want     []string
	}{
		{
			name:     "empty string",
			text:     "",
			maxChars: 10,
			want:     []string{},
		},
		{
			name:     "whitespace only",
			text:     "   ",
			maxChars: 10,
			want:     []string{},
		},
		{
			name:     "single short word",
			text:     "Hello",
			maxChars: 10,
			want:     []string{"Hello"},
		},
		{
			name:     "multiple words fitting",
			text:     "Hello World",
			maxChars: 20,
			want:     []string{"Hello World"},
		},
		{
			name:     "words needing split",
			text:     "Hello World From Testing",
			maxChars: 15,
			want:     []string{"Hello World", "From Testing"},
		},
		{
			name:     "very long single word - hard split",
			text:     "VeryLongWordThatNeedsToBeHardSplit",
			maxChars: 10,
			want:     []string{"VeryLongWo", "rdThatNeed", "…"},
		},
		{
			name:     "maxChars too small (normalized to 3)",
			text:     "Test",
			maxChars: 2,
			want:     []string{"Tes", "t"},
		},
		{
			name:     "maxChars at minimum (3)",
			text:     "Testing",
			maxChars: 3,
			want:     []string{"Tes", "tin", "g"},
		},
		{
			name:     "truncation to 3 lines",
			text:     "One Two Three Four Five Six Seven",
			maxChars: 5,
			want:     []string{"One", "Two", "…"},
		},
		{
			name:     "mixed short and long words",
			text:     "Short VeryLongWordIndeed More",
			maxChars: 10,
			want:     []string{"Short", "VeryLongWo", "…"},
		},
		{
			name:     "exact fit",
			text:     "Exactly Ten",
			maxChars: 11,
			want:     []string{"Exactly Ten"},
		},
		{
			name:     "hard split in middle of processing",
			text:     "Normal VeryVeryVeryLongWord End",
			maxChars: 10,
			want:     []string{"Normal", "VeryVeryVe", "…"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := wrapLabel(tt.text, tt.maxChars)
			if len(got) != len(tt.want) {
				t.Errorf("wrapLabel() length = %v, want %v\nGot: %v\nWant: %v",
					len(got), len(tt.want), got, tt.want)
				return
			}
			for i := range got {
				if got[i] != tt.want[i] {
					t.Errorf("wrapLabel()[%d] = %v, want %v\nGot all: %v\nWant all: %v",
						i, got[i], tt.want[i], got, tt.want)
				}
			}
		})
	}
}

func TestWrapLabel_EdgeCases(t *testing.T) {
	// Test that maxChars < 3 is normalized to 3
	result := wrapLabel("Test", 1)
	if len(result) == 0 {
		t.Error("Expected non-empty result even with maxChars=1")
	}

	// Test with zero maxChars
	result = wrapLabel("Test", 0)
	if len(result) == 0 {
		t.Error("Expected non-empty result even with maxChars=0")
	}

	// Test with negative maxChars
	result = wrapLabel("Test", -5)
	if len(result) == 0 {
		t.Error("Expected non-empty result even with negative maxChars")
	}
}

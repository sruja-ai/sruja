package language

import "testing"

func TestIsIdent(t *testing.T) {
	cases := []struct {
		s    string
		want bool
	}{
		{"valid", true},
		{"with_underscore1", true},
		{"9starts", false},
		{"has-hyphen", false},
		{"", false},
	}
	for _, c := range cases {
		if isIdent(c.s) != c.want {
			t.Fatalf("isIdent(%q)=%v want %v", c.s, isIdent(c.s), c.want)
		}
	}
}

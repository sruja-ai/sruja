package language

import "testing"

func TestBaseName(t *testing.T) {
    cases := []struct{ in, want string }{
        {"/path/to/file.sruja", "file"},
        {"file.sruja", "file"},
        {"/path/", "/path/"},
        {"", "Untitled"},
        {"noext", "noext"},
    }
    for _, c := range cases {
        if got := baseName(c.in); got != c.want {
            t.Fatalf("baseName(%q)=%q want %q", c.in, got, c.want)
        }
    }
}

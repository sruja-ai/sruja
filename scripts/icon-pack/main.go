package main

import (
	"encoding/json"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

// Reads SVG files under shared/icons/src and writes a JSON map of name->inner SVG content
// Output: pkg/export/svg/icons/icons.json
func main() {
	srcDir := filepath.Join("shared", "icons", "src")
	outPath := filepath.Join("pkg", "export", "svg", "icons", "icons.json")

	icons := map[string]string{}
	re := regexp.MustCompile(`(?is)<svg[^>]*>(.*)</svg>`)

	err := filepath.WalkDir(srcDir, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			return nil
		}
		if !strings.HasSuffix(path, ".svg") {
			return nil
		}
		b, err := os.ReadFile(path) // #nosec G304 // variable path is intended behavior
		if err != nil {
			return err
		}
		s := string(b)
		inner := s
		m := re.FindStringSubmatch(s)
		if len(m) == 2 {
			inner = strings.TrimSpace(m[1])
		}
		name := strings.TrimSuffix(filepath.Base(path), ".svg")
		icons[name] = inner
		return nil
	})
	if err != nil {
		panic(err)
	}

	if err := os.MkdirAll(filepath.Dir(outPath), 0o755); err != nil {
		panic(err)
	} // #nosec G301 // 0755 is standard for shared dirs
	out, err := os.Create(outPath) // #nosec G304 // user defined path
	if err != nil {
		panic(err)
	}
	defer out.Close()
	enc := json.NewEncoder(out)
	enc.SetIndent("", "  ")
	if err := enc.Encode(icons); err != nil {
		panic(err)
	}

	fmt.Printf("Wrote %s with %d icons\n", outPath, len(icons))
}

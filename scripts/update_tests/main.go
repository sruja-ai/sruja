package main

import (
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

func main() {
	// Target directories
	dirs := []string{"pkg/engine", "pkg/lsp", "pkg/language", "cmd/sruja", "apps/designer/public/examples", "packages/shared"}

	for _, dir := range dirs {
		err := filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}
			if info.IsDir() {
				return nil
			}
			if !strings.HasSuffix(path, ".go") {
				return nil
			}

			return processFile(path)
		})

		if err != nil {
			fmt.Printf("Error walking directory %s: %v\n", dir, err)
		}
	}
}

func processFile(path string) error {
	content, err := ioutil.ReadFile(path)
	if err != nil {
		return err
	}

	strContent := string(content)

	// Regex to match Go raw string literals: `...`
	// \x60 is backtick
	re := regexp.MustCompile(`(?s)(\x60)([\s\S]*?)(\x60)`)

	newContent := re.ReplaceAllStringFunc(strContent, func(match string) string {
		parts := re.FindStringSubmatch(match)
		if len(parts) < 4 {
			return match
		}
		header := parts[1]
		code := parts[2]
		footer := parts[3]

		// Only process if it looks like DSL (contains "model {" or "specification {" or "views {")
		if !strings.Contains(code, "model {") && !strings.Contains(code, "specification {") && !strings.Contains(code, "views {") {
			return match
		}

		newCode := processCode(code)
		if newCode != code {
			fmt.Printf("Updating DSL in %s\n", path)
			return header + newCode + footer
		}
		return match
	})

	if newContent != strContent {
		err = ioutil.WriteFile(path, []byte(newContent), 0644)
		if err != nil {
			return fmt.Errorf("failed to write file %s: %v", path, err)
		}
	}
	return nil
}

func processCode(code string) string {
	code = removeBlock(code, "specification")
	code = removeBlock(code, "model")
	code = removeBlock(code, "views")
	return code
}

func removeBlock(code string, blockName string) string {
	trimmed := strings.TrimSpace(code)
	// Handle single-line or simple wrapper case: starts with "blockName {" and ends with "}"
	// This covers `model { ... }` where content is inside.
	prefix := blockName + " {"
	if strings.HasPrefix(trimmed, prefix) && strings.HasSuffix(trimmed, "}") {
		// potential match, check if it's the matching brace
		// For single line, it is likely. For multiline, we need to be careful.
		// Let's rely on the fact that these are test strings wrapping the whole DSL.
		// If code starts with `model {` and ends with `}`, we strip them.

		// Verifying that the last } matches the first {.
		// If we strip prefix and suffix, we should have balanced braces inside?
		// We can't guarantee, but for test inputs it's 99% likely correct.
		// We also need to trim whitespace after stripping.

		// Check strictly for start
		// Use regex for flexibility with whitespace
		reStart := regexp.MustCompile(`^\s*` + blockName + `\s*\{\s*`)
		loc := reStart.FindStringIndex(code)
		if loc != nil && loc[0] == 0 {
			// Found start at beginning
			// Check end
			reEnd := regexp.MustCompile(`\s*\}\s*$`)
			endLoc := reEnd.FindStringIndex(code)
			if endLoc != nil {
				// We have start and end.
				// Remove them.
				// But wait, what if `model { ... } other { ... }`?
				// The regex end matches end of string.
				// If we have `model { } \n model { }` it would be invalid DSL anyway (two models).
				// So we assume one block wraps everything or nothing.

				// Take content between
				inner := code[loc[1]:endLoc[0]]
				return strings.TrimSpace(inner)
			}
		}
	}

	lines := strings.Split(code, "\n")

	processedLines := make([]string, 0, len(lines))

	depth := 0
	targetBlockDepth := -1

	// Regex for start of block: simple "blockName {" allowed with optional whitespace
	// Note: In tests, spacing varies.
	startRe := regexp.MustCompile(`^\s*` + blockName + `\s*\{\s*$`)

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)

		if targetBlockDepth == -1 {
			if startRe.MatchString(line) {
				targetBlockDepth = depth
				depth++
				continue
			}
			depth += countDepthChange(line)
			processedLines = append(processedLines, line)
		} else {
			change := countDepthChange(line)
			if change < 0 && (depth+change) == targetBlockDepth {
				if trimmed == "}" {
					depth += change
					targetBlockDepth = -1
					continue
				}
			}
			depth += change

			// Dedent
			dedented := line
			if strings.HasPrefix(line, "\t") {
				dedented = line[1:]
			} else if strings.HasPrefix(line, "    ") {
				dedented = line[4:]
			} else if strings.HasPrefix(line, "  ") {
				dedented = line[2:]
			}
			processedLines = append(processedLines, dedented)
		}
	}

	return strings.Join(processedLines, "\n")
}

func countDepthChange(line string) int {
	clean := line
	opens := strings.Count(clean, "{")
	closes := strings.Count(clean, "}")
	return opens - closes
}

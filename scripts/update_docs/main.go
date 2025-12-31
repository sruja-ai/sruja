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
	rootDir := "apps/website/src/content"
	err := filepath.Walk(rootDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}
		if !strings.HasSuffix(path, ".md") && !strings.HasSuffix(path, ".mdx") {
			return nil
		}

		return processFile(path)
	})

	if err != nil {
		fmt.Printf("Error walking directory: %v\n", err)
		os.Exit(1)
	}
}

func processFile(path string) error {
	content, err := ioutil.ReadFile(path)
	if err != nil {
		return err
	}

	strContent := string(content)

	// Regex to match sruja code blocks
	// We capture the content inside ```sruja ... ```
	re := regexp.MustCompile(`(?s)(` + "```" + `sruja)([\s\S]*?)(` + "```" + `)`)

	newContent := re.ReplaceAllStringFunc(strContent, func(match string) string {
		parts := re.FindStringSubmatch(match)
		if len(parts) < 4 {
			return match
		}
		header := parts[1]
		code := parts[2]
		footer := parts[3]

		newCode := processCode(code)
		if newCode != code {
			fmt.Printf("Updating block in %s\n", path)
			return header + newCode + footer
		}
		return match
	})

	if newContent != strContent {
		err = ioutil.WriteFile(path, []byte(newContent), 0644)
		if err != nil {
			return fmt.Errorf("failed to write file %s: %v", path, err)
		}
		fmt.Printf("Updated %s\n", path)
	}
	return nil
}

func processCode(code string) string {
	// Remove specification { ... }
	code = removeBlock(code, "specification")
	// Remove model { ... }
	code = removeBlock(code, "model")
	// Remove views { ... }
	code = removeBlock(code, "views")
	return code
}

func removeBlock(code string, blockName string) string {
	// Simple regex to find "blockName { content }"
	// This is tricky with nested braces, but tutorials usually have simple structure
	// Let's try to match "blockName {" and corresponding "}"
	// But simply replacing "blockName {" with empty and removing the LAST "}" if it matches structure?
	// Or better: Regex for `blockName\s*{` and then dedent content until `}`

	// Actually, many blocks are like:
	// model {
	//   ...
	// }
	// We can remove "model {" and "}" line if they are on separate lines.

	lines := strings.Split(code, "\n")

	// This is a naive state machine, assuming standard formatting in docs
	// It handles one block type at a time.
	// If multiple blocks (e.g. specification {...} model {...}) are present, it handles them sequentially if we validly detect them.
	// But we iterate lines once? No, let's do regex replacement for well-formatted blocks first.

	// Strategy:
	// 1. Find line with `^\s*blockName\s*\{`
	// 2. Remove that line.
	// 3. For subsequent lines, dedent if they were indented relative to block.
	// 4. Find matching `^\s*\}` line and remove it.
	//
	// Issues: Nested blocks? (e.g. System { ... }). We shouldn't remove checking for closing brace of System.
	// But `model` and `specification` are top level. So their closing brace is usually at the same indentation level (usually 0).

	// Let's try a line-based filter.

	processedLines := make([]string, 0, len(lines))

	// We need to track depth to identify which closing brace belongs to the block
	depth := 0
	targetBlockDepth := -1 // Depth where the block started

	// Regex for start of block
	startRe := regexp.MustCompile(`^\s*` + blockName + `\s*\{\s*$`)

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)

		// If we are looking for the block start
		if targetBlockDepth == -1 {
			if startRe.MatchString(line) {
				// Found block start
				targetBlockDepth = depth
				depth++ // Entering block
				// Do not append this line (we are removing the wrapper)
				continue
			}

			// Track depth for other braces (e.g. System { ... })
			depth += countDepthChange(line)
			processedLines = append(processedLines, line)
		} else {
			// We are inside the block

			// Check if this is the closing brace for the block
			// The closing brace should be at targetBlockDepth (since we incremented depth when entering)
			// Wait, if we are at depth 1 (inside block), and we see `}`, depth becomes 0.
			// So if we see `}` and it brings depth back to targetBlockDepth, it's the valid one.

			change := countDepthChange(line)

			// If this line is JUST `}` and it closes the block
			if change < 0 && (depth+change) == targetBlockDepth {
				if trimmed == "}" {
					depth += change
					targetBlockDepth = -1 // Exited block
					// Do not append this line
					continue
				}
				// If line has more than `}`, it's complicated. But docs usually have `}` on own line.
			}

			depth += change

			// Dedent: if line is indented, remove one level of indentation (usually tab or 2/4 spaces)
			// We can try to be smart or just strip consistent prefix if detected.
			// Simple heuristics: remove leading tab or 2/4 spaces
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
	// remove strings/comments to avoid false positives
	// (Simple version)
	clean := line
	// Count { and }
	opens := strings.Count(clean, "{")
	closes := strings.Count(clean, "}")
	return opens - closes
}

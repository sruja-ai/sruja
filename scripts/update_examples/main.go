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
	// Target directories for .sruja files
	dirs := []string{"apps/designer/public/examples", "examples"}

	for _, dir := range dirs {
		err := filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}
			if info.IsDir() {
				return nil
			}
			if !strings.HasSuffix(path, ".sruja") {
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

	// Only process if it looks like DSL (contains "model {" or "specification {" or "views {")
	if !strings.Contains(strContent, "model {") && !strings.Contains(strContent, "specification {") && !strings.Contains(strContent, "views {") {
		return nil
	}

	newContent := processCode(strContent)
	if newContent != strContent {
		fmt.Printf("Updating DSL in %s\n", path)
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
	lines := strings.Split(code, "\n")
	processedLines := make([]string, 0, len(lines))
	depth := 0
	targetBlockDepth := -1

	// Regex for start of block
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
	opens := strings.Count(line, "{")
	closes := strings.Count(line, "}")
	return opens - closes
}
